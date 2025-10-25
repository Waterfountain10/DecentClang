//! Core assembler utilities (human-readable asm -> byte level representation)

use std::{fmt, iter::Map};
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

pub fn int64_of_sbytes(bs: Vec<SByte>) -> i64 {
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

pub fn sbytes_of_ins(ins: Ins) -> Result<Vec<SByte>, String> {
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
        SByte::InsB0(ins),
        SByte::InsFrag,
        SByte::InsFrag,
        SByte::InsFrag,
        SByte::InsFrag,
        SByte::InsFrag,
        SByte::InsFrag,
        SByte::InsFrag,
    ])
}

pub fn sbytes_of_data(d: Data) -> Result<Vec<SByte>, String> {
    match d {
        Data::Quad(Imm::Lbl(_)) => Err("sbytes_of_data: tried to serialize a label".to_string()),
        Data::Asciz(s) => Ok(sbytes_of_string(&s)),
        Data::Quad(Imm::Lit(i)) => Ok(sbytes_of_int64(i)),
    }
}
