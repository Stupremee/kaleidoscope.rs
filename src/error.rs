use crate::{
    parse::token::Kind,
    source::FileId,
    span::{Locatable, Span},
    Diagnostic,
};

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

/// Any error that can happen while parsing.
#[derive(Debug, Clone)]
pub enum ParseError {
    Expected { expected: Kind, found: Kind },
    ExpectedOneOf { expected: Vec<Kind>, found: Kind },
}

pub type ParseResult<T> = std::result::Result<T, Locatable<ParseError>>;

impl IntoDiagnostic for ParseError {
    fn into_diagnostic(self, file: FileId, span: Span) -> Diagnostic {
        match self {
            ParseError::Expected { expected, found } => {
                diagnostic! {
                    error => "unexpected token",
                    label: primary(format!("expected '{}', found '{}'", expected, found), file, span),
                }
            }
            ParseError::ExpectedOneOf { expected, found } => {
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
        }
    }
}

impl<T: IntoDiagnostic> Locatable<T> {
    fn into_diagnostic(self) -> Diagnostic {
        let (data, span, file) = self.destruct();
        data.into_diagnostic(file, span)
    }
}
