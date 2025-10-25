//! X86Lite IR

use std::fmt;

pub type Lbl = String;
pub type Quad = i64;

#[derive(Debug, Clone)]
pub enum Imm {
    Lit(Quad),
    Lbl(Lbl),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Copy)]
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
    Set(Cnd),
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

#[derive(Debug, Clone)]
pub struct Prog(pub Vec<Elem>);

// -----------------------------------------------------------------------------
// Syntactic sugar for writing x86 assembly code
// -----------------------------------------------------------------------------
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

// -----------------------------------------------------------------------------
// Pretty printing
// -----------------------------------------------------------------------------

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Reg::Rip => "%rip",
            Reg::Rax => "%rax",
            Reg::Rbx => "%rbx",
            Reg::Rcx => "%rcx",
            Reg::Rdx => "%rdx",
            Reg::Rsi => "%rsi",
            Reg::Rdi => "%rdi",
            Reg::Rbp => "%rbp",
            Reg::Rsp => "%rsp",
            Reg::R08 => "%r8",
            Reg::R09 => "%r9",
            Reg::R10 => "%r10",
            Reg::R11 => "%r11",
            Reg::R12 => "%r12",
            Reg::R13 => "%r13",
            Reg::R14 => "%r14",
            Reg::R15 => "%r15",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Imm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Imm::Lit(i) => write!(f, "{}", i),
            Imm::Lbl(l) => write!(f, "{}", l),
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Imm(i) => write!(f, "${}", i),
            Operand::Reg(r) => write!(f, "{}", r),
            Operand::Ind1(i) => write!(f, "{}", i),
            Operand::Ind2(r) => write!(f, "({})", r),
            Operand::Ind3(i, r) => write!(f, "{}({})", i, r),
        }
    }
}

impl fmt::Display for Cnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Cnd::Eq => "e",
            Cnd::Neq => "ne",
            Cnd::Gt => "g",
            Cnd::Ge => "ge",
            Cnd::Lt => "l",
            Cnd::Le => "le",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::Movq => write!(f, "movq"),
            Opcode::Pushq => write!(f, "pushq"),
            Opcode::Popq => write!(f, "popq"),
            Opcode::Leaq => write!(f, "leaq"),
            Opcode::Incq => write!(f, "incq"),
            Opcode::Decq => write!(f, "decq"),
            Opcode::Negq => write!(f, "negq"),
            Opcode::Notq => write!(f, "notq"),
            Opcode::Addq => write!(f, "addq"),
            Opcode::Subq => write!(f, "subq"),
            Opcode::Imulq => write!(f, "imulq"),
            Opcode::Xorq => write!(f, "xorq"),
            Opcode::Orq => write!(f, "orq"),
            Opcode::Andq => write!(f, "andq"),
            Opcode::Shlq => write!(f, "shlq"),
            Opcode::Sarq => write!(f, "sarq"),
            Opcode::Shrq => write!(f, "shrq"),
            Opcode::Jmp => write!(f, "jmp"),
            Opcode::J(c) => write!(f, "j{}", c),
            Opcode::Cmpq => write!(f, "cmpq"),
            Opcode::Set(c) => write!(f, "set{}", c),
            Opcode::Callq => write!(f, "callq"),
            Opcode::Retq => write!(f, "retq"),
        }
    }
}

impl fmt::Display for Ins {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<String> = self.operands.iter().map(|a| format!("{}", a)).collect();
        write!(f, "\t{}\t{}", self.opcode, args.join(", "))
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Data::Asciz(s) => write!(f, "\t.asciz\t\"{}\"", s.escape_default()),
            Data::Quad(i) => write!(f, "\t.quad\t{}", i),
        }
    }
}

impl fmt::Display for Asm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Asm::Text(ins) => {
                writeln!(f, "\t.text")?;
                for i in ins {
                    writeln!(f, "{}", i)?;
                }
            }
            Asm::Data(ds) => {
                writeln!(f, "\t.data")?;
                for d in ds {
                    writeln!(f, "{}", d)?;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Elem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let section = match &self.asm {
            Asm::Text(_) => "\t.text",
            Asm::Data(_) => "\t.data",
        };
        let global = if self.global {
            format!("\t.globl\t{}\n", self.lbl)
        } else {
            "".to_string()
        };
        writeln!(f, "{}\n{}{}:\n{}", section, global, self.lbl, self.asm)
    }
}

impl fmt::Display for Prog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for elem in &self.0 {
            writeln!(f, "{}", elem)?;
        }
        Ok(())
    }
}
