#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    // used for persistent expressions
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    // used as filler for types or abstract entities (example RefTy)
    pub fn dummy() -> Self {
        Self { start: 0, end: 0 }
    }
}

// TYPE ERRORs
#[derive(Debug, Clone)]
pub enum TypeErrorKind {
    Mismatch { expected: String, found: String },
    UnknownIdentifier { name: String },
    NotCallable { ty: String },
    // we can add more error kinds later
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub msg: String,
    pub span: Span,
    pub kind: TypeErrorKind,
}

impl TypeError {
    pub fn new(msg: String, span: Span, kind: TypeErrorKind) -> Self {
        Self { msg, span, kind }
    }
}

// // NAME ERRORS
// #[derive(Debug, Clone)]
// pub enum NameErrorKind {
//     StructFieldNotFound { st_name: String, f_name: String },
//     UnknownIdentifier { name: String },
//     // we can add more error kinds later
// }

// #[derive(Debug, Clone)]
// pub struct NameError {
//     pub msg: String,
//     pub span: Span,
//     pub kind: NameErrorKind,
// }

// impl NameError {
//     pub fn new(msg: String, span: Span, kind: NameErrorKind) -> Self {
//         Self { msg, span, kind }
//     }
// }
