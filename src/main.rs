mod builtins;
mod completion;
mod executor;
mod output;
mod parser;
mod path;

use anyhow::{Context, Result};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

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
#[derive(rustyline::Helper, rustyline::Completer, rustyline::Hinter, rustyline::Validator, rustyline::Highlighter)]
struct MyHelper {
    #[rustyline(Completer)]
    completer: ShellCompleter,
}

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
