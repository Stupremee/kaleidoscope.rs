use self::{
    ast::{Expr, ExprKind, Identifier},
    token::{Kind, Token, TokenStream},
};
use crate::{
    error::{ParseResult, SyntaxError},
    source::{FileId, SourceDatabase},
    span::{Locatable, Span},
};
use lasso::ThreadedRodeo;
use std::{iter::Peekable, sync::Arc};

pub mod ast;
pub mod token;

#[salsa::query_group(FrontendDatabaseStorage)]
pub trait FrontendDatabase: SourceDatabase {
    #[salsa::input]
    fn rodeo(&self) -> Arc<ThreadedRodeo>;
}

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Parser<'input> {
    tokens: Peekable<TokenStream<'input>>,
    db: &'input dyn FrontendDatabase,
    file: FileId,
    eof_span: Span,
}

impl<'input> Parser<'input> {
    pub fn new(db: &'input dyn FrontendDatabase, code: &'input str, file: FileId) -> Self {
        Self {
            db,
            tokens: TokenStream::new(&code).peekable(),
            file,
            eof_span: Span::new(code.len(), code.len()),
        }
    }

    fn peek(&mut self) -> ParseResult<&Token<'input>> {
        self.tokens.peek().ok_or(Locatable::new(
            SyntaxError::UnexecptedEof,
            self.eof_span,
            self.file,
        ))
    }

    fn next(&mut self) -> ParseResult<Token<'input>> {
        self.tokens.next().ok_or(Locatable::new(
            SyntaxError::UnexecptedEof,
            self.eof_span,
            self.file,
        ))
    }

    fn next_is(&mut self, kind: Kind) -> bool {
        self.peek().map_or(false, |tok| tok.kind == kind)
    }

    fn eat(&mut self, kind: Kind) -> ParseResult<Token<'input>> {
        match self.peek()? {
            token if token.kind == kind => Ok(self.next().unwrap()),
            token => Err(Locatable::new(
                SyntaxError::Expected {
                    expected: kind,
                    found: token.kind,
                },
                token.span,
                self.file,
            )),
        }
    }
}

// Expression parsing methods
impl<'input> Parser<'input> {
    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        todo!()
    }

    fn parse_primary(&mut self) -> ParseResult<Expr> {
        let token = self.peek()?;
        match token.kind {
            Kind::LeftParen => {
                let l_paren = self.next().unwrap().span;
                let expr = self.parse_expr()?;
                let r_paren = self.eat(Kind::RightParen)?.span;
                Ok(Expr {
                    span: l_paren.merge(r_paren),
                    kind: expr.kind,
                })
            }
            Kind::Number => {
                let token = self.next().unwrap();
                let num = token.slice;
                let num = num.parse::<f64>().map_err(|_| {
                    Locatable::new(SyntaxError::InvalidNumber, token.span, self.file)
                })?;
                Ok(Expr {
                    span: token.span,
                    kind: ExprKind::Number(num),
                })
            }
            Kind::Identifier => {
                let token = self.next().unwrap();
                let identifier = self.parse_identifier(&token);

                if !self.next_is(Kind::LeftParen) {
                    return Ok(Expr {
                        span: token.span,
                        kind: ExprKind::Var(identifier),
                    });
                }

                let mut args = Vec::new();
                while self.next_is(Kind::RightParen) {
                    let arg = self.parse_expr()?;
                    args.push(arg);
                    if self.next_is(Kind::Comma) {
                        self.eat(Kind::Comma)?;
                    } else {
                        break;
                    }
                }
                let l_paren = self.eat(Kind::LeftParen)?.span;
                Ok(Expr {
                    span: identifier.span.merge(l_paren),
                    kind: ExprKind::Call {
                        callee: identifier,
                        args,
                    },
                })
            }

            _ => Err(Locatable::new(
                SyntaxError::ExpectedExpression,
                token.span,
                self.file,
            )),
        }
    }

    fn parse_identifier(&mut self, token: &Token<'input>) -> Identifier {
        Identifier {
            spur: self.db.rodeo().get_or_intern(token.slice),
            span: token.span,
        }
    }
}
