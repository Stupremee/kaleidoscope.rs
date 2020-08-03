//! REPL implementation using [`rustyline`] with auto complete and
//! syntax highlighting.
//!
//! [`rustyline`]: https://docs.rs/rustyline

mod commands;
mod helper;

use self::helper::ReplHelper;
use kaleidoscope::{parse::FrontendDatabase, Compiler};
use rustyline::{error::ReadlineError, Cmd, CompletionType, Config, EditMode, Editor, KeyPress};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

/// The prefix to execute commands.
const PREFIX: char = '.';
const PROMPT: &str = ">> ";

pub struct Repl {
    editor: Editor<ReplHelper>,
    db: Compiler,
    commands: HashMap<&'static str, fn(&mut Repl, &str)>,
}

impl Repl {
    /// Creates a new `Repl` instance and sets up various things like keybinds.
    pub fn new() -> Self {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .tab_stop(4)
            .build();
        let mut editor = Editor::with_config(config);

        let commands = commands::default_commands();

        let helper = ReplHelper::new(commands.keys().copied().collect());
        editor.set_helper(Some(helper));

        editor.bind_sequence(KeyPress::Up, Cmd::LineUpOrPreviousHistory(1));
        editor.bind_sequence(KeyPress::Down, Cmd::LineDownOrNextHistory(1));

        let mut db = Compiler::default();
        db.set_rodeo(Arc::new(Default::default()));
        Self {
            editor,
            db,
            commands,
        }
    }

    fn history_path(&self) -> Option<PathBuf> {
        let mut path = dirs::data_dir()?;
        path.push("kaleidoscope_history");
        Some(path)
    }

    fn save_history(&mut self) -> Option<()> {
        let path = self.history_path()?;
        self.editor.save_history(&path).ok()
    }

    fn load_history(&mut self) -> Option<()> {
        let path = self.history_path()?;
        self.editor.load_history(&path).ok()
    }

    pub fn run(&mut self) -> rustyline::Result<()> {
        self.load_history();

        let version = env!("CARGO_PKG_VERSION");
        println!("Kaleidoscope {}", version);
        let result = loop {
            let line = self.editor.readline(PROMPT);
            match line {
                Ok(line) => self.process_line(line),
                // Ctrl + C will skip and abort the current line.
                Err(ReadlineError::Interrupted) => continue,
                // Ctrl + D will exit the repl
                Err(ReadlineError::Eof) => break Ok(()),
                Err(error) => break Err(error),
            }
        };
        self.save_history();

        result
    }

    fn process_line(&mut self, line: String) {
        self.editor.add_history_entry(line.clone());

        let trimmed_line = line.trim();
        if trimmed_line.starts_with(PREFIX) {
            let name = trimmed_line.split(' ').next().unwrap();

            match self.commands.get(&name[1..]) {
                Some(cmd) => cmd(self, &trimmed_line[name.len()..]),
                None => println!("unknown command '{}'", name),
            }
        } else {
            self.execute_code(line)
        }
    }

    fn execute_code(&mut self, _line: String) {
        println!("executing code coming soon");
    }
}
