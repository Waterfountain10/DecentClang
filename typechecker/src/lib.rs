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
