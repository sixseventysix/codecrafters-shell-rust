mod builtins;
mod executor;
mod output;
mod parser;
mod path;

use anyhow::Result;
use std::io::{self, Write};

use builtins::BuiltinCommand;
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

fn main() -> Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input)?;

        parse_and_execute(input.trim())?;
    }
}
