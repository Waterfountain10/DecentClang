# DecentClang

[![Made with Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-cargo%20test-success?logo=github&color=green)](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Origin: NUS CS4212](https://img.shields.io/badge/Origin-NUS_CS4212-purple?logo=academia)](https://ilyasergey.net/CS4212/)


> Clang, but decently built in **Rust**.
> 
> Originally coded in **OCaml** for my [CS4212](https://ilyasergey.net/CS4212/) assignments, now rewritten from scratch with a modular backend architecture and clean IR boundaries.

---

## Overview

DecentClang is a small, modular compiler pipeline written in Rust — each stage (lexer → parser → frontend → backend → assembler) is a separate crate. This project rebuilds the same flow with a focus on type-safe systems code.

This started as a university course project, but now became my personal endeavor.

```

DecentClang/
├── lexer/       # tokenize Oat source
├── parser/      # parse into AST
├── oat/         # Oat v1 language spec
├── frontend/    # lower AST → LLVMlite IR
├── llvm/        # LLVMlite IR types
├── backend/     # LLVM IR → x86 lowering
├── x86/         # x86 IR and assembler sugar
├── assembler/   # symbol resolution + exec layout
└── driver/      # CLI entrypoint

````

Each crate compiles independently, with no shared mutable state or unsafe code.  

## Build & Run

Build all crates:
```bash
cargo build --workspace
````

Inspect dependency structure:

```bash
cargo tree -p driver
```

Expected tree:

```
driver v0.1.0
├── backend v0.1.0
│   ├── frontend v0.1.0
│   │   ├── oat v0.1.0
│   │   └── parser v0.1.0
│   │       └── lexer v0.1.0
│   └── x86 v0.1.0
├── frontend v0.1.0 (*)
└── oat v0.1.0
```

Run an Oat example:

```bash
cargo run -p driver -- examples/fact.oat
```
