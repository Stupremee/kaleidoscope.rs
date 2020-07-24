use self::token::{Token, TokenStream};
use crate::source::{FileId, SourceDatabase};
use std::iter::Peekable;

pub mod ast;
pub mod token;

#[salsa::query_group(FrontendDatabaseStorage)]
pub trait FrontendDatabase: SourceDatabase {}

#[derive(Debug, Clone)]
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

    fn peek(&mut self) -> Option<&Token<'input>> {
        self.tokens.peek()
    }

    fn next(&mut self) -> Option<Token<'input>> {
        self.tokens.next()
    }
}

// Expression parsing methods
impl<'input> Parser<'input> {}
