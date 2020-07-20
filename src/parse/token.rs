use logos::Logos;
use std::fmt;

#[derive(Logos, Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    #[regex("#[^\n]*")]
    Comment,

    #[token("def")]
    Def,
    #[token("extern")]
    Extern,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("binary")]
    Binary,
    #[token("unary")]
    Unary,

    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token(",")]
    Comma,

    #[regex("[a-zA-Z][a-zA-Z0-9]*")]
    Identifier,
    #[regex(r"[0-9]*\.?[0-9]+")]
    Number,
    // FIXME: This is probably bad, but that's how Kaleidoscope is made.
    // Probably replace it with a proper regex to only match specific operators.
    #[regex(".", priority = 0)]
    Operator,

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Kind::Comment => "comment",
            Kind::Def => "def",
            Kind::Extern => "extern",
            Kind::If => "if",
            Kind::Then => "then",
            Kind::Else => "else",
            Kind::Binary => "binary",
            Kind::Unary => "unary",
            Kind::LeftParen => "(",
            Kind::RightParen => ")",
            Kind::Comma => ",",
            Kind::Identifier => "identifier",
            Kind::Number => "number",
            Kind::Operator => "operator",
            Kind::Error => "error",
        };
        write!(f, "{}", repr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_assert<S: AsRef<[Kind]>>(input: &str, expected: S) {
        let lex = Kind::lexer(input);
        let kinds = lex.collect::<Vec<_>>();
        assert_eq!(expected.as_ref(), kinds.as_slice())
    }

    #[test]
    fn test_operator() {
        lex_assert("$-+/*", [Kind::Operator].repeat(5));
    }
}
