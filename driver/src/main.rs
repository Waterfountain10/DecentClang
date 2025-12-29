use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Exe, // compile + link
    Obj, // -c
    Asm, // -S (emit assembly)
}

#[derive(Debug)]
struct Options {
    mode: Mode,
    output: Option<PathBuf>,
    inputs: Vec<PathBuf>,
    include_dirs: Vec<String>,
    defines: Vec<String>,
    lib_dirs: Vec<String>,
    libs: Vec<String>,
    verbose: bool,
}

fn print_help() {
    println!(
        r#"dclang - DecentClang driver (scaffold)

USAGE:
  dclang [options] <input.c>...

MODES:
  (default)          Compile + link to an executable (a.out)
  -c                 Compile only; emit object file (.o)
  -S                 Compile only; emit assembly (.s)

OPTIONS:
  -o <file>          Write output to <file>
  -I<dir> / -I <dir> Add include directory
  -D<name[=val]>     Define macro (stored for later)
  -L<dir>            Add library search directory
  -l<name>           Link with library (stored for later)
  -v                 Verbose
  --help             Show this help
  --version          Show version
"#
    );
}

fn print_version() {
    println!("dclang 0.1.0 (first edition yay");
}

fn default_output_for(input: &Path, mode: Mode) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    match mode {
        Mode::Obj => PathBuf::from(format!("{stem}.o")),
        Mode::Asm => PathBuf::from(format!("{stem}.s")),
        Mode::Exe => PathBuf::from("a.out"),
    }
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut opt = Options {
        mode: Mode::Exe,
        output: None,
        inputs: vec![],
        include_dirs: vec![],
        defines: vec![],
        lib_dirs: vec![],
        libs: vec![],
        verbose: false,
    };

    let mut i = 1;
    while i < args.len() {
        let a = &args[i];

        match a.as_str() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                print_version();
                std::process::exit(0);
            }
            "-v" => opt.verbose = true,
            "-c" => opt.mode = Mode::Obj,
            "-S" => opt.mode = Mode::Asm,
            "-o" => {
                i += 1;
                if i >= args.len() {
                    return Err("expected filename after -o".into());
                }
                opt.output = Some(PathBuf::from(&args[i]));
            }
            _ if a.starts_with("-I") => {
                let val = if a.len() > 2 {
                    a[2..].to_string()
                } else {
                    i += 1;
                    if i >= args.len() {
                        return Err("expected dir after -I".into());
                    }
                    args[i].clone()
                };
                opt.include_dirs.push(val);
            }
            _ if a.starts_with("-D") => {
                let val = a[2..].to_string();
                if val.is_empty() {
                    return Err("expected macro after -D".into());
                }
                opt.defines.push(val);
            }
            _ if a.starts_with("-L") => {
                let val = if a.len() > 2 {
                    a[2..].to_string()
                } else {
                    i += 1;
                    if i >= args.len() {
                        return Err("expected dir after -L".into());
                    }
                    args[i].clone()
                };
                opt.lib_dirs.push(val);
            }
            _ if a.starts_with("-l") => {
                let val = a[2..].to_string();
                if val.is_empty() {
                    return Err("expected lib name after -l".into());
                }
                opt.libs.push(val);
            }
            _ if a.starts_with('-') => {
                // Keep unknown flags instead of failing hard (clang-style permissive driver).
                // we will later route them and validate (if they are accidental typos)
                if opt.verbose {
                    eprintln!("[dclang] note: ignoring unknown flag: {a}");
                }
            }
            _ => {
                // Input file
                opt.inputs.push(PathBuf::from(a));
            }
        }

        i += 1;
    }

    if opt.inputs.is_empty() {
        return Err("no input files".into());
    }
    if opt.inputs.len() > 1 && opt.mode != Mode::Exe {
        // Maybe relax later; compatibility for multiple inputs
        return Err("multiple inputs supported only in default (link) mode for now".into());
    }

    Ok(opt)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let opt = match parse_args(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("dclang: error: {e}");
            eprintln!("run `dclang --help` for usage");
            std::process::exit(2);
        }
    };

    // Determine output if not explicitly set
    let output = opt.output.clone().unwrap_or_else(|| {
        if opt.mode == Mode::Exe {
            PathBuf::from("a.out")
        } else {
            default_output_for(&opt.inputs[0], opt.mode)
        }
    });

    // Scaffold behavior: print the pipeline steps
    if opt.verbose {
        eprintln!("[dclang] options: {opt:#?}");
    }

    let input_list: Vec<String> = opt.inputs.iter().map(|p| p.display().to_string()).collect();

    match opt.mode {
        Mode::Obj => {
            println!("[dclang] compile -> object");
            println!("  inputs : {}", input_list.join(", "));
            println!("  output : {}", output.display());
            println!("  (stub) would: parse -> typecheck -> lower -> codegen obj");
        }
        Mode::Asm => {
            println!("[dclang] compile -> assembly");
            println!("  inputs : {}", input_list.join(", "));
            println!("  output : {}", output.display());
            println!("  (stub) would: parse -> typecheck -> lower -> emit .s");
        }
        Mode::Exe => {
            println!("[dclang] compile + link -> executable");
            println!("  inputs : {}", input_list.join(", "));
            println!("  output : {}", output.display());
            println!("  (stub) would: compile each -> link -> exe");
            if !opt.libs.is_empty() || !opt.lib_dirs.is_empty() {
                println!(
                    "  libs   : -L{} -l{}",
                    opt.lib_dirs.join(" -L"),
                    opt.libs.join(" -l")
                );
            }
        }
    }
}
