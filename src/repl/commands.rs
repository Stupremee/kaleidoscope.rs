//! Commands for the REPL.

use super::Repl;
use kaleidoscope::{
    parse::FrontendDatabase,
    pretty::Pretty,
    source::{File, SourceDatabase},
};
use std::{collections::HashMap, sync::Arc};

pub fn default_commands() -> HashMap<&'static str, fn(&mut Repl, &str)> {
    let mut cmds = HashMap::<&'static str, fn(&mut Repl, &str)>::new();
    cmds.insert("help", help_command);
    cmds.insert("h", help_command);
    cmds.insert("ast", ast_command);
    cmds
}

fn help_command(_repl: &mut Repl, _args: &str) {
    print!(
        "\
Available commands:
    {p}help|h       Shows this message
    {p}ast          Pretty prints the parsed AST.
",
        p = super::PREFIX
    )
}

fn ast_command(repl: &mut Repl, code: &str) {
    let file = File::new(Arc::new("pretty".into()), Arc::new(code.into()));
    let file = repl.db.intern_file(file);

    match repl.db.parse(file) {
        Ok(items) => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            for item in items {
                println!("=>");
                let alloc = pretty::Arena::<()>::new();
                item.pretty(&alloc, &repl.db.rodeo())
                    .1
                    .render(50, &mut stdout)
                    .expect("failed to pretty print item");
                println!();
            }
        }
        Err(err) => repl.db.emit(err.into()).expect("failed to emit diagnostic"),
    };
}
