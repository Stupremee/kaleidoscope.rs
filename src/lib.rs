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

pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<FileId>;
pub type Label = codespan_reporting::diagnostic::Label<FileId>;

#[salsa::database(SourceDatabaseStorage, FrontendDatabaseStorage)]
#[derive(Default)]
pub struct Compiler {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Compiler {}

impl Compiler {}

// TODO: Add helper methods (parse, parse_expr), that will take a `&str` argument.
