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
            subtype_list(h, args2.as_slice(), args1.as_slice()) && subtype_ret(h, out1.as_ref(), out2.as_ref())
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

// WELL TYPEDNESS RULES (tc type, tc ref, tc ret ) ------------------------------------------
//
// functions that check that types are well formed according
// to the H |- t and related inference rules

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
