mod builtins;
mod completion;
mod executor;
mod output;
mod parser;
mod path;

use anyhow::{Context, Result};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Editor};
use rustyline::Helper;

use builtins::BuiltinCommand;
use completion::ShellCompleter;
use executor::execute_external;
use output::OutputWriter;
use parser::parse_arguments;

fn parse_and_execute(input: &str) -> Result<()> {
    let parsed = parse_arguments(input);
    if parsed.args.is_empty() {
        return Ok(());
    }

    let command = &parsed.args[0];
    let args: Vec<&str> = parsed.args[1..].iter().map(|s| s.as_str()).collect();

    let mut writer = OutputWriter::new(&parsed.stdout_redirect, &parsed.stderr_redirect)?;

    if let Some(builtin) = BuiltinCommand::from_str(command) {
        builtin.execute(&args, &mut writer)?;
    } else {
        execute_external(command, &args, &parsed.stdout_redirect, &parsed.stderr_redirect)?;
    }

    Ok(())
}

// Helper struct to integrate our completer with rustyline
#[derive(rustyline::Helper)]
struct MyHelper {
    completer: ShellCompleter,
}

impl rustyline::completion::Completer for MyHelper {
    type Candidate = rustyline::completion::Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        self.completer.complete(line, pos, ctx)
    }
}

impl rustyline::hint::Hinter for MyHelper {
    type Hint = String;
}

impl rustyline::highlight::Highlighter for MyHelper {}

impl rustyline::validate::Validator for MyHelper {}

fn main() -> Result<()> {
    let helper = MyHelper {
        completer: ShellCompleter::new(),
    };

    let mut rl = DefaultEditor::new().context("Failed to create readline editor")?;
    rl.set_helper(Some(helper));

    loop {
        match rl.readline("$ ") {
            Ok(line) => {
                parse_and_execute(line.trim())?;
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D
                break;
            }
            Err(err) => {
                return Err(err).context("Readline error")?;
            }
        }
    }

    Ok(())
}
