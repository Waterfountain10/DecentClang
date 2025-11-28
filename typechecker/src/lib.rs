#! Typechecker helpers & Context-related definitions

use ast;

// Context Defintions
type FunTy = Vec<(ast::Ty, ast::RetTy)>;

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
