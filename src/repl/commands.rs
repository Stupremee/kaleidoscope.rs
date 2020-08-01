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
    cmds.insert("pretty", pretty_command);
    cmds
}

fn help_command(_repl: &mut Repl, _args: &str) {
    print!(
        "\
Available commands:
    {p}help|h       Shows this message
    {p}pretty       Pretty prints the given expression.
",
        p = super::PREFIX
    )
}

fn pretty_command(repl: &mut Repl, code: &str) {
    let file = File::new(Arc::new("pretty".into()), Arc::new(code.into()));
    let file = repl.db.intern_file(file);

    match repl.db.parse_expr(file) {
        Ok(expr) => {
            let alloc = pretty::Arena::<()>::new();
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            expr.pretty(&alloc, &repl.db.rodeo())
                .1
                .render(50, &mut stdout)
                .expect("failed to pretty print expression");
        }
        Err(err) => repl.db.emit(err.into()).expect("failed to emit diagnostic"),
    };
}
