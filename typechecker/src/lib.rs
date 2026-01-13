#! Typechecker helpers & Context-related definitions

use std::any::Any;
use std::collections::HashMap;

use ast::*;
use common::Span;
use common::TypeError;
use common::TypeErrorKind;

// Type Context (TypeCtxt) Defintions --------------------------------------------------------
pub type FunTy = Vec<(Ty, RetTy)>;

type LocalCtxt = Vec<HashMap<IdTy, Ty>>; // locals are scoped, so we keep track in a vector
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

impl TypeCtxt {
    pub fn empty() -> Self {
        Self {
            locals: vec![HashMap::new()], // start with one scope
            globals: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    // ----- locals -----
    pub fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        // you might want to prevent popping the last scope
        self.locals.pop();
        if self.locals.is_empty() {
            self.locals.push(HashMap::new());
        }
    }

    pub fn add_local(&mut self, id: IdTy, ty: Ty) {
        // last scope in the stack via last_mut
        self.locals.last_mut().unwrap().insert(id, ty);
    }

    pub fn lookup_local_option(&self, id: &str) -> Option<&Ty> {
        for scope in self.locals.iter().rev() {
            if let Some(t) = scope.get(id) {
                return Some(t);
            }
        }
        None
    }

    // ----- globals -----
    pub fn add_global(&mut self, id: IdTy, ty: Ty) {
        self.globals.insert(id, ty);
    }

    pub fn lookup_global_option(&self, id: &str) -> Option<&Ty> {
        self.globals.get(id)
    }

    // general lookup for : local? global
    pub fn lookup_var_option(&self, id: &str) -> Option<&Ty> {
        self.lookup_local_option(id)
            .or_else(|| self.lookup_global_option(id))
    }

    // ----- functions -----
    pub fn add_function(&mut self, id: IdTy, fty: FunTy) {
        self.functions.insert(id, fty);
    }

    pub fn lookup_function_option(&self, id: &str) -> Option<&FunTy> {
        self.functions.get(id)
    }

    // ----- structs -----
    pub fn add_struct(&mut self, id: IdTy, fields: Vec<Field>) {
        self.structs.insert(id, fields);
    }

    pub fn lookup_struct_option(&self, id: &str) -> Option<&[Field]> {
        self.structs.get(id).map(|v| v.as_slice())
    }

    pub fn lookup_field_option(&self, st_name: &str, f_name: &str) -> Option<&Ty> {
        let fields = self.lookup_struct_option(st_name)?;
        fields
            .iter()
            .find(|f| f.field_name == f_name)
            .map(|f| &f.field_type)
    }
}

// Typechecker utilitites -------------------------------------------------------------------
pub type TcResult<T> = Result<T, TypeError>;

pub fn type_error(msg: impl Into<String>, span: Span, kind: TypeErrorKind) -> TypeError {
    TypeError {
        msg: msg.into(),
        span,
        kind,
    }
}

// typecheck rule helpers

// Helper function to create a spanned type from a Ty
pub fn mk_sty(ty: Ty, span: common::Span) -> ast::STy {
    common::Spanned::new(span, ty)
}

// Helper function to create a spanned reference type
pub fn mk_srefty(rty: RefTy, span: common::Span) -> ast::SRefTy {
    common::Spanned::new(span, rty)
}

// Helper function to create a spanned return type
pub fn mk_sretty(ret: RetTy, span: common::Span) -> ast::SRetTy {
    common::Spanned::new(span, ret)
}

// Helper constructors for common spanned types
pub fn mk_t_int(span: common::Span) -> ast::STy {
    mk_sty(Ty::TInt, span)
}

pub fn mk_t_bool(span: common::Span) -> ast::STy {
    mk_sty(Ty::TBool, span)
}

pub fn mk_t_ref(rty: ast::SRefTy, span: common::Span) -> ast::STy {
    mk_sty(Ty::TRef(rty), span)
}

pub fn mk_t_null_ref(rty: ast::SRefTy, span: common::Span) -> ast::STy {
    mk_sty(Ty::TNullRef(rty), span)
}

pub fn mk_r_string(span: common::Span) -> ast::SRefTy {
    mk_srefty(RefTy::RString, span)
}

pub fn mk_r_array(elt: Box<ast::STy>, span: common::Span) -> ast::SRefTy {
    mk_srefty(RefTy::RArray(elt), span)
}

// Helper function to get the type of a binary operator
pub fn typ_of_binop(b: &ast::BinOp) -> (Ty, Ty, Ty) {
    use ast::BinOp::*;
    match b {
        Add | Sub | Mul | IAnd | IOr | Shl | Shr | Sar => (Ty::TInt, Ty::TInt, Ty::TInt),
        Eq | Neq | Lt | Lte | Gt | Gte => (Ty::TInt, Ty::TInt, Ty::TBool),
        And | Or => (Ty::TBool, Ty::TBool, Ty::TBool),
    }
}

// Helper function to get the type of a unary operator
pub fn typ_of_unop(u: &ast::UnOp) -> (Ty, Ty) {
    use ast::UnOp::*;
    match u {
        Neg | BitNot => (Ty::TInt, Ty::TInt),
        LogNot => (Ty::TBool, Ty::TBool),
    }
}
