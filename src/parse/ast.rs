use crate::span::Span;
use lasso::Spur;
use ordered_float::NotNan;

/// An Identifier name is interned using `lasso`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub spur: Spur,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub span: Span,
    pub kind: ItemKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    Function {
        name: Identifier,
        args: Vec<Identifier>,
        body: Box<Expr>,
    },
    Extern {
        name: Identifier,
        args: Vec<Identifier>,
    },
    Operator {
        op: char,
        prec: isize,
        /// True if the operator is binary, false if its a unary op.
        /// The precedence is -1 if it's a unary op
        is_binary: bool,
        body: Box<Expr>,
        args: Vec<Identifier>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Number(NotNan<f64>),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetVar {
    pub name: Identifier,
    pub val: Option<Expr>,
}
