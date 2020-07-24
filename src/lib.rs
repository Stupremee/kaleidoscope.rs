#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![warn(clippy::all)]

pub mod error;
pub mod parse;
pub mod source;
pub mod span;

use source::FileId;

pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<FileId>;
pub type Label = codespan_reporting::diagnostic::Label<FileId>;
