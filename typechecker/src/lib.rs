#! Typechecker helpers & Context-related definitions

use std::collections::HashMap;

use ast::*;
use common::Span;
use common::TypeError;
use common::TypeErrorKind;

// Context Defintions
pub type FunTy = Vec<(Ty, RetTy)>;

type LocalCtxt = Vec<HashMap<IdTy, Ty>>;

type GlobalCtxt = HashMap<IdTy, Ty>;
type FunCtxt = HashMap<IdTy, FunTy>;
type StructCtxt = HashMap<IdTy, Vec<Field>>;

#[derive(Debug, Clone)]
pub struct TypeCtxt {
    locals: LocalCtxt,
    globals: GlobalCtxt,
    functions: FunCtxt,
    structs: StructCtxt,
}

// pub fn lookup_field(struc_name: IdTy, f_name: IdTy, tc: TypeCtxt) {
//     match (lookup_field_option (struc_name, f_name, tc)) {
//         None =>
//     }
// }

// Typechecker utilitites
pub type TcResult<T> = Result<T, TypeError>;

pub fn type_error(msg: impl Into<String>, span: Span, kind: TypeErrorKind) -> TypeError {
    TypeError {
        msg: msg.into(),
        span,
        kind,
    }
}
