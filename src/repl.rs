//! REPL implementation using [`rustyline`] with auto complete and
//! syntax highlighting.
//!
//! [`rustyline`]: https://docs.rs/rustyline

mod helper;

use self::helper::ReplHelper;
use rustyline::{error::ReadlineError, Cmd, CompletionType, Config, EditMode, Editor, KeyPress};
use smol_str::SmolStr;

#[derive(Debug)]
pub struct Repl {
    editor: Editor<ReplHelper>,
    prompt: SmolStr,
}

impl Repl {
    /// Creates a new `Repl` instance and sets up various things like keybinds.
    pub fn new(prompt: impl Into<SmolStr>) -> Self {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .tab_stop(4)
            .build();
        let mut editor = Editor::with_config(config);

        let helper = ReplHelper::default();
        editor.set_helper(Some(helper));

        editor.bind_sequence(KeyPress::Up, Cmd::LineUpOrPreviousHistory(1));
        editor.bind_sequence(KeyPress::Down, Cmd::LineDownOrNextHistory(1));

        Self {
            editor,
            prompt: prompt.into(),
        }
    }

    pub fn run(&mut self) -> rustyline::Result<()> {
        let version = env!("CARGO_PKG_VERSION");
        println!("Kaleidoscope {}", version);
        loop {
            let line = self.editor.readline(&self.prompt);
            match line {
                Ok(line) => self.process_line(line),
                // Ctrl + C will skip and abort the current line.
                Err(ReadlineError::Interrupted) => continue,
                // Ctrl + D will exit the repl
                Err(ReadlineError::Eof) => break Ok(()),
                Err(error) => break Err(error),
            }
        }
    }

    fn process_line(&mut self, line: String) {
        self.editor.add_history_entry(line);
        // todo!();
    }
}
