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

use ast::RefTy::*;
use ast::RetTy::*;
use ast::Ty::*;
use ast::*;
use std::any::Any;
use std::boxed;
use std::cell::Ref;
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
fn subtype(h: &TypeCtxt, t1: &Ty, t2: &Ty) -> bool {
    match (t1, t2) {
        (TInt, TInt) => true,

        (TBool, TBool) => true,

        (TNullRef(rty1), TNullRef(rty2))
        | (TRef(rty1), TNullRef(rty2))
        | (TRef(rty1), TRef(rty2))
        | (TNullRef(rty1), TRef(rty2)) => subtype_ref(h, rty1, rty2),

        (_, _) => false, // incorrect subtyping
    }
}

// reference subtyping ---------------------------------------------------------------- *)
//  Decides whether H |-ref t1 <: t2
//     - assumes that H contains the declarations of all the possible struct types
//
fn subtype_ref(h: &TypeCtxt, t1: &RefTy, t2: &RefTy) -> bool {
    match (t1, t2) {
        (RString, RString) => true,

        (RArray(elt_t1), RArray(elt_t2)) => elt_t1.as_ref() == elt_t2.as_ref(), // check type equalities for elts

        // RFun : (Vec<Ty>, RetTy)
        (RFun(args1, out1), RFun(args2, out2)) => {
            subtype_list(h, args2.as_slice(), args1.as_slice()) && subtype_ret(h, out1, out2)
        }

        (RStruct(id1), RStruct(id2)) => {
            id1 == id2 || subtype_fields(h, id1.to_string(), id2.to_string())
        }

        (_, _) => false,
    }
}

// return subtyping ---------------------------------------------------------------- *)
//  Decides whether H |-ret t1 <: t2
//     - assumes that H contains the declarations of all the possible struct types
//
fn subtype_ret(h: &TypeCtxt, t1: &RetTy, t2: &RetTy) -> bool {
    match (t1, t2) {
        (RetVoid, RetVoid) => true,

        (RetVal(v1), RetVal(v2)) => subtype(h, v1.as_ref(), v2.as_ref()),

        _ => false,
    }
}

// helper for subtyping list like arguments in functions
// ex : [a1,a2,a3] and [b1,b2,b3]
fn subtype_list(h: &TypeCtxt, l1: &[Ty], l2: &[Ty]) -> bool {
    if l1.len() != l2.len() {
        return false;
    }

    l1.iter().zip(l2.iter()).all(|(t1, t2)| subtype(h, t1, t2))
}

// fields n1 are a subtype of n2 if n2 is a prefix of n1
fn subtype_fields(h: &TypeCtxt, n1: IdTy, n2: IdTy) -> bool {
    print!("not implemented yet.");
    return false;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("not implemented yet");
    Ok(())
}

// TYPECHECKING (a.k.a TYPE INFERENCE RULES)

// used for:
// CArr,
// NewArrInit : int[] myArray = [1,2,3]
fn typecheck_ty(h: TypeCtxt, t: Ty) -> TcResult<Ty> {
    match (t) {}
}

fn typecheck_exp(env: &Env, e: &Exp) -> TcResult<Ty> {
    match e {
        Exp::Int(_, _) => Ok(Ty::TInt),

        Exp::Bool(_, _) => Ok(Ty::TBool),

        Exp::Var(span, x) => env
            .lookup(x)
            .cloned()
            .ok_or_else(|| type_error(*span, format!("unbound variable `{}`", x))),

        Exp::Add(span, e1, e2) => {
            let t1 = typecheck_exp(env, e1)?; // ← HERE
            let t2 = typecheck_exp(env, e2)?; // ← HERE

            if t1 == Ty::TInt && t2 == Ty::TInt {
                Ok(Ty::TInt)
            } else {
                Err(type_error(*span, "both operands of + must be int"))
            }
        }
    }
}
