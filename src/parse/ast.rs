use crate::span::Span;
use lasso::Spur;

/// An Identifier name is interned using `lasso`.
#[derive(Debug, Clone)]
pub struct Identifier {
    pub spur: Spur,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Prototype {
    pub name: Identifier,
    pub args: Vec<Identifier>,
    pub is_op: bool,
    pub prec: usize,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub proto: Prototype,
    /// If body is `None`, its an `extern` function.
    pub body: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Number(f64),
    Var(Identifier),
    Unary {
        op: char,
        val: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: char,
        right: Box<Expr>,
    },
    Call {
        callee: Identifier,
        args: Vec<Expr>,
    },
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
    For {
        var: Identifier,
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
        body: Box<Expr>,
    },
    /// The var / in expression
    Let {
        vars: Vec<LetVar>,
        body: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub struct LetVar {
    pub name: Identifier,
    pub val: Option<Expr>,
}
