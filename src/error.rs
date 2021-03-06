use crate::{
    parse::token::Kind,
    source::{FileCache, FileId},
    span::{Locatable, Span},
    Diagnostic, SourceDatabase,
};
use std::io;

/// A helper macro to generate `Diagnostic`s using a nice dsl.
///
/// # Example
///
/// ```ignore
/// let diagnostic = diagnostic! {
///     error => "some bad error",
///     code: 404,
///     label: primary("message here", file_id, span),
///     note: "some important note here",
/// };
/// ```
macro_rules! diagnostic {
    ($type:ident => $msg:expr, $($tail:tt)*) => {{
        let diagnostic = codespan_reporting::diagnostic::Diagnostic::$type().with_message($msg);
        #[allow(unused_mut)]
        let mut labels = Vec::new();
        #[allow(unused_mut)]
        let mut notes = Vec::<String>::new();

        diagnostic!(@internal, notes, labels, diagnostic, $($tail)*);

        diagnostic.with_labels(labels).with_notes(notes)
    }};

    (@internal, $n:ident, $l:ident, $d:ident, code: $code:expr, $($tail:tt)*) => {
        $d = $d.with_code($code);
        diagnostic!(@internal, $n, $l, $d, $($tail)*);
    };

    (@internal, $n:ident, $l:ident, $d:ident, note: $note:expr, $($tail:tt)*) => {
        $n.push($note.into());
        diagnostic!(@internal, $n, $l, $d, $($tail)*);
    };

    (@internal, $n:ident, $l:ident, $d:ident, label: $type:ident($msg:expr, $file:expr, $range:expr), $($tail:tt)*) => {
        $l.push(codespan_reporting::diagnostic::Label::$type($file, $range).with_message($msg));
        diagnostic!(@internal, $n, $l, $d, $($tail)*);
    };

    (@internal, $n:ident, $l:ident, $d:ident,) => {};
}

/// Represents anything that can be turned into a `Diagnostic` with a given file
/// and span.
pub trait IntoDiagnostic {
    fn into_diagnostic(self, file: FileId, span: Span) -> Diagnostic;
}

/// Any error that can happen while code generation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CompileError {
    UnknownVariable,
    UnknownFunction,
    InvalidArguments { expected: usize, found: usize },
    UnknownOperator,
    InvalidCall,
    InvalidFunctionGenerated,
}

pub type CompileResult<T> = std::result::Result<T, Locatable<CompileError>>;

impl IntoDiagnostic for CompileError {
    fn into_diagnostic(self, file: FileId, span: Span) -> Diagnostic {
        match self {
            CompileError::UnknownVariable => diagnostic! {
                error => "unknown variable",
                label: primary("variable not in scope", file, span),
            },
            CompileError::UnknownFunction => diagnostic! {
                error => "unknown function",
                label: primary("function not in scope", file, span),
            },
            CompileError::UnknownOperator => diagnostic! {
                error => "unknown operator",
                label: primary("operator not in scope", file, span),
            },
            CompileError::InvalidCall => diagnostic! {
                error => "internal error",
                label: primary("invalid call produced", file, span),
            },
            CompileError::InvalidArguments { expected, found } => diagnostic! {
                error => "invalid number of arguments provided",
                label: primary(format!("function takes {} arguments, but only {} were provided", expected, found), file, span),
            },
            CompileError::InvalidFunctionGenerated => diagnostic! {
                error => "invalid function generated",
                label: primary("codegen generated invalid code for this function", file, span),
            },
        }
    }
}

/// Any error that can happen while parsing.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SyntaxError {
    Expected { expected: Kind, found: Kind },
    // This is just for the `for` expression.
    ExpectedOp { expected: char },
    ExpectedOneOf { expected: Vec<Kind>, found: Kind },
    UnexecptedEof,
    ExpectedExpression,
    InvalidNumber,
    InvalidPrecedence,
    InvalidArgs(usize),
}

pub type ParseResult<T> = std::result::Result<T, Locatable<SyntaxError>>;

impl IntoDiagnostic for SyntaxError {
    fn into_diagnostic(self, file: FileId, span: Span) -> Diagnostic {
        match self {
            SyntaxError::Expected { expected, found } => diagnostic! {
                error => "unexpected token",
                label: primary(format!("expected '{}', found '{}'", expected, found), file, span),
            },
            SyntaxError::ExpectedOneOf { expected, found } => {
                let expected = expected
                    .into_iter()
                    .map(|kind| format!("'{}'", kind))
                    .collect::<Vec<_>>()
                    .join(", ");
                diagnostic! {
                    error => "unexpected token",
                    label: primary(format!("expected one of {}, found '{}'", expected, found), file, span),
                }
            }
            SyntaxError::UnexecptedEof => diagnostic! {
                error => "unexpected eof",
                label: primary("unexpected eof here", file, span),
            },
            SyntaxError::ExpectedExpression => diagnostic! {
                error => "expected expression",
                label: primary("expected expression here", file, span),
            },
            SyntaxError::InvalidNumber => diagnostic! {
                error => "invalid number",
                label: primary("is not a valid number", file, span),
            },
            SyntaxError::InvalidPrecedence => diagnostic! {
                error => "invalid precedence",
                label: primary("the operator precedence must be 1..100", file, span),
            },
            SyntaxError::InvalidArgs(expected) => diagnostic! {
                error => "invalid number of arguments",
                label: primary(format!("expected function to have {} arguments", expected), file, span),
            },
            SyntaxError::ExpectedOp { expected } => diagnostic! {
                error => "unexpected operator",
                label: primary(format!("expected '{}'", expected), file, span),
            },
        }
    }
}

impl<T: IntoDiagnostic> Into<Diagnostic> for Locatable<T> {
    fn into(self) -> Diagnostic {
        let (data, span, file) = self.destruct();
        data.into_diagnostic(file, span)
    }
}
pub fn emit(db: &dyn SourceDatabase, err: Diagnostic) -> io::Result<()> {
    use codespan_reporting::term::{self, termcolor};

    let file_cache = FileCache::new(db);
    let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);
    let config = term::Config::default();
    term::emit(&mut stdout, &config, &file_cache, &err.into())
}
