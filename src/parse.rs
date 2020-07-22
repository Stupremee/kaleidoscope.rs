use self::token::TokenStream;
use crate::source::{FileId, SourceDatabase};
use std::{fmt, iter::Peekable};

pub mod ast;
mod token;

#[salsa::query_group(FrontendDatabaseStorage)]
pub trait FrontendDatabase: SourceDatabase {}

#[derive(Clone)]
pub struct Parser<'input> {
    tokens: Peekable<TokenStream<'input>>,
    file: FileId,
}

impl<'input> Parser<'input> {
    fn new(code: &'input str, file: FileId) -> Self {
        Self {
            tokens: TokenStream::new(code).peekable(),
            file,
        }
    }
}

impl fmt::Debug for Parser<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parser")
            .field("tokens", &self.tokens)
            .field("file", &self.file)
            .finish()
    }
}
