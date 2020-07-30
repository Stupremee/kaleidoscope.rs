//! REPL implementation using [`rustyline`] with auto complete and
//! syntax highlighting.
//!
//! [`rustyline`]: https://docs.rs/rustyline

mod helper;

use self::helper::ReplHelper;
use kaleidoscope::{
    parse::FrontendDatabase,
    source::SourceDatabase,
    source::{emit, File},
    Compiler,
};
use rustyline::{error::ReadlineError, Cmd, CompletionType, Config, EditMode, Editor, KeyPress};
use smol_str::SmolStr;
use std::sync::Arc;

pub struct Repl {
    editor: Editor<ReplHelper>,
    prompt: SmolStr,
    db: Compiler,
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
            db: Compiler::default(),
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
        self.editor.add_history_entry(line.clone());
        let name = Arc::new(SmolStr::new("repl"));
        let source = Arc::new(line);
        let file = File::new(name, source);
        let file = self.db.intern_file(file);

        match self.db.parse(file) {
            Ok(items) => {
                for item in items {
                    println!("=> {:#?}", item);
                }
            }
            Err(err) => {
                use codespan_reporting::term::{self, termcolor};

                let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);
                let config = term::Config::default();
                term::emit(&mut stdout, &config, &self.db, &err.into())
                    .expect("failed to emit diagnostic");
            }
        }
    }
}
