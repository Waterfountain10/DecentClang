#! Abstract Syntax Tree for Oat

use common;

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub elt: T,
    pub loc: common::Span,
}

pub type IdTy = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    TBool,
    TInt,
    TRef(SRefTy),
    TNullRef(SRefTy),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RefTy {
    RString,
    RArray(Box<STy>),
    RFun(Vec<STy>, Box<SRetTy>),
    RStruct(IdTy),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RetTy {
    RetVoid,
    RetVal(Box<STy>),
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Neg,
    LogNot,
    BitNot,
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
    IAnd,
    IOr,
    Shl,
    Shr,
    Sar,
}

#[derive(Debug, Clone)]
pub enum Exp {
    CNull(SRefTy),
    CBool(bool),
    CInt(i64),
    CStr(String),
    CArr(Ty, Vec<Node<SExp>>),
    NewArr(Ty, Box<Node<SExp>>),
    Id(IdTy),
    Index(Box<Node<SExp>>, Box<Node<SExp>>),
    Call(Box<Node<SExp>>, Vec<Node<SExp>>),
    Bop(BinOp, Box<Node<SExp>>, Box<Node<SExp>>),
    Uop(UnOp, Box<Node<SExp>>),
}

#[derive(Debug, Clone)]
pub struct CField {
    pub cf_id: IdTy,
    pub cf_node: Node<SExp>,
}

// VDecl example:
//   int x = 5; where vd_id=IntTy, vd_node=Some<Node<CInt(5)>>
//   int x;     where vd_id=IntTy, vd_node=None
#[derive(Debug, Clone)]
pub struct VDecl {
    pub vd_id: IdTy,
    pub vd_node: Option<Node<SExp>>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assn(Node<SExp>, Node<SExp>),
    Decl(VDecl),
    Ret(Option<Node<SExp>>),
    SCall(Node<SExp>, Vec<Node<SExp>>),
    If(Node<SExp>, Vec<Node<SStmt>>, Vec<Node<SStmt>>),
    For(
        Vec<VDecl>,
        Option<Node<SExp>>,
        Option<Box<Node<SStmt>>>,
        Vec<Node<SStmt>>,
    ),
    While(Node<SExp>, Vec<Node<SStmt>>),
}

pub type Block = Vec<Node<SStmt>>;

#[derive(Debug, Clone)]
pub struct GDecl {
    pub name: IdTy,
    pub init: Node<SExp>,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub ty: STy,
    pub id: IdTy,
}

#[derive(Debug, Clone)]
pub struct FDecl {
    pub fret_ty: RetTy,
    pub fname: IdTy,
    pub args: Vec<Arg>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub field_name: IdTy,
    pub field_type: Ty,
}

#[derive(Debug, Clone)]
pub struct TDecl {
    pub td_id: IdTy,
    pub td_node: Vec<Field>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    GVDecl(Node<GDecl>),
    GFDecl(Node<FDecl>),
    GTDecl(Node<TDecl>),
}

pub type Prog = Vec<Decl>;

// Spanned-related constructs (useful for error-log in typechecker)
pub type STy = common::Spanned<Ty>;
pub type SRetTy = common::Spanned<RetTy>;
pub type SRefTy = common::Spanned<RefTy>;
pub type SExp = common::Spanned<Exp>;
pub type SStmt = common::Spanned<Stmt>;
