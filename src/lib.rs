#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

pub mod codegen;
pub mod error;
pub mod parse;
pub mod pretty;
pub mod source;
pub mod span;

pub use parse::{FrontendDatabase, FrontendDatabaseStorage};
use source::FileId;
pub use source::{SourceDatabase, SourceDatabaseStorage};
use std::io::Write;

pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<FileId>;
pub type Label = codespan_reporting::diagnostic::Label<FileId>;

#[salsa::database(SourceDatabaseStorage, FrontendDatabaseStorage)]
#[derive(Default)]
pub struct CompilerDatabase {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for CompilerDatabase {}

impl CompilerDatabase {}

// TODO: Add helper methods (parse, parse_expr), that will take a `&str` argument.

macro_rules! print_flush {
    ( $( $x:expr ),* ) => {
        print!( $($x, )* );
        std::io::stdout().flush().expect("Could not flush to standard output.");
    };
}

#[no_mangle]
pub extern "C" fn putchard(x: f64) -> f64 {
    print_flush!("{}", x as u8 as char);
    x
}

#[no_mangle]
pub extern "C" fn printd(x: f64) -> f64 {
    println!("{}", x);
    x
}
