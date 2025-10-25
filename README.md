# DecentClang
Clang but decently made in Rust (originally built in OCaml for cs4212 @ nus)


Dependency to allow modular modifications

```bash
cargo build --workspace
cargo tree -p driver
```
should output this:
```bash
driver v0.1.0 (...)
├── backend v0.1.0 (...)
│   ├── frontend v0.1.0 (...)
│   │   ├── oat v0.1.0 (...)
│   │   └── parser v0.1.0 (...)
│   │       └── lexer v0.1.0 (...)
│   └── x86 v0.1.0 (...)
├── frontend v0.1.0 (...) (*)
└── oat v0.1.0 (...)
```
