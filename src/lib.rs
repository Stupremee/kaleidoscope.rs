#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

pub mod error;
pub mod parse;
pub mod source;
pub mod span;

pub use parse::{FrontendDatabase, FrontendDatabaseStorage};
use smol_str::SmolStr;
use source::FileId;
pub use source::{SourceDatabase, SourceDatabaseStorage};
use std::{io, ops::Range, sync::Arc};

pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<FileId>;
pub type Label = codespan_reporting::diagnostic::Label<FileId>;

#[salsa::database(SourceDatabaseStorage, FrontendDatabaseStorage)]
#[derive(Default)]
pub struct Compiler {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Compiler {}

impl Compiler {
    pub fn emit(&self, err: Diagnostic) -> io::Result<()> {
        use codespan_reporting::term::{self, termcolor};

        let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);
        let config = term::Config::default();
        term::emit(&mut stdout, &config, self, &err.into())
    }
}

impl<'a> codespan_reporting::files::Files<'a> for Compiler {
    type FileId = FileId;
    type Name = Arc<SmolStr>;
    type Source = StringRef;

    fn name(&'a self, id: Self::FileId) -> Option<Self::Name> {
        let name = SourceDatabase::name(self, id);
        Some(name)
    }

    fn source(&'a self, id: Self::FileId) -> Option<Self::Source> {
        let source = SourceDatabase::source(self, id);
        Some(StringRef { string: source })
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Option<usize> {
        SourceDatabase::line_index(self, id, byte_index)
    }

    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Option<Range<usize>> {
        SourceDatabase::line_range(self, id, line_index)
    }
}

/// A atomic counted reference to a `String`, which implements `AsRef<str>`
#[derive(Debug)]
pub struct StringRef {
    string: Arc<String>,
}

impl AsRef<str> for StringRef {
    fn as_ref(&self) -> &str {
        self.string.as_ref()
    }
}
