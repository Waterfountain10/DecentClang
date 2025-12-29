## Building the driver binary for CLI

build binary (change to -p<name_package> if we change it)

```bash
cargo build -p driver --release
```

attaching the symlink (essentially allowing local use of dclang)

```bash
ln -sf "$PWD/target/release/dclang" /usr/local/bin/dclang
```

now we can test it locally with : 

```bash
dclang --help
```

and later:
```bash
dclang -c hello.c
dclang -S hello.c -o hello.s
dclang hello.c -o hello
```
