#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn dummy() -> Self {
        Self { start: 0, end: 0 }
    }
}

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
