use ansi_term::Style;
use rustyline::{
    completion::Completer,
    highlight::{Highlighter, MatchingBracketHighlighter},
    hint::Hinter,
    line_buffer::LineBuffer,
    validate::{MatchingBracketValidator, ValidationContext, ValidationResult, Validator},
    Context,
};
use rustyline_derive::Helper;
use std::{borrow::Cow, marker::PhantomData};

#[derive(Helper)]
pub(super) struct ReplHelper {
    highlighter: MatchingBracketHighlighter,
    _priv: PhantomData<Self>,
}

impl Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        let prompt = Style::new().bold().paint(prompt);
        Cow::Owned(prompt.to_string())
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        let hint = Style::new().dimmed().paint(hint);
        Cow::Owned(hint.to_string())
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Completer for ReplHelper {
    type Candidate = String;

    // TODO: Complete function names, etc.
}

impl Hinter for ReplHelper {
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        // TODO: Hint commands
        let _ = (line, pos, ctx);
        None
    }
}

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        let mut stack = vec![];
        for c in input.chars() {
            match c {
                '(' | '[' | '{' => stack.push(c),
                ')' | ']' | '}' => match (stack.pop(), c) {
                    (Some('('), ')') | (Some('['), ']') | (Some('{'), '}') => {}
                    (_, _) => {
                        return Ok(ValidationResult::Invalid(Some(
                            "unclosed bracket".to_string(),
                        )));
                    }
                },
                _ => continue,
            }
        }

        if stack.is_empty() {
            Ok(ValidationResult::Valid(None))
        } else {
            Ok(ValidationResult::Incomplete)
        }
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}
