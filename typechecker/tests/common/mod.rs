// Test helper functions for building AST nodes quickly

use ast::*;
use common::{Span, Spanned};

// Helper to create a dummy span
pub fn dummy_span() -> Span {
    Span::new(0, 0)
}

// Helper to create a Node with dummy location
pub fn node<T>(elt: T) -> Node<T> {
    Node {
        elt,
        loc: dummy_span(),
    }
}

// Helper to create a Spanned type
pub fn spanned<T>(node: T) -> Spanned<T> {
    Spanned::new(dummy_span(), node)
}

// Type constructors
pub fn t_int() -> Ty {
    Ty::TInt
}

pub fn t_bool() -> Ty {
    Ty::TBool
}

pub fn t_ref(rty: RefTy) -> Ty {
    Ty::TRef(spanned(rty))
}

pub fn t_null_ref(rty: RefTy) -> Ty {
    Ty::TNullRef(spanned(rty))
}

pub fn r_string() -> RefTy {
    RefTy::RString
}

pub fn r_array(ty: Ty) -> RefTy {
    RefTy::RArray(Box::new(spanned(ty)))
}

pub fn r_fun(args: Vec<Ty>, ret: RetTy) -> RefTy {
    let spanned_args = args.into_iter().map(|t| spanned(t)).collect();
    RefTy::RFun(spanned_args, Box::new(spanned(ret)))
}

pub fn r_struct(name: &str) -> RefTy {
    RefTy::RStruct(name.to_string())
}

pub fn ret_void() -> RetTy {
    RetTy::RetVoid
}

pub fn ret_val(ty: Ty) -> RetTy {
    RetTy::RetVal(Box::new(spanned(ty)))
}

// Expression constructors
pub fn e_int(i: i64) -> Node<SExp> {
    node(spanned(Exp::CInt(i)))
}

pub fn e_bool(b: bool) -> Node<SExp> {
    node(spanned(Exp::CBool(b)))
}

pub fn e_str(s: &str) -> Node<SExp> {
    node(spanned(Exp::CStr(s.to_string())))
}

pub fn e_id(name: &str) -> Node<SExp> {
    node(spanned(Exp::Id(name.to_string())))
}

pub fn e_null(rty: RefTy) -> Node<SExp> {
    node(spanned(Exp::CNull(spanned(rty))))
}

pub fn e_bop(op: BinOp, left: Node<SExp>, right: Node<SExp>) -> Node<SExp> {
    node(spanned(Exp::Bop(op, Box::new(left), Box::new(right))))
}

pub fn e_uop(op: UnOp, expr: Node<SExp>) -> Node<SExp> {
    node(spanned(Exp::Uop(op, Box::new(expr))))
}

pub fn e_call(func: Node<SExp>, args: Vec<Node<SExp>>) -> Node<SExp> {
    node(spanned(Exp::Call(Box::new(func), args)))
}

pub fn e_index(arr: Node<SExp>, idx: Node<SExp>) -> Node<SExp> {
    node(spanned(Exp::Index(Box::new(arr), Box::new(idx))))
}

pub fn e_new_arr(ty: Ty, size: Node<SExp>) -> Node<SExp> {
    node(spanned(Exp::NewArr(ty, Box::new(size))))
}

// Statement constructors
pub fn s_assn(lhs: Node<SExp>, rhs: Node<SExp>) -> Node<SStmt> {
    node(spanned(Stmt::Assn(lhs, rhs)))
}

pub fn s_decl(name: &str, init: Node<SExp>) -> Node<SStmt> {
    node(spanned(Stmt::Decl(VDecl {
        vd_id: name.to_string(),
        vd_node: Some(init),
    })))
}

pub fn s_ret(expr: Option<Node<SExp>>) -> Node<SStmt> {
    node(spanned(Stmt::Ret(expr)))
}

pub fn s_if(
    guard: Node<SExp>,
    then_block: Vec<Node<SStmt>>,
    else_block: Vec<Node<SStmt>>,
) -> Node<SStmt> {
    node(spanned(Stmt::If(guard, then_block, else_block)))
}

pub fn s_while(guard: Node<SExp>, body: Vec<Node<SStmt>>) -> Node<SStmt> {
    node(spanned(Stmt::While(guard, body)))
}

// Declaration constructors
pub fn d_gvar(name: &str, init: Node<SExp>) -> Decl {
    Decl::GVDecl(node(GDecl {
        name: name.to_string(),
        init,
    }))
}

pub fn d_func(name: &str, args: Vec<(&str, Ty)>, ret: RetTy, body: Vec<Node<SStmt>>) -> Decl {
    let args = args
        .into_iter()
        .map(|(id, ty)| Arg {
            ty: spanned(ty),
            id: id.to_string(),
        })
        .collect();

    Decl::GFDecl(node(FDecl {
        fret_ty: ret,
        fname: name.to_string(),
        args,
        body,
    }))
}

pub fn d_struct(name: &str, fields: Vec<(&str, Ty)>) -> Decl {
    let fields = fields
        .into_iter()
        .map(|(name, ty)| Field {
            field_name: name.to_string(),
            field_type: ty,
        })
        .collect();

    Decl::GTDecl(node(TDecl {
        td_id: name.to_string(),
        td_node: fields,
    }))
}
