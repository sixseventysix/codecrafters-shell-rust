use anyhow::Result;
use std::env;
use crate::output::OutputWriter;
use crate::path::find_in_path;

/// Built-in commands
pub enum BuiltinCommand {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
}

impl BuiltinCommand {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "exit" => Some(Self::Exit),
            "echo" => Some(Self::Echo),
            "type" => Some(Self::Type),
            "pwd" => Some(Self::Pwd),
            "cd" => Some(Self::Cd),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Exit => "exit",
            Self::Echo => "echo",
            Self::Type => "type",
            Self::Pwd => "pwd",
            Self::Cd => "cd",
        }
    }

    pub fn execute(&self, args: &[&str], writer: &mut OutputWriter) -> Result<()> {
        match self {
            Self::Exit => {
                if args.first() == Some(&"0") {
                    std::process::exit(0);
                }
                Ok(())
            }
            Self::Echo => {
                writer.write_stdout(&args.join(" "))
            }
            Self::Type => {
                if let Some(&target) = args.first() {
                    let output = if let Some(builtin) = BuiltinCommand::from_str(target) {
                        format!("{} is a shell builtin", builtin.name())
                    } else if let Some(path) = find_in_path(target) {
                        format!("{} is {}", target, path)
                    } else {
                        format!("{}: not found", target)
                    };
                    writer.write_stdout(&output)?;
                }
                Ok(())
            }
            Self::Pwd => {
                let current_dir = env::current_dir()?;
                writer.write_stdout(&format!("{}", current_dir.display()))
            }
            Self::Cd => {
                if let Some(&path) = args.first() {
                    let target_path = if path == "~" {
                        env::var("HOME").unwrap_or_else(|_| path.to_string())
                    } else {
                        path.to_string()
                    };

                    if let Err(_) = env::set_current_dir(&target_path) {
                        writer.write_stderr(&format!("cd: {}: No such file or directory", path))?;
                    }
                }
                Ok(())
            }
        }
    }
}
