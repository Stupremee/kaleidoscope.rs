#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![warn(clippy::all)]

mod repl;

use std::{ffi::OsStr, path::PathBuf};

const HELP_MESSAGE: &str = concat!(
    "Kaleidoscope ",
    env!("CARGO_PKG_VERSION"),
    "\n\n",
    "Usage: ",
    env!("CARGO_PKG_NAME"),
    "[FLAGS] [OPTIONS] [<file>]

FLAGS:
    -h, --help
        --emit-ast           If set, the compiler will print the AST. This flag will not affect the REPL.
        --emit-lex           If set, the compiler will print the tokens. This flag will not affect the REPL.
        --emit-ir            If set, the compiler will print generated LLVM IR. This flag will not affect the REPL.

OPTIONS:
    -o, --output             The output file to use. (default: a.out)

ARGS:
        <file>               The input file for the compiler. If no file is specified,
                             the REPL will be started."
);

/// The arguments for the CLI. Parsed by [`pico-args`].
///
/// [`pico-args`]: https://docs.rs/pico-args
#[derive(Debug)]
struct Args {
    /// Pretty prints the parsed AST.
    emit_ast: bool,
    /// Emits the LLVM IR.
    emit_ir: bool,
    /// Emits the lex output.
    emit_lex: bool,
    /// If provided, the file will be compiled.
    /// If no file is provided, the REPL will be started.
    file: Option<PathBuf>,
    /// Place the compiled output in this file.
    output: PathBuf,
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(err) => {
            println!("failed to parse cli arguments: {}", err);
            std::process::exit(1);
        }
    };

    if let Some(_) = args.file {
        todo!()
    } else {
        let mut repl = repl::Repl::new();
        match repl.run() {
            Ok(_) => {}
            Err(err) => {
                println!("unexpected error occurred: {}", err);
                std::process::exit(1);
            }
        }
    }
}

fn os_str_to_path_buf(os_str: &OsStr) -> Result<PathBuf, bool> {
    Ok(os_str.into())
}

fn parse_args() -> Result<Args, pico_args::Error> {
    let mut args = pico_args::Arguments::from_env();
    if args.contains(["-h", "--help"]) {
        println!("{}", HELP_MESSAGE);
        std::process::exit(0);
    }

    let output = args
        .opt_value_from_os_str(["-o", "--output"], os_str_to_path_buf)?
        .unwrap_or_else(|| "a.out".into());
    let file = args.free_from_os_str(os_str_to_path_buf)?;

    Ok(Args {
        emit_ast: args.contains("--emit-ast"),
        emit_ir: args.contains("--emit-ir"),
        emit_lex: args.contains("--emit-lex"),
        file,
        output,
    })
}
