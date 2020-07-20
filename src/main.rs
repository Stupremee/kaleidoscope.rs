#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![warn(clippy::all)]

use smol_str::SmolStr;

const PROMPT: SmolStr = SmolStr::new_inline_from_ascii(3, b">> ");

mod repl;

fn main() {
    let mut repl = repl::Repl::new(PROMPT);
    match repl.run() {
        Ok(_) => {}
        Err(err) => {
            println!("unexpected error occurred: {}", err);
            std::process::exit(1);
        }
    }
}
