//! LLVM IR

pub type Uid = String;
pub type Gid = String;
pub type Tid = String; // type id
pub type Lbl = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Void,
    I1,
    I8,
    I64,
    Ptr(Box<Ty>),
    Struct(Vec<Ty>),
    Array(usize, Box<Ty>),
    Fun(Vec<Ty>, Box<Ty>),
    Namedt(Tid),
}

/// function type: argument types and return type
pub type Fty = (Vec<Ty>, Ty);

/// values
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Null,
    Const(i64),
    Gid(Gid),
    Id(Uid),
}

/// binary ops
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bop {
    Add,
    Sub,
    Mul,
    Shl,
    Lshr,
    Ashr,
    And,
    Or,
    Xor,
}

/// comparisons ops
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cnd {
    Eq,
    Ne,
    Slt,
    Sle,
    Sgt,
    Sge,
}

/// instructions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Insn {
    Binop(Bop, Ty, Operand, Operand),
    Alloca(Ty),
    Load(Ty, Operand),
    Store(Ty, Operand, Operand),
    Icmp(Cnd, Ty, Operand, Operand),
    Call(Ty, Operand, Vec<(Ty, Operand)>),
    Bitcast(Ty, Operand, Ty),
    Gep(Ty, Operand, Vec<Operand>),
}

/// terminators (i.e end of block)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminator {
    Ret(Ty, Option<Operand>),
    Br(Lbl),
    Cbr(Operand, Lbl, Lbl),
}

/// Basic Blocks
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub insns: Vec<(Uid, Insn)>,
    pub term: (Uid, Terminator),
}

/// Control Flow Graphs: entry and labeled blocks
pub type Cfg = (Block, Vec<(Lbl, Block)>);

/// Function Declarations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fdecl {
    pub f_ty: Fty,
    pub f_param: Vec<Uid>,
    pub f_cfg: Cfg,
}

/// Global Data Initializers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ginit {
    GNull,
    GGid(Gid),
    GInt(i64),
    GString(String),
    GArray(Vec<(Ty, Ginit)>),
    GStruct(Vec<(Ty, Ginit)>),
    GBitcast(Ty, Box<Ginit>, Ty),
}

/// Global Declarations
pub type Gdecl = (Ty, Ginit);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prog {
    pub tdecls: Vec<(Tid, Ty)>,    // type def
    pub gdecls: Vec<(Gid, Gdecl)>, // global var
    pub fdecls: Vec<(Gid, Fdecl)>, // fun def
    pub edecls: Vec<(Gid, Ty)>,    // external declarations (ex. declare i64 @printf(i8*,...))
}

impl Prog {
    pub fn new() -> Self {
        Prog {
            tdecls: Vec::new(),
            gdecls: Vec::new(),
            fdecls: Vec::new(),
            edecls: Vec::new(),
        }
    }
}

impl Default for Prog {
    fn default() -> Self {
        Self::new()
    }
}

impl Block {
    pub fn new(insns: Vec<(Uid, Insn)>, term: (Uid, Terminator)) -> Self {
        Block { insns, term }
    }
}

impl Fdecl {
    pub fn new(f_ty: Fty, f_param: Vec<Uid>, f_cfg: Cfg) -> Self {
        Fdecl {
            f_ty,
            f_param,
            f_cfg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ty_creation() {
        let t = Ty::I64;
        assert_eq!(t, Ty::I64);

        let ptr = Ty::Ptr(Box::new(Ty::I8));
        assert!(matches!(ptr, Ty::Ptr(_)));
    }

    #[test]
    fn test_operand_creation() {
        let op = Operand::Const(42);
        assert_eq!(op, Operand::Const(42));
    }

    #[test]
    fn test_prog_creation() {
        let prog = Prog::new();
        assert!(prog.tdecls.is_empty());
        assert!(prog.gdecls.is_empty());
        assert!(prog.fdecls.is_empty());
        assert!(prog.edecls.is_empty());
    }

    #[test]
    fn test_block_creation() {
        let insns = vec![];
        let term = ("ret".to_string(), Terminator::Ret(Ty::Void, None));
        let block = Block::new(insns, term);
        assert!(block.insns.is_empty());
    }
}
