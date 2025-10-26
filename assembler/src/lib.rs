//! Core assembler utilities (human-readable asm -> byte level representation)

use std::{collections::HashMap, fmt, iter::Map};
use x86::*;

pub enum SByte {
    InsB0(Ins), // 1st byte of ins
    InsFrag,    // 2nd - 8th bytes of ins
    Byte(char), // non instruction byte, example filler padding or wtv
}

pub fn rind(r: Reg) -> u8 {
    match r {
        Reg::Rip => 16,
        Reg::Rax => 0,
        Reg::Rbx => 1,
        Reg::Rcx => 2,
        Reg::Rdx => 3,
        Reg::Rsi => 4,
        Reg::Rdi => 5,
        Reg::Rbp => 6,
        Reg::Rsp => 7,
        Reg::R08 => 8,
        Reg::R09 => 9,
        Reg::R10 => 10,
        Reg::R11 => 11,
        Reg::R12 => 12,
        Reg::R13 => 13,
        Reg::R14 => 14,
        Reg::R15 => 15,
    }
}

// helpers for reading/writing sbytes

pub fn sbytes_of_int64(i: i64) -> Vec<SByte> {
    [0, 8, 16, 24, 32, 40, 48, 56]
        .iter()
        .map(|n| {
            let char_byte = ((i >> n) & 0xff) as u8 as char;
            SByte::Byte(char_byte)
        })
        .collect() // collects all 8 bytes into vec<Sbytes>
}

pub fn int64_of_sbytes(bs: &Vec<SByte>) -> i64 {
    bs.iter().rev().fold(0i64, |acc, b| match b {
        SByte::Byte(c) => (acc << 8) | (*c as u8 as i64), // shifted acc becomes last 2 digits, and c is the highest ones
        _ => 0i64,                                        // start with acc = 0
    })
}

pub fn sbytes_of_string(s: &str) -> Vec<SByte> {
    s.chars()
        .map(|c| SByte::Byte(c))
        .chain(std::iter::once(SByte::Byte('\x00')))
        .collect()
}

pub fn sbytes_of_ins(ins: &Ins) -> Result<Vec<SByte>, String> {
    // Check that no operand contains a label
    for operand in &ins.operands {
        match operand {
            Operand::Imm(Imm::Lbl(_))
            | Operand::Ind1(Imm::Lbl(_))
            | Operand::Ind3(Imm::Lbl(_), _) => {
                return Err("sbytes_of_ins: tried to serialize a label!".to_string());
            }
            _ => {}
        }
    }
    // simply return vec, after checking passed
    Ok(vec![
        SByte::InsB0(ins.clone()),
        SByte::InsFrag,
        SByte::InsFrag,
        SByte::InsFrag,
        SByte::InsFrag,
        SByte::InsFrag,
        SByte::InsFrag,
        SByte::InsFrag,
    ])
}

pub fn sbytes_of_data(d: &Data) -> Result<Vec<SByte>, String> {
    match d {
        Data::Quad(Imm::Lbl(_)) => Err("sbytes_of_data: tried to serialize a label".to_string()),
        Data::Asciz(s) => Ok(sbytes_of_string(&s)),
        Data::Quad(Imm::Lit(i)) => Ok(sbytes_of_int64(i.clone())),
    }
}

/// Assemble should raise this when a label is used but not defined
#[derive(Debug, Clone)]
pub struct UndefinedSym(pub Lbl);
impl std::error::Error for UndefinedSym {}
impl std::fmt::Display for UndefinedSym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Undefined symbol: {}", self.0)
    }
}

/// Assemble should raise this when a label is defined more than once
#[derive(Debug, Clone)]
pub struct RedefinedSym(pub Lbl);
impl std::error::Error for RedefinedSym {}
impl std::fmt::Display for RedefinedSym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Redefined symbol: {}", self.0)
    }
}

pub struct Exec {
    pub entry: Quad,
    pub text_pos: Quad,
    pub data_pos: Quad,
    pub text_seg: Vec<SByte>,
    pub data_seg: Vec<SByte>,
}

/// Resolve a symbol (label) to its address
pub fn resolve_sym(lbl: &str, map: &HashMap<String, i64>) -> Result<i64, UndefinedSym> {
    map.get(lbl)
        .copied()
        .ok_or_else(|| UndefinedSym(lbl.to_string()))
}

/// Helper: add a symbol to the symmap, checking for redefinition
pub fn add_sym(map: &mut HashMap<String, i64>, lbl: &str, addr: i64) -> Result<(), RedefinedSym> {
    if map.contains_key(lbl) {
        Err(RedefinedSym(lbl.to_string()))
    } else {
        map.insert(lbl.to_string(), addr);
        Ok(())
    }
}

/// Calculate the size of an instruction block (8 bytes per instruction)
pub fn ins_block_size(ins_list: &[Ins]) -> i64 {
    (ins_list.len() as i64) * 8
}

/// Calculate the size of a data block
pub fn data_block_size(data_list: &[Data]) -> i64 {
    data_list
        .iter()
        .map(|d| match d {
            Data::Asciz(s) => (s.len() as i64) + 1, // +1 for null terminator
            Data::Quad(_) => 8,
        })
        .sum()
}
