use anyhow::Result;
use std::io::{self, Write};
use std::env;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::process::Command as ProcessCommand;
use std::os::unix::process::CommandExt;

// Built-in commands
enum BuiltinCommand {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
}

impl BuiltinCommand {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "exit" => Some(Self::Exit),
            "echo" => Some(Self::Echo),
            "type" => Some(Self::Type),
            "pwd" => Some(Self::Pwd),
            "cd" => Some(Self::Cd),
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Exit => "exit",
            Self::Echo => "echo",
            Self::Type => "type",
            Self::Pwd => "pwd",
            Self::Cd => "cd",
        }
    }

    fn execute(&self, args: &[&str]) -> Result<()> {
        match self {
            Self::Exit => {
                if args.first() == Some(&"0") {
                    std::process::exit(0);
                }
                Ok(())
            }
            Self::Echo => {
                println!("{}", args.join(" "));
                Ok(())
            }
            Self::Type => {
                if let Some(&target) = args.first() {
                    if let Some(builtin) = BuiltinCommand::from_str(target) {
                        println!("{} is a shell builtin", builtin.name());
                    } else if let Some(path) = find_in_path(target) {
                        println!("{} is {}", target, path);
                    } else {
                        println!("{}: not found", target);
                    }
                }
                Ok(())
            }
            Self::Pwd => {
                let current_dir = env::current_dir()?;
                println!("{}", current_dir.display());
                Ok(())
            }
            Self::Cd => {
                if let Some(&path) = args.first() {
                    let target_path = if path == "~" {
                        env::var("HOME").unwrap_or_else(|_| path.to_string())
                    } else {
                        path.to_string()
                    };

                    if let Err(_) = env::set_current_dir(&target_path) {
                        println!("cd: {}: No such file or directory", path);
                    }
                }
                Ok(())
            }
        }
    }
}

fn find_in_path(command: &str) -> Option<String> {
    let path_env = env::var("PATH").ok()?;

    for dir in path_env.split(':') {
        let full_path = Path::new(dir).join(command);

        if let Ok(metadata) = full_path.metadata() {
            if metadata.is_file() {
                let permissions = metadata.permissions();
                if permissions.mode() & 0o111 != 0 {
                    return Some(full_path.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}

fn execute_external(command: &str, args: &[&str]) -> Result<()> {
    if let Some(path) = find_in_path(command) {
        let output = ProcessCommand::new(path)
            .arg0(command)
            .args(args)
            .output()?;

        print!("{}", String::from_utf8_lossy(&output.stdout));
        print!("{}", String::from_utf8_lossy(&output.stderr));
    } else {
        println!("{}: command not found", command);
    }
    Ok(())
}

fn parse_arguments(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(ch);
            }
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}

fn parse_and_execute(input: &str) -> Result<()> {
    let parts = parse_arguments(input);
    if parts.is_empty() {
        return Ok(());
    }

    let command = &parts[0];
    let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

    if let Some(builtin) = BuiltinCommand::from_str(command) {
        builtin.execute(&args)?;
    } else {
        execute_external(command, &args)?;
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
