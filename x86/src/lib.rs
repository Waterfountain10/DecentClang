//! X86Lite IR

use std::fmt;

pub type Lbl = String;
pub type Quad = i64;

#[derive(Debug, Clone)]
pub enum Imm {
    Lit(Quad),
    Lbl(Lbl),
}

#[derive(Debug, Clone, Eq, Copy)]
pub enum Reg {
    Rip,
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    Rbp,
    Rsp,
    R08,
    R09,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Imm(Imm),
    Reg(Reg),
    Ind1(Imm),      // displ       ex. some_label:
    Ind2(Reg),      // (%reg)      ex. Rax
    Ind3(Imm, Reg), // displ(%reg) ex. 2bytes(Rax)
}

#[derive(Debug, Clone)]
pub enum Cnd {
    Eq,
    Neq,
    Gt,
    Ge,
    Lt,
    Le,
}

#[derive(Debug, Clone)]
pub enum Opcode {
    Movq,
    Pushq,
    Popq,
    Leaq,
    Incq,
    Decq,
    Negq,
    Notq,
    Addq,
    Subq,
    Imulq,
    Xorq,
    Orq,
    Andq,
    Shlq,
    Sarq,
    Shrq,
    Jmp,
    J(Cnd),
    Cmpq,
    Set(Cdn),
    Callq,
    Retq,
}

// instructions in x86 : opcode AND its operands ... ex. Movq, (Rax, Rbx, Imm(21))
#[derive(Debug, Clone)]
pub struct Ins {
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
}

#[derive(Debug, Clone)]
pub enum Data {
    Asciz(String),
    Quad(Imm),
}

// Assembly code
#[derive(Debug, Clone)]
pub enum Asm {
    Text(Vec<Ins>),  // code
    Data(Vec<Data>), // data
}

// Labeled blocks of data or code ex. Loop1 (...) not global
#[derive(Debug, Clone)]
pub struct Elem {
    pub lbl: Lbl,
    pub global: bool, // global var
    pub asm: Asm,
}

pub type Prog = Vec<Elem>;

// Syntactic sugar for writing x86 assembly code
pub mod asm {
    use super::*;

    /// Integer literal → Imm::Lit
    ///
    /// Example: `lit(42)` → `Operand::Imm(Imm::Lit(42))`
    pub fn lit(i: i32) -> Operand {
        Operand::Imm(Imm::Lit(i as i64))
    }

    /// Label reference → Imm::Lbl
    ///
    /// Example: `lbl("main")` → `Operand::Imm(Imm::Lbl("main"))`
    pub fn lbl(s: &str) -> Operand {
        Operand::Imm(Imm::Lbl(s.to_string()))
    }

    /// Register reference → Reg
    ///
    /// Example: `reg(Reg::Rax)` → `Operand::Reg(Reg::Rax)`
    pub fn reg(r: Reg) -> Operand {
        Operand::Reg(r)
    }

    /// Helper for data sections (global)
    ///
    /// Example: `data("str", vec![Data::Asciz("hello".to_string())])`
    pub fn data(label: &str, ds: Vec<Data>) -> Elem {
        Elem {
            lbl: label.to_string(),
            global: true,
            asm: Asm::Data(ds),
        }
    }

    /// Helper for code sections (non-global)
    ///
    /// Example: `text("loop", vec![...])`
    pub fn text(label: &str, is: Vec<Ins>) -> Elem {
        Elem {
            lbl: label.to_string(),
            global: false,
            asm: Asm::Text(is),
        }
    }

    /// Helper for global code sections
    ///
    /// Example: `gtext("main", vec![...])`
    pub fn gtext(label: &str, is: Vec<Ins>) -> Elem {
        Elem {
            lbl: label.to_string(),
            global: true,
            asm: Asm::Text(is),
        }
    }
}
