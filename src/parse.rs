use self::{
    ast::{Expr, ExprKind, Identifier, LetVar},
    token::{Kind, Token, TokenStream},
};
use crate::{
    error::{ParseResult, SyntaxError},
    source::{FileId, SourceDatabase},
    span::{Locatable, Span},
};
use lasso::ThreadedRodeo;
use std::{collections::HashMap, iter::Peekable, sync::Arc};

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
    rodeo: Arc<ThreadedRodeo>,
    file: FileId,
    eof_span: Span,
    operators: HashMap<char, i32>,
}

impl<'input> Parser<'input> {
    pub fn new(rodeo: Arc<ThreadedRodeo>, code: &'input str, file: FileId) -> Self {
        let mut operators = HashMap::new();

        operators.insert('=', 2);
        operators.insert('<', 10);
        operators.insert('+', 20);
        operators.insert('-', 20);
        operators.insert('*', 40);
        operators.insert('/', 40);

        Self {
            rodeo,
            tokens: TokenStream::new(&code).peekable(),
            file,
            eof_span: Span::new(code.len(), code.len()),
            operators,
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
        let lhs = self.parse_unary()?;
        self.parse_bin_op(0, lhs)
    }

    fn token_precendence(&mut self) -> i32 {
        let token = if let Ok(Token {
            kind: Kind::Operator,
            slice,
            ..
        }) = self.peek()
        {
            slice.chars().next().unwrap()
        } else {
            return -1;
        };
        self.operators.get(&token).copied().unwrap_or(-1)
    }

    fn parse_bin_op(&mut self, prec: i32, mut lhs: Expr) -> ParseResult<Expr> {
        loop {
            let token_prec = self.token_precendence();
            if token_prec < prec {
                return Ok(lhs);
            }

            let bin_op = match self.eat(Kind::Operator)? {
                Token {
                    kind: Kind::Operator,
                    slice,
                    ..
                } => slice.chars().next().unwrap(),
                _ => unreachable!(),
            };
            let mut rhs = self.parse_unary()?;

            let next_prec = self.token_precendence();
            if token_prec < next_prec {
                rhs = self.parse_bin_op(token_prec + 1, rhs)?;
            }

            lhs = Expr {
                span: lhs.span.merge(rhs.span),
                kind: ExprKind::Binary {
                    left: Box::new(lhs),
                    op: bin_op,
                    right: Box::new(rhs),
                },
            }
        }
    }

    fn parse_unary(&mut self) -> ParseResult<Expr> {
        if !self.next_is(Kind::Operator) {
            return self.parse_primary();
        }
        let op = self.eat(Kind::Operator)?;
        let val = self.parse_unary()?;
        Ok(Expr {
            span: op.span.merge(val.span),
            kind: ExprKind::Unary {
                op: op.slice.chars().next().unwrap(),
                val: Box::new(val),
            },
        })
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
                let identifier = self.intern_identifier(&token);

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
            Kind::If => {
                let if_span = self.next().unwrap().span;
                let cond = self.parse_expr()?;
                self.eat(Kind::Then)?;
                let then = self.parse_expr()?;
                self.eat(Kind::Else)?;
                let else_ = self.parse_expr()?;
                Ok(Expr {
                    span: if_span.merge(else_.span),
                    kind: ExprKind::If {
                        cond: Box::new(cond),
                        then: Box::new(then),
                        else_: Box::new(else_),
                    },
                })
            }
            Kind::For => {
                let for_span = self.next().unwrap().span;
                let name = self.eat(Kind::Identifier)?;
                let name = self.intern_identifier(&name);
                self.eat(Kind::Equal)?;
                let start = self.parse_expr()?;
                self.eat(Kind::Comma)?;
                let end = self.parse_expr()?;

                let step = if let Ok(_) = self.eat(Kind::Comma) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.eat(Kind::In)?;
                let body = self.parse_expr()?;
                Ok(Expr {
                    span: for_span.merge(body.span),
                    kind: ExprKind::For {
                        var: name,
                        start: Box::new(start),
                        end: Box::new(end),
                        step: step.map(Box::new),
                        body: Box::new(body),
                    },
                })
            }
            Kind::Var => {
                let var_span = self.next().unwrap().span;

                let mut vars = Vec::new();
                loop {
                    let name = self.eat(Kind::Identifier)?;
                    let name = self.intern_identifier(&name);

                    let init = if self.next_is(Kind::Equal) {
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };

                    vars.push(LetVar { name, val: init });

                    if !self.next_is(Kind::Comma) {
                        break;
                    }
                    self.eat(Kind::Comma)?;
                }

                self.eat(Kind::In)?;
                let body = self.parse_expr()?;
                Ok(Expr {
                    span: var_span.merge(body.span),
                    kind: ExprKind::Let {
                        vars,
                        body: Box::new(body),
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

    fn intern_identifier(&mut self, token: &Token<'input>) -> Identifier {
        Identifier {
            spur: self.rodeo.get_or_intern(token.slice),
            span: token.span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert(code: &str) {
        let rodeo = Arc::new(ThreadedRodeo::new());
        let mut parser = Parser::new(rodeo, code, FileId::default());
        let expr = parser.parse_expr().unwrap();
        eprintln!("result: {:#?}", expr);
        panic!();
    }

    #[test]
    fn parse_expr() {
        assert("1 + 1");
    }
}
