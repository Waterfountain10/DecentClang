//! Typechecking flow
//!
//! Typechecking rules from CS-4212 definitions pdf page :
//! !!!!!!!!!!!!!!!!!insert link dont forget
//!
//! The main call works from biggest to smallest data structure.
//! i.e we typecheck in the following order:
//!
//! Flow of our Typechecker:
//!
//! ```text
//! ast program
//!   ↓  (create struct H, function L,and global G contexts)
//!   ↓  (iterate through ast program and typecheck recursively)
//! fdecl, tdcecl
//!   ↓
//! block (main scope or function scope)
//!   ↓
//! statement (if statment or assignement)
//!   ↓
//! expression (var or integer)
//!   ↓
//! types/ret_types
//!   ↓  (check for both well-types rules AND subtyping rules )
//! end of typecheck flow
//! ```
//!

use ast::BinOp;
use ast::Exp;
use ast::RefTy;
use ast::RetTy;
use ast::Ty;

use ::common::TypeErrorKind;
use typechecker::*;

// fn typecheck_program(p: ast::Prog) -> Result<(), ()> {
//     for decl in p {
//         match &decl {
//             GFDecl(elt) => {
//                 let f = elt
//                 let res = typecheck_fdecl(tc, f, l)
//             }
//             GTDecl(_) => {
//                 let res = typecheck_t
//             }
//             () =>
//         }
//     }

//     Ok(res)
// }
//

// subtyping ---------------------------------------------------------------- *)
//  Decides whether H |- t1 <: t2
//     - assumes that H contains the declarations of all the possible struct types
//
fn subtype(h: &TypeCtxt, t1: &ast::STy, t2: &ast::STy) -> bool {
    match (&t1.node, &t2.node) {
        (Ty::TInt, Ty::TInt) => true,

        (Ty::TBool, Ty::TBool) => true,

        (Ty::TNullRef(rty1), Ty::TNullRef(rty2))
        | (Ty::TRef(rty1), Ty::TNullRef(rty2))
        | (Ty::TRef(rty1), Ty::TRef(rty2))
        | (Ty::TNullRef(rty1), Ty::TRef(rty2)) => subtype_ref(h, rty1, rty2),

        (_, _) => false, // incorrect subtyping
    }
}

// reference subtyping ---------------------------------------------------------------- *)
//  Decides whether H |-ref t1 <: t2
//     - assumes that H contains the declarations of all the possible struct types
//
fn subtype_ref(h: &TypeCtxt, t1: &ast::SRefTy, t2: &ast::SRefTy) -> bool {
    match (&t1.node, &t2.node) {
        (RefTy::RString, RefTy::RString) => true,

        (RefTy::RArray(elt_t1), RefTy::RArray(elt_t2)) => elt_t1.node == elt_t2.node, // check type equalities for elts

        (RefTy::RFun(args1, out1), RefTy::RFun(args2, out2)) => {
            subtype_list(h, args2.as_slice(), args1.as_slice())
                && subtype_ret(h, out1.as_ref(), out2.as_ref())
        }

        (RefTy::RStruct(id1), RefTy::RStruct(id2)) => {
            id1 == id2 || subtype_fields(h, id1.to_string(), id2.to_string())
        }

        (_, _) => false,
    }
}

// return subtyping ---------------------------------------------------------------- *)
//  Decides whether H |-ret t1 <: t2
//     - assumes that H contains the declarations of all the possible struct types
//
fn subtype_ret(h: &TypeCtxt, t1: &ast::SRetTy, t2: &ast::SRetTy) -> bool {
    match (&t1.node, &t2.node) {
        (RetTy::RetVoid, RetTy::RetVoid) => true,

        (RetTy::RetVal(v1), RetTy::RetVal(v2)) => subtype(h, v1.as_ref(), v2.as_ref()),

        _ => false,
    }
}

// helper for subtyping list like arguments in functions
// ex : [a1,a2,a3] and [b1,b2,b3]
fn subtype_list(h: &TypeCtxt, l1: &[ast::STy], l2: &[ast::STy]) -> bool {
    if l1.len() != l2.len() {
        return false;
    }

    l1.iter().zip(l2.iter()).all(|(t1, t2)| subtype(h, t1, t2))
}

// struct n1 <: struct n2 iff
// fields(n2) is a prefix of fields(n1)
//
// ex: s1.a <= s2.a,
fn subtype_fields(_h: &TypeCtxt, _n1: ast::IdTy, _n2: ast::IdTy) -> bool {
    return false;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("not implemented yet");
    Ok(())
}

// WELL-TYPEDNESS RULES (tc type, tc ref, tc ret ) ------------------------------------------

fn typecheck_ty(h: &TypeCtxt, t: &ast::STy) -> TcResult<()> {
    match &t.node {
        Ty::TBool | Ty::TInt => Ok(()),

        Ty::TRef(r) | Ty::TNullRef(r) => typecheck_ref(h, r),
    }
}

fn typecheck_ref(h: &TypeCtxt, r: &ast::SRefTy) -> TcResult<()> {
    match &r.node {
        RefTy::RString => Ok(()),

        RefTy::RStruct(id) => {
            // if struct's id is not findable in our current context -> error
            if h.lookup_struct_option(id.as_str()).is_none() {
                Err(type_error(
                    format!("Unbound struct type for {}", id),
                    r.span.clone(),
                    TypeErrorKind::UnknownIdentifier { name: (id.clone()) },
                ))
            } else {
                Ok(())
            }
        }

        //  elt : Box<STy>
        RefTy::RArray(elt) => typecheck_ty(h, elt.as_ref()), // as_ref : go inside the box

        // args: Vec<STy>, ret: Box<SRetTy>
        RefTy::RFun(args, ret) => {
            for a in args {
                typecheck_ty(h, a)?
            }
            typecheck_ret(h, ret.as_ref())?;
            Ok(())
        }
    }
}
fn typecheck_ret(h: &TypeCtxt, ret: &ast::SRetTy) -> TcResult<()> {
    match &ret.node {
        RetTy::RetVoid => Ok(()),

        RetTy::RetVal(val) => typecheck_ty(h, val.as_ref()),
    }
}

// TYPECHECKING (a.k.a TYPE INFERENCE RULES) -------------------------------------------

fn typecheck_exp(h: &TypeCtxt, e: &ast::Node<ast::SExp>) -> TcResult<ast::STy> {
    let span = e.loc.clone();

    match &e.elt.node {
        Exp::CNull(r) => Ok(mk_sty(Ty::TNullRef(r.clone()), span)),

        Exp::CBool(_) => Ok(mk_sty(Ty::TBool, span)),

        Exp::CInt(_) => Ok(mk_sty(Ty::TInt, span)),

        Exp::CStr(_) => {
            let rstring = common::Spanned::new(span.clone(), RefTy::RString);
            Ok(mk_sty(Ty::TRef(rstring), span))
        }

        Exp::Id(id) => match h.lookup_var_option(id.as_str()) {
            Some(ty) => Ok(mk_sty(ty.clone(), span)),
            None => Err(type_error(
                format!("Unbound identifier {}", id),
                span,
                TypeErrorKind::UnknownIdentifier { name: id.clone() },
            )),
        },

        Exp::CArr(t, l) => {
            // Typecheck the element type
            let sty = mk_sty(t.clone(), span.clone());
            typecheck_ty(h, &sty)?;

            // Typecheck all elements
            let mut types_of = Vec::new();
            for elem in l {
                types_of.push(typecheck_exp(h, elem)?);
            }

            // Check that all elements are subtypes of t
            let sty_check = mk_sty(t.clone(), span.clone());
            for elem_ty in &types_of {
                if !subtype(h, elem_ty, &sty_check) {
                    return Err(type_error(
                        "Mismatched array type",
                        span,
                        TypeErrorKind::Mismatch {
                            expected: format!("{:?}", t),
                            found: format!("{:?}", elem_ty.node),
                        },
                    ));
                }
            }

            let sty_elem = mk_sty(t.clone(), span.clone());
            let rarray = common::Spanned::new(span.clone(), RefTy::RArray(Box::new(sty_elem)));
            Ok(mk_sty(Ty::TRef(rarray), span))
        }

        Exp::NewArr(t, e1) => {
            // Check that t is not a non-nullable reference type
            match t {
                Ty::TBool | Ty::TInt | Ty::TNullRef(_) => {}
                Ty::TRef(_) => {
                    return Err(type_error(
                        "Non-null types cannot be used with default-initialized arrays",
                        span,
                        TypeErrorKind::Mismatch {
                            expected: "nullable or primitive type".to_string(),
                            found: format!("{:?}", t),
                        },
                    ));
                }
            }

            let size_type = typecheck_exp(h, e1)?;
            if size_type.node != Ty::TInt {
                return Err(type_error(
                    "Array size not an int",
                    e1.loc.clone(),
                    TypeErrorKind::Mismatch {
                        expected: "TInt".to_string(),
                        found: format!("{:?}", size_type.node),
                    },
                ));
            }

            let sty_elem = mk_sty(t.clone(), span.clone());
            let rarray = common::Spanned::new(span.clone(), RefTy::RArray(Box::new(sty_elem)));
            Ok(mk_sty(Ty::TRef(rarray), span))
        }

        Exp::Bop(b, l, r) => {
            let ltyp = typecheck_exp(h, l)?;
            let rtyp = typecheck_exp(h, r)?;

            match b {
                BinOp::Eq | BinOp::Neq => {
                    // Check type compatibility (mutual subtyping)
                    if subtype(h, &ltyp, &rtyp) && subtype(h, &rtyp, &ltyp) {
                        Ok(mk_sty(Ty::TBool, span))
                    } else {
                        Err(type_error(
                            "== or != used with non type-compatible arguments",
                            span,
                            TypeErrorKind::Mismatch {
                                expected: format!("{:?}", ltyp.node),
                                found: format!("{:?}", rtyp.node),
                            },
                        ))
                    }
                }
                _ => {
                    let (bl, br, bres) = typ_of_binop(b);
                    if ltyp.node != bl {
                        return Err(type_error(
                            "Incorrect type in binary expression",
                            l.loc.clone(),
                            TypeErrorKind::Mismatch {
                                expected: format!("{:?}", bl),
                                found: format!("{:?}", ltyp.node),
                            },
                        ));
                    }
                    if rtyp.node != br {
                        return Err(type_error(
                            "Incorrect type in binary expression",
                            r.loc.clone(),
                            TypeErrorKind::Mismatch {
                                expected: format!("{:?}", br),
                                found: format!("{:?}", rtyp.node),
                            },
                        ));
                    }
                    Ok(mk_sty(bres, span))
                }
            }
        }

        Exp::Uop(u, e_inner) => {
            let t = typecheck_exp(h, e_inner)?;
            let (us, ures) = typ_of_unop(u);
            if t.node == us {
                Ok(mk_sty(ures, span))
            } else {
                Err(type_error(
                    "Incorrect type for unary operator",
                    e_inner.loc.clone(),
                    TypeErrorKind::Mismatch {
                        expected: format!("{:?}", us),
                        found: format!("{:?}", t.node),
                    },
                ))
            }
        }

        Exp::Index(e1, e2) => {
            let arr_t = typecheck_exp(h, e1)?;
            let ind_t = typecheck_exp(h, e2)?;

            if ind_t.node != Ty::TInt {
                return Err(type_error(
                    "Index of array index operator not an int",
                    e2.loc.clone(),
                    TypeErrorKind::Mismatch {
                        expected: "TInt".to_string(),
                        found: format!("{:?}", ind_t.node),
                    },
                ));
            }

            match &arr_t.node {
                Ty::TRef(r) => match &r.node {
                    RefTy::RArray(t) => Ok((**t).clone()),
                    _ => Err(type_error(
                        format!("Tried to compute index into type {:?}", arr_t.node),
                        e1.loc.clone(),
                        TypeErrorKind::Mismatch {
                            expected: "array type".to_string(),
                            found: format!("{:?}", arr_t.node),
                        },
                    )),
                },
                _ => Err(type_error(
                    format!("Tried to compute index into type {:?}", arr_t.node),
                    e1.loc.clone(),
                    TypeErrorKind::Mismatch {
                        expected: "array type".to_string(),
                        found: format!("{:?}", arr_t.node),
                    },
                )),
            }
        }

        Exp::Call(f, args) => {
            // typecheck args
            let mut argtyps = Vec::new();
            for arg in args {
                argtyps.push(typecheck_exp(h, arg)?);
            }

            // typecheck f
            let ftyp = typecheck_exp(h, f)?;

            match &ftyp.node {
                Ty::TRef(r) => {
                    match &r.node {
                        RefTy::RFun(param_types, ret_ty) => {
                            // Check correct number of arguments
                            if param_types.len() != argtyps.len() {
                                return Err(type_error(
                                    "Incorrect number of arguments",
                                    span,
                                    TypeErrorKind::Mismatch {
                                        expected: format!("{} arguments", param_types.len()),
                                        found: format!("{} arguments", argtyps.len()),
                                    },
                                ));
                            }

                            // Check argument types
                            for (i, (arg, param)) in
                                argtyps.iter().zip(param_types.iter()).enumerate()
                            {
                                if !subtype(h, arg, param) {
                                    return Err(type_error(
                                        format!("Incorrect type of argument {}", i),
                                        span.clone(),
                                        TypeErrorKind::Mismatch {
                                            expected: format!("{:?}", param.node),
                                            found: format!("{:?}", arg.node),
                                        },
                                    ));
                                }
                            }

                            // Return the return type
                            match &ret_ty.node {
                                RetTy::RetVal(r) => Ok((**r).clone()),
                                RetTy::RetVoid => Err(type_error(
                                    "Cannot use void function in expression context",
                                    span,
                                    TypeErrorKind::Mismatch {
                                        expected: "non-void return type".to_string(),
                                        found: "void".to_string(),
                                    },
                                )),
                            }
                        }
                        _ => Err(type_error(
                            "Need function argument for function call",
                            f.loc.clone(),
                            TypeErrorKind::NotCallable {
                                ty: format!("{:?}", ftyp.node),
                            },
                        )),
                    }
                }
                _ => Err(type_error(
                    "Need function argument for function call",
                    f.loc.clone(),
                    TypeErrorKind::NotCallable {
                        ty: format!("{:?}", ftyp.node),
                    },
                )),
            }
        }
    }
}
