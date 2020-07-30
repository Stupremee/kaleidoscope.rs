#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

pub mod error;
pub mod parse;
pub mod source;
pub mod span;

use source::FileId;

pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<FileId>;
pub type Label = codespan_reporting::diagnostic::Label<FileId>;

#[salsa::database(source::SourceDatabaseStorage, parse::FrontendDatabaseStorage)]
#[derive(Default)]
pub struct Compiler {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Compiler {}
