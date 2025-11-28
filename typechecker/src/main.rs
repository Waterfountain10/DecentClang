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
fn subtype(h: TypeCtxt, t1: Ty, t2: Ty) -> bool {
    match (t1, t2) {
        (TInt, TInt) => true,

        (TBool, TBool) => true,

        (TNullRef(rty1), TNullRef(rty2))
        | (TRef(rty1), TNullRef(rty2)) // ref <= nullref is one-sided
        | (TRef(rty1), TRef(rty2)) => subtype_ref(h, rty1, rty2),

        // TODO:
        // ENSURE MUTUAL RECURSION BEST PRACTICES
        // between subtype <-> ref_subtype
        //

        (_, _) => false, // incorrect subtyping
    }
}

// reference subtyping ---------------------------------------------------------------- *)
//  Decides whether H |-ref t1 <: t2
//     - assumes that H contains the declarations of all the possible struct types
//
fn subtype_ref(h: TypeCtxt, t1: RefTy, t2: RefTy) -> bool {
    match (t1, t2) {
        (RString, RString) => true,

        (RArray(elt_t1), RArray(elt_t2)) => elt_t1.type_id().eq(&elt_t2.type_id()), // check type equalities for elts

        (RStruct(id1), RStruct(id2)) => {
            eprintln!("not implemented yet!");
            false
        }
        // Vec<Ty>, RetTy
        (RFun(args1, out1), RFun(args2, out2)) => {
            eprintln!("not implemented yet!");
            false
        }

        (_, _) => false,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("not implemented yet");
    Ok(())
}
