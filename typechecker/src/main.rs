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
use common::Span;
use common::TypeError;
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
fn subtype(h: &TypeCtxt, t1: &ast::Ty, t2: &ast::Ty) -> bool {
    match (t1, t2) {
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
fn subtype_ref(h: &TypeCtxt, t1: &ast::RefTy, t2: &ast::RefTy) -> bool {
    match (t1, t2) {
        (RefTy::RString, RefTy::RString) => true,

        (RefTy::RArray(elt_t1), RefTy::RArray(elt_t2)) => elt_t1.as_ref() == elt_t2.as_ref(), // check type equalities for elts

        (RefTy::RFun(args1, out1), RefTy::RFun(args2, out2)) => {
            subtype_list(h, args2.as_slice(), args1.as_slice()) && subtype_ret(h, out1, out2)
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
fn subtype_ret(h: &TypeCtxt, t1: &ast::RetTy, t2: &ast::RetTy) -> bool {
    match (t1, t2) {
        (RetTy::RetVoid, RetTy::RetVoid) => true,

        (RetTy::RetVal(v1), RetTy::RetVal(v2)) => subtype(h, v1.as_ref(), v2.as_ref()),

        _ => false,
    }
}

// helper for subtyping list like arguments in functions
// ex : [a1,a2,a3] and [b1,b2,b3]
fn subtype_list(h: &TypeCtxt, l1: &[ast::Ty], l2: &[ast::Ty]) -> bool {
    if l1.len() != l2.len() {
        return false;
    }

    l1.iter().zip(l2.iter()).all(|(t1, t2)| subtype(h, t1, t2))
}

// struct n1 <: struct n2 iff
// fields(n2) is a prefix of fields(n1)
//
// ex: s1.a <= s2.a,
fn subtype_fields(h: &TypeCtxt, n1: ast::IdTy, n2: ast::IdTy) -> bool {
    return false;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("not implemented yet");
    Ok(())
}

// TYPECHECKING (a.k.a TYPE INFERENCE RULES) -------------------------------------------

// WELL TYPEDNESS RULES (tc type, tc ref, tc ret )
//
// functions that check that types are well formed according
// to the H |- t and related inference rules
// used for:
// CArr : int[] a; => tc the TYPE int
// NewArrInit : int[] a = [1, 2, 3]; => tc the TYPE int again
//
// not NewArr : int[] a = new int[10]; => tc also, but different because init. arrays dont support TRef types
fn typecheck_ty(h: TypeCtxt, t: ast::Ty, null: bool) -> TcResult<ast::Ty> {
    match t {
        Ty::TBool => Ok(Ty::TBool),

        Ty::TInt => Ok(Ty::TInt),

        Ty::TRef(r) | Ty::TNullRef(r) => match typecheck_ref(h, r, &null) {
            Ok(rt) => {
                if null == true {
                    Ok(Ty::TNullRef(rt))
                } else {
                    Ok(Ty::TRef(rt))
                }
            }
            Err(te) => Err(te), // typecheck_ref handles the Err format already
        },
    }
}

fn typecheck_ref(h: TypeCtxt, r: ast::RefTy, mut null: &bool) -> TcResult<ast::RefTy> {
    match r {
        RefTy::RString => Ok(RefTy::RString),

        RefTy::RStruct(id) => {
            if h.lookup_struct_option(id.as_str()).is_none() {
                Err(type_error(
                    format!("Unbound struct type for {}", id),
                    Span::dummy(),
                    TypeErrorKind::UnknownIdentifier { name: (id) },
                ))
            } else {
                Ok(RefTy::RStruct(id))
            }
        }
        RefTy::RArray(b) => match typecheck_ty(h, b.as_ref().into(), null*) {
            Ok(t) => Ok(RefTy::RArray(Box::new(t))),
            Err(te) => Err(te)
            }
        }


        RefTy::RFun(_, _) => (),
    }
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
