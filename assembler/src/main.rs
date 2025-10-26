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
use x86::*;

/// 1 - `filter_sections(prog: &Prog)` — pure, returns separate text/data sections.
fn filter_sections(prog: &Prog) -> (Vec<Elem>, Vec<Elem>) {
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
fn build_symbol_table(
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

/// 3 - resolve_labels(prog, symtab) — replaces symbolic labels with concrete immediates.
fn resolve_labels(prog : &Prog, symtab)

fn main() {}
