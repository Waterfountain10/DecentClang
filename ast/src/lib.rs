#! Abstract Syntax Tree for Oat


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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    TBool,
    TInt,
    TRef(RefTy),
    TNullRef(RefTy), // TODO: did we want tnullref here or somehwer else??
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefTy {
    RString,
    RArray(Box<Ty>),
    RFun(Vec<Ty>, Box<RetTy>),
    RStruct(IdTy), // TODO: did we want structs here or somehwer else??
}

trait IntoRefTy {}

trait Zero {
    const ZERO: Self;
    fn is_zero(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetTy {
    RetVoid,
    RetVal(Box<Ty>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    LogNot,
    BitNot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct CField {
    pub cf_id: IdTy,
    pub cf_node: Node<Exp>,
}

#[derive(Debug, Clone)]
pub struct VDecl {
    pub vd_id: IdTy,
    pub vd_node: Node<Exp>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct GDecl {
    pub name: IdTy,
    pub init: Node<Exp>,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub ty: Ty,
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
