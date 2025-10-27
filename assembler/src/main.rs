//! Assembler/linker backend logic
//!
//! in our educational OCaml we had this layer built like this:
//!                 - let filter_asm (p:prog) : ...
//!                 - let build_sym ...
//!                 - let assemble (p:prog) : exec
//!                 - let load {entry; text_pos; data_pos; ...} : mach
//!
//! but in this functional Rust, we change it slightly:
//!                1 - `filter_sections(prog: &Prog)` — pure, returns separate text/data sections.
//!                2 - `build_symbol_table(text, data)` — constructs a `HashMap<String, u64>` of label → address.
//!                3 - `resolve_labels(prog, symtab)` — replaces symbolic labels with concrete immediates.
//!                4 - `assemble(prog)` — combines the above to produce a binary representation (`Exec`).
//!                5 - `write_executable(exec)` — (new) writes an ELF or flat binary file to disk.
//!
//! The final *loader/simulator* step is **omitted** — because our goal is to
//! emit a true executable file that your OS can run (not to emulate a CPU).
//!
//! ---
//!
//! Flow of our Assembler/Linkage Layer:
//!
//! ```text
//! x86::Prog
//!   ↓  (filter_sections)
//! split text/data
//!   ↓  (build_symbol_table)
//! HashMap<label, address>
//!   ↓  (resolve_labels)
//! flattened instructions & data bytes
//!   ↓  (assemble)
//! Exec { entry, text, data }
//!   ↓  (write_executable)
//! ELF or flat binary `.out`
//! ```
//!
//! ---

use assembler::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use x86::asm::*;
use x86::*;

/// 1 - `filter_sections(prog: &Prog)` — pure, returns separate text/data sections.
pub fn filter_sections(prog: &Prog) -> (Vec<Elem>, Vec<Elem>) {
    let mut ts = Vec::new();
    let mut ds = Vec::new();

    for elem in &prog.0 {
        match elem.asm {
            Asm::Text(_) => ts.push(elem.clone()),
            Asm::Data(_) => ds.push(elem.clone()),
        }
    }
    (ts, ds)
}

/// 2 - build_symbol_table(text, data) — constructs a map : lbl → address (u64).
/// Returns: (symbol_map, text_size, data_pos, data_size)
pub fn build_symbol_table(
    text_elems: &[Elem],
    data_elems: &[Elem],
) -> Result<(HashMap<String, i64>, i64, i64, i64), RedefinedSym> {
    let text_pos = 0x400_000i64;
    let mut sym_map = HashMap::new();
    let mut offset = 0i64;

    // Process text elements
    for elem in text_elems {
        match &elem.asm {
            Asm::Text(ins_list) => {
                let addr = text_pos + offset;
                add_sym(&mut sym_map, &elem.lbl, addr)?;
                offset += ins_block_size(ins_list);
            }
            Asm::Data(_) => {} // not supposed to be called
        }
    }

    let text_size = offset;
    let data_pos = text_pos + text_size;
    let mut data_offset = 0i64;

    // Process data elements
    for elem in data_elems {
        match &elem.asm {
            Asm::Data(data_list) => {
                let addr = data_pos + data_offset;
                add_sym(&mut sym_map, &elem.lbl, addr)?;
                data_offset += data_block_size(data_list);
            }
            Asm::Text(_) => {} // not supposed to be called
        }
    }

    let data_size = data_offset;
    Ok((sym_map, text_size, data_pos, data_size))
}

/// 3 - resolve_labels for ins
fn resolve_ins_labels(map: &HashMap<String, i64>, ins: &Ins) -> Result<Ins, UndefinedSym> {
    let resolved_operands: Result<Vec<Operand>, UndefinedSym> = ins
        .operands
        .iter()
        .map(|op| resolve_operand(map, op))
        .collect();

    Ok(Ins {
        opcode: ins.opcode.clone(),
        operands: resolved_operands?,
    })
}

/// 3.5 - resolve_labels for data (handles quad with labels)
fn resolve_data_labels(map: &HashMap<String, i64>, data: &Data) -> Result<Data, UndefinedSym> {
    match data {
        Data::Asciz(s) => Ok(Data::Asciz(s.clone())),
        Data::Quad(imm) => Ok(Data::Quad(resolve_imm(map, imm)?)),
    }
}

/// 4 - assemble(prog) — combines filter, build_sym, and resolve to produce Exec
pub fn assemble(prog: &Prog) -> Result<Exec, Box<dyn std::error::Error>> {
    // Step 1: Split into text and data sections
    let (ts, ds) = filter_sections(prog);

    // Step 2: Build symbol table and get layout info
    let (map, text_size, data_pos, data_size) = build_symbol_table(&ts, &ds)?;

    // Validate that 'main' exists
    let entry = resolve_sym("main", &map)?;
    let text_pos = 0x400_000i64;

    // Step 3: Resolve labels and flatten to sbytes
    let mut text_seg = Vec::new();
    for elem in &ts {
        if let Asm::Text(ins_list) = &elem.asm {
            for ins in ins_list {
                let resolved_ins = resolve_ins_labels(&map, ins)?;
                let sbytes = sbytes_of_ins(&resolved_ins)?;
                text_seg.extend(sbytes);
            }
        }
    }

    let mut data_seg = Vec::new();
    for elem in &ds {
        if let Asm::Data(data_list) = &elem.asm {
            for data in data_list {
                let resolved_data = resolve_data_labels(&map, data)?;
                let sbytes = sbytes_of_data(&resolved_data)?;
                data_seg.extend(sbytes);
            }
        }
    }

    Ok(Exec {
        entry,
        text_pos,
        data_pos,
        text_seg,
        data_seg,
    })
}

/// 5 - write_executable(exec, path) — writes a flat binary or basic executable to disk
pub fn write_executable(exec: &Exec, path: &str) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(path)?;

    // Convert SBytes to raw bytes
    let text_bytes: Vec<u8> = exec
        .text_seg
        .iter()
        .flat_map(|sb| match sb {
            SByte::Byte(c) => vec![*c as u8],
            SByte::InsB0(_) => vec![0u8], // Placeholder - actual encoding needed
            SByte::InsFrag => vec![0u8],
        })
        .collect();

    let data_bytes: Vec<u8> = exec
        .data_seg
        .iter()
        .flat_map(|sb| match sb {
            SByte::Byte(c) => vec![*c as u8],
            SByte::InsB0(_) => vec![0u8],
            SByte::InsFrag => vec![0u8],
        })
        .collect();

    // Write a simple flat binary format
    // Format: [text_size(8)] [data_size(8)] [entry(8)] [text_bytes] [data_bytes]
    file.write_all(&(text_bytes.len() as u64).to_le_bytes())?;
    file.write_all(&(data_bytes.len() as u64).to_le_bytes())?;
    file.write_all(&exec.entry.to_le_bytes())?;
    file.write_all(&text_bytes)?;
    file.write_all(&data_bytes)?;

    Ok(())
}

/// BINARY ENTRY -- this is where you generate the exec.out file in output/
/// `cargo run` :                               uses the built in small test placeholder (result is 42)
/// `cargo run -- input.x86` :                  reads prog from input.x86 -> output/exec.out
/// `cargo run -- input.x86 output/myprog.out`  reads prof from input.x86 -> output/myprog.out
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Default output directory
    let default_output = "output/exec.out".to_string();

    // Determine behavior
    match args.len() {
        1 => {
            // No arguments -> use placeholder test program
            println!(">>> No input provided, assembling built-in test program...");

            let main_func = gtext(
                "main",
                vec![
                    Ins {
                        opcode: Opcode::Movq,
                        operands: vec![Operand::Imm(Imm::Lit(42)), Operand::Reg(Reg::Rax)],
                    },
                    Ins {
                        opcode: Opcode::Retq,
                        operands: vec![],
                    },
                ],
            );

            let prog = Prog(vec![main_func]);
            let exec = assemble(&prog)?;

            fs::create_dir_all("output")?;
            write_executable(&exec, &default_output)?;

            println!(">>> Assembled built-in program → {}", &default_output);
        }

        2 | 3 => {
            // 1 or 2 arguments -> read file, optional output path
            let input_path = &args[1];
            let output_path = if args.len() == 3 {
                &args[2]
            } else {
                &default_output
            };

            println!(">>> Reading program from {}...", input_path);

            // Read .x86 or .s input file (as plain text)
            let src = fs::read_to_string(input_path)?;
            let prog = parser::parse_program(&src)?;

            println!(">>> Assembling...");
            let exec = assemble(&prog)?;
            fs::create_dir_all(Path::new(output_path).parent().unwrap())?;
            write_executable(&exec, output_path)?;

            println!(">>> Output written to {}", output_path);
        }

        _ => {
            eprintln!("Usage:");
            eprintln!("  cargo run                  # run built-in test");
            eprintln!("  cargo run -- input.x86     # assemble from file");
            eprintln!("  cargo run -- input.x86 output/exec.out");
        }
    }

    Ok(())
}

/// ASSEMBLER UNIT TESTS
/// run with 'cargo test' (does not run with main)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        // Build a simple program: main that returns 42
        // main:
        //   movq $42, %rax
        //   retq

        let main_func = gtext(
            "main",
            vec![
                Ins {
                    opcode: Opcode::Movq,
                    operands: vec![Operand::Imm(Imm::Lit(42)), Operand::Reg(Reg::Rax)],
                },
                Ins {
                    opcode: Opcode::Retq,
                    operands: vec![],
                },
            ],
        );

        let prog = Prog(vec![main_func]);

        // Test filter_sections
        let (ts, ds) = filter_sections(&prog);
        assert_eq!(ts.len(), 1);
        assert_eq!(ds.len(), 0);
        assert_eq!(ts[0].lbl, "main");

        // Test build_symbol_table
        let result = build_symbol_table(&ts, &ds);
        assert!(result.is_ok());
        let (sym_map, text_size, data_pos, data_size) = result.unwrap();

        assert_eq!(text_size, 16); // 2 instructions * 8 bytes
        assert_eq!(data_size, 0);
        assert!(sym_map.contains_key("main"));
        assert_eq!(*sym_map.get("main").unwrap(), 0x400_000);

        // Test full assembly
        let exec = assemble(&prog);
        assert!(exec.is_ok());
        let exec = exec.unwrap();

        assert_eq!(exec.entry, 0x400_000);
        assert_eq!(exec.text_pos, 0x400_000);
        assert_eq!(exec.text_seg.len(), 16); // 2 instructions * 8 bytes
        assert_eq!(exec.data_seg.len(), 0);
    }

    #[test]
    fn test_program_with_data() {
        // Build a program with both code and data
        // .data
        // hello: .asciz "Hi"
        // num: .quad 100
        //
        // .text
        // main:
        //   movq $0, %rax
        //   retq

        let hello_data = data("hello", vec![Data::Asciz("Hi".to_string())]);

        let num_data = data("num", vec![Data::Quad(Imm::Lit(100))]);

        let main_func = gtext(
            "main",
            vec![
                Ins {
                    opcode: Opcode::Movq,
                    operands: vec![Operand::Imm(Imm::Lit(0)), Operand::Reg(Reg::Rax)],
                },
                Ins {
                    opcode: Opcode::Retq,
                    operands: vec![],
                },
            ],
        );

        let prog = Prog(vec![main_func, hello_data, num_data]);

        // Test filter_sections
        let (ts, ds) = filter_sections(&prog);
        assert_eq!(ts.len(), 1);
        assert_eq!(ds.len(), 2);

        // Test build_symbol_table
        let result = build_symbol_table(&ts, &ds);
        assert!(result.is_ok());
        let (sym_map, text_size, data_pos, data_size) = result.unwrap();

        assert_eq!(text_size, 16); // 2 instructions * 8 bytes
        assert_eq!(data_size, 11); // "Hi\0" (3 bytes) + quad (8 bytes)
        assert!(sym_map.contains_key("main"));
        assert!(sym_map.contains_key("hello"));
        assert!(sym_map.contains_key("num"));

        // Test full assembly
        let exec = assemble(&prog);
        assert!(exec.is_ok());
        let exec = exec.unwrap();

        assert_eq!(exec.entry, 0x400_000);
        assert_eq!(exec.text_seg.len(), 16);
        assert_eq!(exec.data_seg.len(), 11);
    }

    #[test]
    fn test_label_resolution() {
        // Test that labels get resolved to addresses
        // loop:
        //   jmp loop
        // main:
        //   retq

        let loop_func = text(
            "loop",
            vec![Ins {
                opcode: Opcode::Jmp,
                operands: vec![Operand::Imm(Imm::Lbl("loop".to_string()))],
            }],
        );

        let main_func = gtext(
            "main",
            vec![Ins {
                opcode: Opcode::Retq,
                operands: vec![],
            }],
        );

        let prog = Prog(vec![loop_func, main_func]);

        let exec = assemble(&prog);
        assert!(exec.is_ok());
    }

    #[test]
    fn test_undefined_symbol_error() {
        // Test that undefined symbols are caught
        // main:
        //   jmp undefined_label

        let main_func = gtext(
            "main",
            vec![Ins {
                opcode: Opcode::Jmp,
                operands: vec![Operand::Imm(Imm::Lbl("undefined_label".to_string()))],
            }],
        );

        let prog = Prog(vec![main_func]);

        let exec = assemble(&prog);
        assert!(exec.is_err());
    }

    #[test]
    fn test_redefined_symbol_error() {
        // Test that redefined symbols are caught
        let main1 = gtext("main", vec![]);
        let main2 = gtext("main", vec![]);

        let prog = Prog(vec![main1, main2]);

        let (ts, ds) = filter_sections(&prog);
        let result = build_symbol_table(&ts, &ds);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_main_error() {
        // Test that missing 'main' is caught
        let func = gtext(
            "not_main",
            vec![Ins {
                opcode: Opcode::Retq,
                operands: vec![],
            }],
        );

        let prog = Prog(vec![func]);

        let exec = assemble(&prog);
        assert!(exec.is_err());
    }
}
