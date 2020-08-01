//! REPL implementation using [`rustyline`] with auto complete and
//! syntax highlighting.
//!
//! [`rustyline`]: https://docs.rs/rustyline

mod commands;
mod helper;

use self::helper::ReplHelper;
use kaleidoscope::{parse::FrontendDatabase, source::File, source::SourceDatabase, Compiler};
use rustyline::{error::ReadlineError, Cmd, CompletionType, Config, EditMode, Editor, KeyPress};
use smol_str::SmolStr;
use std::{collections::HashMap, sync::Arc};

/// The prefix to execute commands.
const PREFIX: char = '.';

pub struct Repl {
    editor: Editor<ReplHelper>,
    prompt: SmolStr,
    db: Compiler,
    commands: HashMap<&'static str, fn(&mut Repl, &str)>,
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

        let commands = commands::default_commands();

        let helper = ReplHelper::new(commands.keys().copied().collect());
        editor.set_helper(Some(helper));

        editor.bind_sequence(KeyPress::Up, Cmd::LineUpOrPreviousHistory(1));
        editor.bind_sequence(KeyPress::Down, Cmd::LineDownOrNextHistory(1));

        let mut db = Compiler::default();
        db.set_rodeo(Arc::new(Default::default()));
        Self {
            editor,
            prompt: prompt.into(),
            db,
            commands,
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

    fn execute_code(&mut self, line: String) {
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
            Err(err) => self.db.emit(err.into()).expect("failed to emit diagnostic"),
        };
    }
}
