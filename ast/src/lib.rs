#! Abstract Syntax Tree for Oat (aka mini C language)

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeTy {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    pub elt: T,
    pub loc: RangeTy,
}

pub type IdTy = String;

pub enum Ty {
    Tbool,
    TInt,
    TRef(RefTy),
}

pub enum RefTy {
    RString,
    RArray(Box<Ty>),
    RFun(Vec<Ty>, RetTy),
}

pub enum RetTy {
    RetVoid,
    RetVal(Box<Ty>),
}

pub enum UnOp {
    Neg,
    LogNot,
    BitNot,
}

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

pub enum Exp {
    CNull(RefTy),
    CBool(bool),
    CInt(i64),
    CStr(String),
    CArr(Ty, Vec<Node<Exp>>),
    NewArr(Ty, Box<Node<Exp>>),
    Id(IdTy),
    Index(Box<Node<Exp>>, Box<Node<Exp>>),
    Call(Box<Node<Exp>>, Vec<Node<Exp>>),
    Bop(BinOp, Box<Node<Exp>>, Box<Node<Exp>>),
    Uop(UnOp, Box<Node<Exp>>),
}

pub struct CField {
    pub cf_id: IdTy,
    pub cf_node: Node<Exp>,
}

pub struct VDecl {
    pub vd_id: IdTy,
    pub vd_node: Node<Exp>,
}

pub enum Stmt {
    Assn(Node<Exp>, Node<Exp>),
    Decl(VDecl),
    Ret(Option<Node<Exp>>),
    SCall(Node<Exp>, Vec<Node<Exp>>),
    If(Node<Exp>, Vec<Node<Stmt>>, Vec<Node<Stmt>>),
    For(
        Vec<VDecl>,
        Option<Node<Exp>>,
        Option<Box<Node<Stmt>>>,
        Vec<Node<Stmt>>,
    ),
    While(Node<Exp>, Vec<Node<Stmt>>),
}

pub type Block = Vec<Node<Stmt>>;

pub struct GDecl {
    pub name: IdTy,
    pub init: Node<Exp>,
}

pub struct Arg {
    pub ty: Ty,
    pub id: IdTy,
}

pub struct FDecl {
    pub fret_ty: RetTy,
    pub fname: IdTy,
    pub args: Vec<Arg>,
    pub body: Block,
}

pub struct Field {
    pub field_name: IdTy,
    pub field_type: Ty,
}

pub enum Decl {
    GvDecl(Node<GDecl>),
    GfDecl(Node<FDecl>),
}

pub type Prog = Vec<Decl>;
