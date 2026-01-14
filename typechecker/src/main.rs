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
use ast::Stmt;
use ast::Ty;

use ::common::TypeErrorKind;
use typechecker::*;

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
    eprintln!("Typechecker implementation complete!");
    eprintln!("Use typecheck_prog() to typecheck a program.");
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

// Typecheck an Expression
//
//   This function should implement the statment typechecking rules from oat.pdf.
//
//   Inputs:
//    - h: the type context library
//    - e: node containing Spanned Expression
//
//   Returns:
//    - the (most precise) type for the expression,
//       if it is type correct according to the inference rules.
//
//    - context gets updated implicitly
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

// Typecheck a statement
//
//   This function should implement the statment typechecking rules from oat.pdf.
//
//    Inputs:
//     - h: the type context
//     - s: the statement node
//     - to_ret: the desired return type (from the function declaration)
//
//    Returns:
//      - the new type context (which includes newly declared variables in scope
//        after this statement)

//      - A boolean indicating the return behavior of a statement:
//         false:  might not return
//         true: definitely returns

//         in the branching statements, the return behavior of the branching
//         statement is the conjunction of the return behavior of the two
//         branches: both both branches must definitely return in order for
//         the whole statement to definitely return.

//         Intuitively: if one of the two branches of a conditional does not
//         contain a return statement, then the entire conditional statement might
//         not return.

//         looping constructs never definitely return (While, For)
//
// Example statements:
//   - Assn: x = 5;
//   - Decl: var x = 5;
//   - Ret: return; or return x;
//   - SCall: foo();
//   - If: if (x) { ... } else { ... }
//   - For: for (var i = 0; i < 10; i = i + 1;) { ... }
//   - While: while (x) { ... }
//
// Returns: Ok(bool) where bool indicates if the statement definitely returns
//          true = definitely returns, false = might not return
fn typecheck_stmt(
    h: &mut TypeCtxt,
    s: &ast::Node<ast::SStmt>,
    to_ret: &ast::SRetTy,
) -> TcResult<bool> {
    match &s.elt.node {
        // Assn: x = 5;
        Stmt::Assn(e1, e2) => {
            // Special check: prevent assignment to global functions
            //   ex: foo = 5; where foo() is a global function -> ERROR...
            //   else everything is OK!
            if let Exp::Id(x) = &e1.elt.node {
                // If x is NOT a local variable
                if h.lookup_local_option(x.as_str()).is_none() {
                    // Check if x is a global function
                    if let Some(Ty::TRef(r)) = h.lookup_global_option(x.as_str()) {
                        if matches!(&r.node, RefTy::RFun(..)) {
                            return Err(type_error(
                                format!("cannot assign to global function {}", x),
                                s.loc.clone(),
                                TypeErrorKind::Mismatch {
                                    expected: "assignable lvalue".to_string(),
                                    found: "function".to_string(),
                                },
                            ));
                        }
                    }
                }
            }

            // Typecheck both sides and ensure types match
            let assn_to = typecheck_exp(h, e1)?; // STY
            let assn_from = typecheck_exp(h, e2)?; // STy
            if subtype(h, &assn_from, &assn_to) {
                Ok(false) // Assignment doesn't definitely return
            } else {
                Err(type_error(
                    "Mismatched types in assignment",
                    s.loc.clone(),
                    TypeErrorKind::Mismatch {
                        expected: format!("{:?}", assn_to.node),
                        found: format!("{:?}", assn_from.node),
                    },
                ))
            }
        }

        // Decl: int x = 5; where id is x and exp is
        Stmt::Decl(vdecl) => {
            // Check if variable is already declared in the current scope
            // (not parent scopes - we allow shadowing)
            // ex. int x = 2;
            //     string x = "hey";
            if h.is_declared_in_current_scope(&vdecl.vd_id) {
                return Err(type_error(
                    "Cannot redeclare variable",
                    s.loc.clone(),
                    TypeErrorKind::UnknownIdentifier {
                        name: vdecl.vd_id.clone(),
                    },
                ));
            }
            // VDecl can be with or without initializer:
            match &vdecl.vd_node {
                // var x; (uninitialized - should we allow this?)
                None => {
                    // For now, this is an error in our type system
                    // because we need to know the type
                    return Err(type_error(
                        "Variable declaration without initializer not supported",
                        s.loc.clone(),
                        TypeErrorKind::Mismatch {
                            expected: "initializer expression".to_string(),
                            found: "none".to_string(),
                        },
                    ));
                }

                // var x = e; (with initializer)
                Some(exp_node) => {
                    let exp_type = typecheck_exp(h, exp_node)?;
                    h.add_local(vdecl.vd_id.clone(), exp_type.node);
                    Ok(false) // Declaration doesn't definitely return
                }
            }
        }

        // Return Statement:
        //      ex : return; or return x + 5;
        Stmt::Ret(r) => match (r, &to_ret.node) {
            // return; in VOID -> ok!
            (None, RetTy::RetVoid) => Ok(true),

            // return e; in Non-VOID -> ok!
            (Some(r_exp), RetTy::RetVal(expected_ty)) => {
                let t = typecheck_exp(h, r_exp)?;
                if subtype(h, &t, expected_ty.as_ref()) {
                    Ok(true) // Return statement definitely returns
                } else {
                    Err(type_error(
                        "Returned incorrect type",
                        s.loc.clone(),
                        TypeErrorKind::Mismatch {
                            expected: format!("{:?}", expected_ty.node),
                            found: format!("{:?}", t.node),
                        },
                    ))
                }
            }

            // return; but NON-VOID -> (ERROR)
            (None, RetTy::RetVal(_)) => Err(type_error(
                "Returned void in non-void function",
                s.loc.clone(),
                TypeErrorKind::Mismatch {
                    expected: "non-void value".to_string(),
                    found: "void".to_string(),
                },
            )),

            // return e; but VOID -> (ERROR)
            (Some(_), RetTy::RetVoid) => Err(type_error(
                "Returned non-void in void function",
                s.loc.clone(),
                TypeErrorKind::Mismatch {
                    expected: "void".to_string(),
                    found: "non-void value".to_string(),
                },
            )),
        },

        // foo(); (statement call - function must return void)
        // Example: print_int(42);
        Stmt::SCall(f, args) => {
            let mut argtyps = Vec::new();
            for arg in args {
                argtyps.push(typecheck_exp(h, arg)?);
            }

            let ftyp = typecheck_exp(h, f)?;

            match &ftyp.node {
                Ty::TRef(r) | Ty::TNullRef(r) => {
                    match &r.node {
                        RefTy::RFun(param_types, ret_ty) => {
                            // Check that function returns void
                            if !matches!(&ret_ty.node, RetTy::RetVoid) {
                                return Err(type_error(
                                    "Statement call requires void function",
                                    s.loc.clone(),
                                    TypeErrorKind::Mismatch {
                                        expected: "void return type".to_string(),
                                        found: format!("{:?}", ret_ty.node),
                                    },
                                ));
                            }

                            // Check correct number of arguments
                            if param_types.len() != argtyps.len() {
                                return Err(type_error(
                                    "Incorrect number of arguments",
                                    s.loc.clone(),
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
                                        s.loc.clone(),
                                        TypeErrorKind::Mismatch {
                                            expected: format!("{:?}", param.node),
                                            found: format!("{:?}", arg.node),
                                        },
                                    ));
                                }
                            }

                            Ok(false) // Statement call doesn't definitely return
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

        // if (e) { ... } else { ... } (conditional statement)
        // Example: if (x > 0) { return 1; } else { return -1; }
        Stmt::If(guard, then_block, else_block) => {
            let guard_type = typecheck_exp(h, guard)?;

            if guard_type.node != Ty::TBool {
                return Err(type_error(
                    "Incorrect type for guard",
                    guard.loc.clone(),
                    TypeErrorKind::Mismatch {
                        expected: "TBool".to_string(),
                        found: format!("{:?}", guard_type.node),
                    },
                ));
            }

            let lft_ret = typecheck_block(h, then_block, to_ret)?;
            let rgt_ret = typecheck_block(h, else_block, to_ret)?;

            // Both branches must return for the if to definitely return
            Ok(lft_ret && rgt_ret)
        }

        // for (var x = e1, var y = e2, ...; guard; increment) { body }
        // Example: for (var i = 0; i < 10; i = i + 1;) { print_int(i); }
        Stmt::For(vdecls, guard, increment, body) => {
            // Create new scope for the for loop
            h.push_scope();

            // Add all declared variables to the new scope
            for vdecl in vdecls {
                // Check the initializer if present
                match &vdecl.vd_node {
                    None => {
                        h.pop_scope();
                        return Err(type_error(
                            "For loop variable declaration requires initializer",
                            s.loc.clone(),
                            TypeErrorKind::Mismatch {
                                expected: "initializer".to_string(),
                                found: "none".to_string(),
                            },
                        ));
                    }
                    Some(exp_node) => {
                        let t = typecheck_exp(h, exp_node)?;
                        h.add_local(vdecl.vd_id.clone(), t.node);
                    }
                }
            }

            // Check guard expression if present
            if let Some(guard_exp) = guard {
                let guard_type = typecheck_exp(h, guard_exp)?;
                if guard_type.node != Ty::TBool {
                    h.pop_scope();
                    return Err(type_error(
                        "Incorrect type for guard",
                        guard_exp.loc.clone(),
                        TypeErrorKind::Mismatch {
                            expected: "TBool".to_string(),
                            found: format!("{:?}", guard_type.node),
                        },
                    ));
                }
            }

            // Check increment statement if present
            if let Some(inc_stmt) = increment {
                let rt = typecheck_stmt(h, inc_stmt, to_ret)?;
                if rt {
                    h.pop_scope();
                    return Err(type_error(
                        "Cannot return in for loop increment",
                        inc_stmt.loc.clone(),
                        TypeErrorKind::Mismatch {
                            expected: "non-returning statement".to_string(),
                            found: "return statement".to_string(),
                        },
                    ));
                }
            }

            // Typecheck the body
            let _ = typecheck_block(h, body, to_ret)?;

            // Pop the for loop scope
            h.pop_scope();

            // For loops never definitely return
            Ok(false)
        }

        // while (e) { body } (while loop)
        // Example: while (x > 0) { x = x - 1; }
        Stmt::While(guard, body) => {
            let guard_type = typecheck_exp(h, guard)?;

            if guard_type.node != Ty::TBool {
                return Err(type_error(
                    "Incorrect type for guard",
                    guard.loc.clone(),
                    TypeErrorKind::Mismatch {
                        expected: "TBool".to_string(),
                        found: format!("{:?}", guard_type.node),
                    },
                ));
            }

            let _ = typecheck_block(h, body, to_ret)?;

            // While loops never definitely return
            Ok(false)
        }
    }
}

// Typecheck a block of statements
// Returns true if the block definitely returns, false otherwise
fn typecheck_block(
    h: &mut TypeCtxt,
    block: &[ast::Node<ast::SStmt>],
    to_ret: &ast::SRetTy,
) -> TcResult<bool> {
    // Push a new scope for the block
    h.push_scope();

    let mut definitely_returns = false;

    for stmt in block {
        let returns = typecheck_stmt(h, stmt, to_ret)?;
        if returns {
            definitely_returns = true;
            // for now, we will ignore unreachable code,
            // once every block "returns" wether they are reachable or not,
            // then we pop the scope and output a OK(true), else Ok(false)
            //
        }
    }

    // Pop the block scope
    h.pop_scope();

    Ok(definitely_returns)
}

// PROGRAM-LEVEL TYPECHECKING -------------------------------------------

// Typecheck a global variable declaration ---------------------------
//      ex: int global_x = 42;
fn typecheck_gvdecl(h: &mut TypeCtxt, gvdecl: &ast::Node<ast::GDecl>) -> TcResult<()> {
    let name = &gvdecl.elt.name;
    let init = &gvdecl.elt.init;

    // Typecheck the initializer expression
    let init_type = typecheck_exp(h, init)?;

    // Add to global context
    h.add_global(name.clone(), init_type.node);

    Ok(())
}

// Typecheck a struct/type declaration -------------------------------
//      ex : struct Point { int x; int y; }
fn typecheck_tdecl(h: &mut TypeCtxt, tdecl: &ast::Node<ast::TDecl>) -> TcResult<()> {
    let struct_name = &tdecl.elt.td_id;
    let fields = &tdecl.elt.td_node;

    // check duplicate field names
    let mut field_names = std::collections::HashSet::new();
    for field in fields {
        if !field_names.insert(&field.field_name) {
            return Err(type_error(
                format!(
                    "Duplicate field name '{}' in struct '{}'",
                    field.field_name, struct_name
                ),
                tdecl.loc.clone(),
                TypeErrorKind::UnknownIdentifier {
                    name: field.field_name.clone(),
                },
            ));
        }

        // check field ty is well formed
        let field_ty = mk_sty(field.field_type.clone(), tdecl.loc.clone());
        typecheck_ty(h, &field_ty)?;
    }

    h.add_struct(struct_name.clone(), fields.clone());

    Ok(())
}

// Typecheck a function declaration ------------------------------------
//     ex: int foo(int x, bool y) { return x; }
fn typecheck_fdecl(h: &mut TypeCtxt, fdecl: &ast::Node<ast::FDecl>) -> TcResult<()> {
    let fname = &fdecl.elt.fname;
    let args = &fdecl.elt.args;
    let ret_ty = &fdecl.elt.fret_ty;
    let body = &fdecl.elt.body;

    // Build function type for context
    let arg_types: Vec<ast::STy> = args.iter().map(|arg| arg.ty.clone()).collect();
    let ret_ty_spanned = mk_sretty(ret_ty.clone(), fdecl.loc.clone());

    // Create function type: TRef(RFun(args, ret))
    let fun_ref_ty = mk_srefty(
        RefTy::RFun(arg_types.clone(), Box::new(ret_ty_spanned.clone())),
        fdecl.loc.clone(),
    );
    let fun_ty = mk_sty(Ty::TRef(fun_ref_ty), fdecl.loc.clone());

    h.add_global(fname.clone(), fun_ty.node);

    // new scope for function body
    h.push_scope();

    for arg in args {
        h.add_local(arg.id.clone(), arg.ty.node.clone());
    }

    let body_returns = typecheck_block(h, body, &ret_ty_spanned)?;

    // Check that non-void functions return on all paths
    if !matches!(ret_ty, RetTy::RetVoid) && !body_returns {
        h.pop_scope();
        return Err(type_error(
            format!("Function '{}' does not return on all paths", fname),
            fdecl.loc.clone(),
            TypeErrorKind::Mismatch {
                expected: "return statement".to_string(),
                found: "end of function".to_string(),
            },
        ));
    }

    h.pop_scope();

    Ok(())
}

// Typecheck an entire program -------------------------------
//      Remember: a Prog is a Vec of Decl
//      and Decl: global variables, functions, and structs
pub fn typecheck_prog(prog: &ast::Prog) -> TcResult<()> {
    let mut h = TypeCtxt::empty();

    // Two-pass approach:
    // Pass 1: Add all struct declarations first (so functions can reference them)
    for decl in prog {
        if let ast::Decl::GTDecl(tdecl) = decl {
            typecheck_tdecl(&mut h, tdecl)?;
        }
    }

    // Pass 2: Add all function signatures and global variables, then typecheck bodies
    for decl in prog {
        match decl {
            ast::Decl::GVDecl(gvdecl) => {
                typecheck_gvdecl(&mut h, gvdecl)?;
            }
            ast::Decl::GFDecl(fdecl) => {
                typecheck_fdecl(&mut h, fdecl)?;
            }
            ast::Decl::GTDecl(_) => {
                // Already processed in pass 1
            }
        }
    }

    Ok(())
}
