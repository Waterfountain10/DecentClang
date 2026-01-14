//! Assembly parsing helpers (.asm text -> x86::Prog/Reg/Ins/etc...)

/// Parse a `.s` text file into an internal `Prog`
///
/// Supports:
/// ```asm
/// .text
/// main:
///   movq $42, %rax
///   retq
///
/// .data
/// msg: .asciz "Hello"
/// num: .quad 99
/// ```
use std::error::Error;
use x86::asm::*;
use x86::*;

pub fn parse_program(src: &str) -> Result<Prog, Box<dyn Error>> {
    let mut elems = Vec::new();
    let mut section = String::from(".text");
    let mut cur_label = String::new();
    let mut cur_instrs = Vec::new();
    let mut cur_data = Vec::new();

    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }

        // Switch sections
        if line.starts_with(".text") {
            section = ".text".into();
            continue;
        } else if line.starts_with(".data") {
            section = ".data".into();
            continue;
        }

        // Label line
        if line.ends_with(':') {
            // Commit previous label’s contents before starting new one
            if !cur_label.is_empty() {
                if section == ".text" && !cur_instrs.is_empty() {
                    elems.push(gtext(&cur_label, cur_instrs.clone()));
                    cur_instrs.clear();
                } else if section == ".data" && !cur_data.is_empty() {
                    elems.push(data(&cur_label, cur_data.clone()));
                    cur_data.clear();
                }
            }

            cur_label = line.trim_end_matches(':').to_string();
            continue;
        }

        // Text section instruction
        if section == ".text" {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }
            let opcode = tokens[0].to_lowercase();

            // Very minimal parser for operands
            let operands: Vec<Operand> = if tokens.
                () > 1 {
                tokens[1]
                    .split(',')
                    .map(|tok| parse_operand(tok.trim()))
                    .collect::<Result<_, _>>()?
            } else {
                vec![]
            };

            cur_instrs.push(Ins {
                opcode: parse_opcode(&opcode)?,
                operands,
            });
        } else if section == ".data" {
            // Data section line
            if line.contains(".asciz") {
                let value = line
                    .split(".asciz")
                    .nth(1)
                    .unwrap()
                    .trim()
                    .trim_matches('"')
                    .to_string();
                cur_data.push(Data::Asciz(value));
            } else if line.contains(".quad") {
                let num_str = line.split(".quad").nth(1).unwrap().trim();
                let val: i64 = num_str.parse()?;
                cur_data.push(Data::Quad(Imm::Lit(val)));
            }
        }
    }

    // Push last label
    if !cur_label.is_empty() {
        if section == ".text" && !cur_instrs.is_empty() {
            elems.push(gtext(&cur_label, cur_instrs.clone()));
        } else if section == ".data" && !cur_data.is_empty() {
            elems.push(data(&cur_label, cur_data.clone()));
        }
    }

    Ok(Prog(elems))
}

/// Helper: map textual opcode → enum
fn parse_opcode(s: &str) -> Result<Opcode, Box<dyn Error>> {
    Ok(match s {
        "movq" => Opcode::Movq,
        "retq" => Opcode::Retq,
        "jmp" => Opcode::Jmp,
        _ => return Err(format!("unknown opcode '{}'", s).into()),
    })
}

/// Helper: parse operands like `$42`, `%rax`, or `loop`
fn parse_operand(tok: &str) -> Result<Operand, Box<dyn Error>> {
    if tok.starts_with('$') {
        let val: i64 = tok.trim_start_matches('$').parse()?;
        Ok(Operand::Imm(Imm::Lit(val)))
    } else if tok.starts_with('%') {
        Ok(Operand::Reg(parse_register(tok)?))
    } else {
        // Assume it's a label
        Ok(Operand::Imm(Imm::Lbl(tok.to_string())))
    }
}

fn parse_register(s: &str) -> Result<Reg, Box<dyn Error>> {
    Ok(match s.trim_start_matches('%') {
        "rax" => Reg::Rax,
        "rbx" => Reg::Rbx,
        "rcx" => Reg::Rcx,
        "rdx" => Reg::Rdx,
        _ => return Err(format!("unknown register '{}'", s).into()),
    })
}
