#! Typechecker helpers & Context-related definitions

use ast;
use common::Span;
use common::TypeError;
use common::TypeErrorKind;

// Context Defintions
pub type FunTy = Vec<(ast::Ty, ast::RetTy)>;

type LocalCtxt = Vec<(ast::IdTy, ast::Ty)>;
type GlobalCtxt = Vec<(ast::IdTy, ast::Ty)>;
type FunCtxt = Vec<(ast::IdTy, FunTy)>;
type StructCtxt = Vec<(ast::IdTy, Vec<ast::Field>)>;

pub struct TypeCtxt {
    locals: LocalCtxt,
    globals: GlobalCtxt,
    functions: FunCtxt,
    structs: StructCtxt,
}

// Typechecker utilitites
pub type TcResult<T> = Result<T, TypeError>;

pub fn type_error(msg: impl Into<String>, span: Span, kind: TypeErrorKind) -> TypeError {
    TypeError {
        msg: msg.into(),
        span,
        kind,
    }
}
