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
use ast::Ty::*;
use ast::*;
use std::any::Any;
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

        (RArray(elt_t1), RArray(elt_t2)) => elt_t1 == elt_t2, // check type equalities for elts

        // RFun : (Vec<Ty>, RetTy)
        (RFun(args1, out1), RFun(args2, out2)) => {
            subtype_list(h, args1.as_slice(), args2.as_slice()) && subtype_ret(h, out1, out2)
        }

        (RStruct(id1), RStruct(id2)) => {
            eprintln!("not implemented yet!");
            false
        }

        (_, _) => false,
    }
}

fn subtype_ret(h: &TypeCtxt, t1: &RetTy, t2: &RetTy) -> bool {
    print!("not implemented yet");
    return false;
}

// helper for subtyping list like arguments in functions
// ex : [a1,a2,a3] and [b1,b2,b3]
fn subtype_list(h: &TypeCtxt, l1: &[Ty], l2: &[Ty]) -> bool {
    if l1.len() != l2.len() {
        return false;
    }

    l1.iter().zip(l2.iter()).all(|(t1, t2)| subtype(&h, t1, t2))
}

// pub fn int64_of_sbytes(bs: &Vec<SByte>) -> i64 {
//     bs.iter().rev().fold(0i64, |acc, b| match b {
//         SByte::Byte(c) => (acc << 8) | (*c as u8 as i64), // shifted acc becomes last 2 digits, and c is the highest ones
//         _ => 0i64,                                        // start with acc = 0
// }
//     })

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("not implemented yet");
    Ok(())
}
