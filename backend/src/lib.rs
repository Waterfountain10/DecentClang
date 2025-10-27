//! llvm ir → x86 compiler backend
//!
//! translates llvm ir (from llvm crate) into x86 assembly (x86 crate)
//! output can be fed directly into assembler which produces exec

use llvm;
use std::collections::HashMap;
use x86::asm::*;
use x86::{Cnd, Data, Elem, Imm, Ins, Opcode, Operand, Prog, Reg};

// --- context for compilation ---

/// stack layout: maps uid → stack slot offset from rbp
type Layout = HashMap<llvm::Uid, Operand>;

/// compilation context: type decls + stack layout
struct Ctxt {
    tdecls: HashMap<llvm::Tid, llvm::Ty>,
    layout: Layout,
}

// --- helpers ---

/// map ll cmp ops → x86 condition codes
fn compile_cnd(c: &llvm::Cnd) -> Cnd {
    match c {
        llvm::Cnd::Eq => Cnd::Eq,
        llvm::Cnd::Ne => Cnd::Neq,
        llvm::Cnd::Slt => Cnd::Lt,
        llvm::Cnd::Sle => Cnd::Le,
        llvm::Cnd::Sgt => Cnd::Gt,
        llvm::Cnd::Sge => Cnd::Ge,
    }
}

/// size of ty in bytes (for gep calculations)
fn size_ty(tdecls: &HashMap<llvm::Tid, llvm::Ty>, t: &llvm::Ty) -> usize {
    match t {
        llvm::Ty::Void | llvm::Ty::I8 | llvm::Ty::Fun(_, _) => 0,
        llvm::Ty::Ptr(_) | llvm::Ty::I1 | llvm::Ty::I64 => 8,
        llvm::Ty::Array(n, inner) => n * size_ty(tdecls, inner),
        llvm::Ty::Struct(tys) => tys.iter().map(|t| size_ty(tdecls, t)).sum(),
        llvm::Ty::Namedt(tid) => {
            if let Some(resolved) = tdecls.get(tid) {
                size_ty(tdecls, resolved)
            } else {
                0
            }
        }
    }
}

/// resolve named types recursively
fn resolve_ty<'a>(tdecls: &'a HashMap<llvm::Tid, llvm::Ty>, t: &'a llvm::Ty) -> &'a llvm::Ty {
    match t {
        llvm::Ty::Namedt(tid) => {
            if let Some(resolved) = tdecls.get(tid) {
                resolve_ty(tdecls, resolved)
            } else {
                t
            }
        }
        _ => t,
    }
}

/// check if n is power of 2
fn is_pow2(n: usize) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

/// log2 of n (assumes n is pow2)
fn log2(mut n: usize) -> usize {
    let mut result = 0;
    while n > 1 {
        n >>= 1;
        result += 1;
    }
    result
}

// --- operand compilation ---

/// compile ll operand → x86 ins that moves it into dest reg
/// dest must be a register
fn compile_operand(ctxt: &Ctxt, dest: Operand, op: &llvm::Operand) -> Ins {
    let reg_dest = match dest {
        Operand::Reg(r) => r,
        _ => panic!("compile_operand: dest must be reg"),
    };

    match op {
        llvm::Operand::Null => Ins {
            opcode: Opcode::Movq,
            operands: vec![Operand::Imm(Imm::Lit(0)), Operand::Reg(reg_dest)],
        },
        llvm::Operand::Const(c) => Ins {
            opcode: Opcode::Movq,
            operands: vec![Operand::Imm(Imm::Lit(*c)), Operand::Reg(reg_dest)],
        },
        llvm::Operand::Gid(g) => Ins {
            opcode: Opcode::Leaq,
            operands: vec![
                Operand::Ind3(Imm::Lbl(mangle(g)), Reg::Rip),
                Operand::Reg(reg_dest),
            ],
        },
        llvm::Operand::Id(uid) => {
            let slot = ctxt.layout.get(uid).expect("uid not in layout");
            Ins {
                opcode: Opcode::Movq,
                operands: vec![slot.clone(), Operand::Reg(reg_dest)],
            }
        }
    }
}

// --- gep compilation ---

/// compile getelementptr (address calculation with type-aware indexing)
fn compile_gep(
    ctxt: &Ctxt,
    base_ty: &llvm::Ty,
    base_op: &llvm::Operand,
    path: &[llvm::Operand],
) -> Vec<Ins> {
    let mut insns = Vec::new();

    // base addr → rax
    let base_elt_ty = match resolve_ty(&ctxt.tdecls, base_ty) {
        llvm::Ty::Ptr(inner) => resolve_ty(&ctxt.tdecls, inner),
        _ => panic!("gep: base type must be ptr"),
    };
    insns.push(compile_operand(ctxt, Operand::Reg(Reg::Rax), base_op));

    // walk path: first idx is array offset into base_elt_ty
    if path.is_empty() {
        return insns;
    }

    // helper: add immediate offset to rax
    let add_imm = |insns: &mut Vec<Ins>, bytes: i64| {
        if bytes != 0 {
            insns.push(Ins {
                opcode: Opcode::Addq,
                operands: vec![Operand::Imm(Imm::Lit(bytes)), Operand::Reg(Reg::Rax)],
            });
        }
    };

    // helper: add scaled index to rax (idx * elem_sz)
    let add_scaled = |insns: &mut Vec<Ins>, idx_op: &llvm::Operand, elem_sz: usize| {
        if let llvm::Operand::Const(c) = idx_op {
            add_imm(insns, *c * elem_sz as i64);
        } else {
            insns.push(compile_operand(ctxt, Operand::Reg(Reg::R11), idx_op));
            if elem_sz == 0 {
                // nothing to add
            } else if is_pow2(elem_sz) {
                insns.push(Ins {
                    opcode: Opcode::Shlq,
                    operands: vec![
                        Operand::Imm(Imm::Lit(log2(elem_sz) as i64)),
                        Operand::Reg(Reg::R11),
                    ],
                });
                insns.push(Ins {
                    opcode: Opcode::Addq,
                    operands: vec![Operand::Reg(Reg::R11), Operand::Reg(Reg::Rax)],
                });
            } else {
                insns.push(Ins {
                    opcode: Opcode::Imulq,
                    operands: vec![
                        Operand::Imm(Imm::Lit(elem_sz as i64)),
                        Operand::Reg(Reg::R11),
                    ],
                });
                insns.push(Ins {
                    opcode: Opcode::Addq,
                    operands: vec![Operand::Reg(Reg::R11), Operand::Reg(Reg::Rax)],
                });
            }
        }
    };

    let sz0 = size_ty(&ctxt.tdecls, base_elt_ty);
    add_scaled(&mut insns, &path[0], sz0);

    // subsequent indices depend on type
    let mut cur_ty = base_elt_ty;
    for idx_op in &path[1..] {
        match resolve_ty(&ctxt.tdecls, cur_ty) {
            llvm::Ty::Struct(fields) => {
                // struct: idx must be const, calc field offset
                let field_idx = match idx_op {
                    llvm::Operand::Const(c) => *c as usize,
                    _ => panic!("gep: struct index must be const"),
                };
                let offset: usize = fields
                    .iter()
                    .take(field_idx)
                    .map(|t| size_ty(&ctxt.tdecls, t))
                    .sum();
                add_imm(&mut insns, offset as i64);
                cur_ty = &fields[field_idx];
            }
            llvm::Ty::Array(_, elem_ty) => {
                let esz = size_ty(&ctxt.tdecls, elem_ty);
                add_scaled(&mut insns, idx_op, esz);
                cur_ty = elem_ty;
            }
            _ => panic!("gep: invalid path for type"),
        }
    }

    insns
}

// --- instruction compilation ---

/// compile single llvm insn → x86 insns
fn compile_insn(ctxt: &Ctxt, uid: &llvm::Uid, insn: &llvm::Insn) -> Vec<Ins> {
    let dst = ctxt.layout.get(uid).expect("uid not in layout").clone();
    let mut result = Vec::new();

    match insn {
        llvm::Insn::Binop(bop, _ty, x1, x2) => {
            result.push(compile_operand(ctxt, Operand::Reg(Reg::Rax), x1));
            result.push(compile_operand(ctxt, Operand::Reg(Reg::Rbx), x2));
            let opcode = match bop {
                llvm::Bop::Add => Opcode::Addq,
                llvm::Bop::Sub => Opcode::Subq,
                llvm::Bop::Mul => Opcode::Imulq,
                llvm::Bop::Shl => {
                    result.push(Ins {
                        opcode: Opcode::Movq,
                        operands: vec![Operand::Reg(Reg::Rbx), Operand::Reg(Reg::Rcx)],
                    });
                    Opcode::Shlq
                }
                llvm::Bop::Lshr => {
                    result.push(Ins {
                        opcode: Opcode::Movq,
                        operands: vec![Operand::Reg(Reg::Rbx), Operand::Reg(Reg::Rcx)],
                    });
                    Opcode::Shrq
                }
                llvm::Bop::Ashr => {
                    result.push(Ins {
                        opcode: Opcode::Movq,
                        operands: vec![Operand::Reg(Reg::Rbx), Operand::Reg(Reg::Rcx)],
                    });
                    Opcode::Sarq
                }
                llvm::Bop::And => Opcode::Andq,
                llvm::Bop::Or => Opcode::Orq,
                llvm::Bop::Xor => Opcode::Xorq,
            };
            let src = if matches!(bop, llvm::Bop::Shl | llvm::Bop::Lshr | llvm::Bop::Ashr) {
                Operand::Reg(Reg::Rcx)
            } else {
                Operand::Reg(Reg::Rbx)
            };
            result.push(Ins {
                opcode,
                operands: vec![src, Operand::Reg(Reg::Rax)],
            });
            result.push(Ins {
                opcode: Opcode::Movq,
                operands: vec![Operand::Reg(Reg::Rax), dst],
            });
        }

        llvm::Insn::Alloca(ty) => {
            let nbytes = size_ty(&ctxt.tdecls, ty);
            result.push(Ins {
                opcode: Opcode::Subq,
                operands: vec![
                    Operand::Imm(Imm::Lit(nbytes as i64)),
                    Operand::Reg(Reg::Rsp),
                ],
            });
            result.push(Ins {
                opcode: Opcode::Movq,
                operands: vec![Operand::Reg(Reg::Rsp), dst],
            });
        }

        llvm::Insn::Load(_ty, ptr) => {
            result.push(compile_operand(ctxt, Operand::Reg(Reg::Rax), ptr));
            result.push(Ins {
                opcode: Opcode::Movq,
                operands: vec![Operand::Ind2(Reg::Rax), Operand::Reg(Reg::Rbx)],
            });
            result.push(Ins {
                opcode: Opcode::Movq,
                operands: vec![Operand::Reg(Reg::Rbx), dst],
            });
        }

        llvm::Insn::Store(_ty, val, ptr) => {
            result.push(compile_operand(ctxt, Operand::Reg(Reg::Rax), ptr));
            result.push(compile_operand(ctxt, Operand::Reg(Reg::Rbx), val));
            result.push(Ins {
                opcode: Opcode::Movq,
                operands: vec![Operand::Reg(Reg::Rbx), Operand::Ind2(Reg::Rax)],
            });
        }

        llvm::Insn::Icmp(cnd, _ty, x1, x2) => {
            result.push(compile_operand(ctxt, Operand::Reg(Reg::Rax), x1));
            result.push(compile_operand(ctxt, Operand::Reg(Reg::Rbx), x2));
            result.push(Ins {
                opcode: Opcode::Cmpq,
                operands: vec![Operand::Reg(Reg::Rbx), Operand::Reg(Reg::Rax)],
            });
            result.push(Ins {
                opcode: Opcode::Movq,
                operands: vec![Operand::Imm(Imm::Lit(0)), dst.clone()],
            });
            result.push(Ins {
                opcode: Opcode::Set(compile_cnd(cnd)),
                operands: vec![dst],
            });
        }

        llvm::Insn::Call(ret_ty, callee, args) => {
            // arg regs: rdi, rsi, rdx, rcx, r8, r9, then stack
            let arg_regs = [Reg::Rdi, Reg::Rsi, Reg::Rdx, Reg::Rcx, Reg::R08, Reg::R09];
            for (i, (_ty, arg_op)) in args.iter().enumerate() {
                if i < 6 {
                    result.push(compile_operand(ctxt, Operand::Reg(arg_regs[i]), arg_op));
                } else {
                    result.push(compile_operand(ctxt, Operand::Reg(Reg::Rax), arg_op));
                    result.push(Ins {
                        opcode: Opcode::Pushq,
                        operands: vec![Operand::Reg(Reg::Rax)],
                    });
                }
            }

            // call
            match callee {
                llvm::Operand::Id(f) => {
                    result.push(Ins {
                        opcode: Opcode::Callq,
                        operands: vec![Operand::Imm(Imm::Lbl(mangle(f)))],
                    });
                }
                _ => {
                    result.push(compile_operand(ctxt, Operand::Reg(Reg::Rax), callee));
                    result.push(Ins {
                        opcode: Opcode::Callq,
                        operands: vec![Operand::Reg(Reg::Rax)],
                    });
                }
            }

            // store return val if non-void
            if !matches!(ret_ty, llvm::Ty::Void) {
                result.push(Ins {
                    opcode: Opcode::Movq,
                    operands: vec![Operand::Reg(Reg::Rax), dst],
                });
            }
        }

        llvm::Insn::Bitcast(_t1, op, _t2) => {
            result.push(compile_operand(ctxt, Operand::Reg(Reg::Rax), op));
            result.push(Ins {
                opcode: Opcode::Movq,
                operands: vec![Operand::Reg(Reg::Rax), dst],
            });
        }

        llvm::Insn::Gep(ty, base, path) => {
            result.extend(compile_gep(ctxt, ty, base, path));
            result.push(Ins {
                opcode: Opcode::Movq,
                operands: vec![Operand::Reg(Reg::Rax), dst],
            });
        }
    }

    result
}

// --- terminator compilation ---

/// make label unique to function: fn.label
fn mk_lbl(fname: &str, lbl: &llvm::Lbl) -> String {
    format!("{}.{}", fname, lbl)
}

/// compile block terminator → x86 insns
fn compile_terminator(fname: &str, ctxt: &Ctxt, term: &llvm::Terminator) -> Vec<Ins> {
    match term {
        llvm::Terminator::Ret(llvm::Ty::Void, _) => vec![
            Ins {
                opcode: Opcode::Movq,
                operands: vec![Operand::Reg(Reg::Rbp), Operand::Reg(Reg::Rsp)],
            },
            Ins {
                opcode: Opcode::Popq,
                operands: vec![Operand::Reg(Reg::Rbp)],
            },
            Ins {
                opcode: Opcode::Retq,
                operands: vec![],
            },
        ],

        llvm::Terminator::Ret(_ty, Some(op)) => vec![
            compile_operand(ctxt, Operand::Reg(Reg::Rax), op),
            Ins {
                opcode: Opcode::Movq,
                operands: vec![Operand::Reg(Reg::Rbp), Operand::Reg(Reg::Rsp)],
            },
            Ins {
                opcode: Opcode::Popq,
                operands: vec![Operand::Reg(Reg::Rbp)],
            },
            Ins {
                opcode: Opcode::Retq,
                operands: vec![],
            },
        ],

        llvm::Terminator::Br(lbl) => vec![Ins {
            opcode: Opcode::Jmp,
            operands: vec![Operand::Imm(Imm::Lbl(mk_lbl(fname, lbl)))],
        }],

        llvm::Terminator::Cbr(cond, lbl_true, lbl_false) => vec![
            compile_operand(ctxt, Operand::Reg(Reg::Rax), cond),
            Ins {
                opcode: Opcode::Cmpq,
                operands: vec![Operand::Imm(Imm::Lit(0)), Operand::Reg(Reg::Rax)],
            },
            Ins {
                opcode: Opcode::J(Cnd::Eq),
                operands: vec![Operand::Imm(Imm::Lbl(mk_lbl(fname, lbl_false)))],
            },
            Ins {
                opcode: Opcode::Jmp,
                operands: vec![Operand::Imm(Imm::Lbl(mk_lbl(fname, lbl_true)))],
            },
        ],

        _ => panic!("unrecognized terminator"),
    }
}

// --- block compilation ---

/// compile llvm block → x86 insns
fn compile_block(fname: &str, ctxt: &Ctxt, blk: &llvm::Block) -> Vec<Ins> {
    let mut result = Vec::new();
    for (uid, insn) in &blk.insns {
        result.extend(compile_insn(ctxt, uid, insn));
    }
    let (_term_uid, term) = &blk.term;
    result.extend(compile_terminator(fname, ctxt, term));
    result
}

/// compile labeled block → x86 elem
fn compile_lbl_block(fname: &str, lbl: &llvm::Lbl, ctxt: &Ctxt, blk: &llvm::Block) -> Elem {
    text(&mk_lbl(fname, lbl), compile_block(fname, ctxt, blk))
}

// --- function compilation ---

/// x86 calling convention: arg locations
fn arg_loc(n: usize) -> Operand {
    match n {
        0 => Operand::Reg(Reg::Rdi),
        1 => Operand::Reg(Reg::Rsi),
        2 => Operand::Reg(Reg::Rdx),
        3 => Operand::Reg(Reg::Rcx),
        4 => Operand::Reg(Reg::R08),
        5 => Operand::Reg(Reg::R09),
        k => Operand::Ind3(Imm::Lit((16 + 8 * (k - 6)) as i64), Reg::Rbp),
    }
}

/// build stack layout: each uid gets a stack slot (offset from rbp)
fn stack_layout(args: &[llvm::Uid], cfg: &llvm::Cfg) -> Layout {
    let mut layout = HashMap::new();
    let (entry_blk, labeled_blks) = cfg;

    let mut all_uids = args.to_vec();
    for (uid, _) in &entry_blk.insns {
        all_uids.push(uid.clone());
    }
    for (_, blk) in labeled_blks {
        for (uid, _) in &blk.insns {
            all_uids.push(uid.clone());
        }
    }

    for (i, uid) in all_uids.iter().enumerate() {
        let offset = -(((i + 1) * 8) as i64);
        layout.insert(uid.clone(), Operand::Ind3(Imm::Lit(offset), Reg::Rbp));
    }

    layout
}

/// compile function decl → x86 prog (multiple elems)
pub fn compile_fdecl(
    tdecls: &[(llvm::Tid, llvm::Ty)],
    name: &str,
    fdecl: &llvm::Fdecl,
) -> Vec<Elem> {
    let fname = mangle(name);
    let tdecls_map: HashMap<_, _> = tdecls.iter().cloned().collect();
    let layout = stack_layout(&fdecl.f_param, &fdecl.f_cfg);
    let frame_size = layout.len() * 8;

    let ctxt = Ctxt {
        tdecls: tdecls_map,
        layout,
    };

    // prologue
    let mut prologue = vec![
        Ins {
            opcode: Opcode::Pushq,
            operands: vec![Operand::Reg(Reg::Rbp)],
        },
        Ins {
            opcode: Opcode::Movq,
            operands: vec![Operand::Reg(Reg::Rsp), Operand::Reg(Reg::Rbp)],
        },
        Ins {
            opcode: Opcode::Subq,
            operands: vec![
                Operand::Imm(Imm::Lit(frame_size as i64)),
                Operand::Reg(Reg::Rsp),
            ],
        },
    ];

    // move args to stack slots
    for (i, uid) in fdecl.f_param.iter().enumerate() {
        let dst = ctxt.layout.get(uid).expect("arg not in layout").clone();
        prologue.push(Ins {
            opcode: Opcode::Movq,
            operands: vec![arg_loc(i), Operand::Reg(Reg::Rax)],
        });
        prologue.push(Ins {
            opcode: Opcode::Movq,
            operands: vec![Operand::Reg(Reg::Rax), dst],
        });
    }

    // compile cfg
    let (entry_blk, labeled_blks) = &fdecl.f_cfg;
    let mut body = compile_block(&fname, &ctxt, entry_blk);

    // combine
    let mut all_insns = prologue;
    all_insns.append(&mut body);

    let mut result = vec![gtext(&fname, all_insns)];

    // compile labeled blocks
    for (lbl, blk) in labeled_blks {
        result.push(compile_lbl_block(&fname, lbl, &ctxt, blk));
    }

    result
}

// --- global data compilation ---

/// compile global init → x86 data
fn compile_ginit(g: &llvm::Ginit) -> Vec<Data> {
    match g {
        llvm::Ginit::GNull => vec![Data::Quad(Imm::Lit(0))],
        llvm::Ginit::GGid(gid) => vec![Data::Quad(Imm::Lbl(mangle(gid)))],
        llvm::Ginit::GInt(c) => vec![Data::Quad(Imm::Lit(*c))],
        llvm::Ginit::GString(s) => vec![Data::Asciz(s.clone())],
        llvm::Ginit::GArray(elems) | llvm::Ginit::GStruct(elems) => {
            elems.iter().flat_map(|(_ty, g)| compile_ginit(g)).collect()
        }
        llvm::Ginit::GBitcast(_t1, g, _t2) => compile_ginit(g),
    }
}

/// compile global decl → x86 data elem
fn compile_gdecl(lbl: &str, gdecl: &llvm::Gdecl) -> Elem {
    let (_ty, ginit) = gdecl;
    data(&mangle(lbl), compile_ginit(ginit))
}

// --- program compilation ---

/// compile full llvm prog → x86 prog
pub fn compile_prog(prog: &llvm::Prog) -> Prog {
    let mut result = Vec::new();

    // compile globals
    for (gid, gdecl) in &prog.gdecls {
        result.push(compile_gdecl(gid, gdecl));
    }

    // compile functions
    for (fname, fdecl) in &prog.fdecls {
        result.extend(compile_fdecl(&prog.tdecls, fname, fdecl));
    }

    // wrapper for 'program' label → 'main' (ocaml compat)
    if prog.fdecls.iter().any(|(n, _)| n == "program") {
        let wrapper = gtext(
            &mangle("main"),
            vec![
                Ins {
                    opcode: Opcode::Callq,
                    operands: vec![Operand::Imm(Imm::Lbl(mangle("program")))],
                },
                Ins {
                    opcode: Opcode::Retq,
                    operands: vec![],
                },
            ],
        );
        result.push(wrapper);
    }

    Prog(result)
}

// --- platform-specific mangling ---

/// platform-specific label mangling (macos adds '_', linux doesn't)
fn mangle(s: &str) -> String {
    if cfg!(target_os = "macos") {
        format!("_{}", s)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_ty() {
        let tdecls = HashMap::new();
        assert_eq!(size_ty(&tdecls, &llvm::Ty::I64), 8);
        assert_eq!(size_ty(&tdecls, &llvm::Ty::Ptr(Box::new(llvm::Ty::I8))), 8);
        assert_eq!(
            size_ty(&tdecls, &llvm::Ty::Array(10, Box::new(llvm::Ty::I64))),
            80
        );
    }

    #[test]
    fn test_is_pow2() {
        assert!(is_pow2(1));
        assert!(is_pow2(2));
        assert!(is_pow2(4));
        assert!(is_pow2(8));
        assert!(!is_pow2(3));
        assert!(!is_pow2(6));
    }

    #[test]
    fn test_log2() {
        assert_eq!(log2(1), 0);
        assert_eq!(log2(2), 1);
        assert_eq!(log2(4), 2);
        assert_eq!(log2(8), 3);
    }

    #[test]
    fn test_compile_cnd() {
        assert!(matches!(compile_cnd(&llvm::Cnd::Eq), Cnd::Eq));
        assert!(matches!(compile_cnd(&llvm::Cnd::Ne), Cnd::Neq));
        assert!(matches!(compile_cnd(&llvm::Cnd::Slt), Cnd::Lt));
        assert!(matches!(compile_cnd(&llvm::Cnd::Sle), Cnd::Le));
        assert!(matches!(compile_cnd(&llvm::Cnd::Sgt), Cnd::Gt));
        assert!(matches!(compile_cnd(&llvm::Cnd::Sge), Cnd::Ge));
    }
}
