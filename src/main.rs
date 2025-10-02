use anyhow::Result;
use std::io::{self, Write};
use std::env;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command as ProcessCommand, Stdio};
use std::os::unix::process::CommandExt;
use std::fs::File;

// Output writer that handles redirection
struct OutputWriter {
    stdout_file: Option<File>,
    stderr_file: Option<File>,
}

impl OutputWriter {
    fn new(stdout_redirect: &Option<String>, stderr_redirect: &Option<String>) -> Result<Self> {
        let stdout_file = if let Some(path) = stdout_redirect {
            Some(File::create(path)?)
        } else {
            None
        };

        let stderr_file = if let Some(path) = stderr_redirect {
            Some(File::create(path)?)
        } else {
            None
        };

        Ok(Self {
            stdout_file,
            stderr_file,
        })
    }

    fn write_stdout(&mut self, content: &str) -> Result<()> {
        if let Some(ref mut file) = self.stdout_file {
            writeln!(file, "{}", content)?;
        } else {
            println!("{}", content);
        }
        Ok(())
    }

    fn write_stderr(&mut self, content: &str) -> Result<()> {
        if let Some(ref mut file) = self.stderr_file {
            writeln!(file, "{}", content)?;
        } else {
            eprintln!("{}", content);
        }
        Ok(())
    }
}

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

    fn execute(&self, args: &[&str], writer: &mut OutputWriter) -> Result<()> {
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

fn execute_external(
    command: &str,
    args: &[&str],
    stdout_redirect: &Option<String>,
    stderr_redirect: &Option<String>,
) -> Result<()> {
    if let Some(path) = find_in_path(command) {
        let mut cmd = ProcessCommand::new(path);
        cmd.arg0(command).args(args);

        // Set up stdout redirection
        if let Some(stdout_file) = stdout_redirect {
            let file = File::create(stdout_file)?;
            cmd.stdout(Stdio::from(file));
        }

        // Set up stderr redirection
        if let Some(stderr_file) = stderr_redirect {
            let file = File::create(stderr_file)?;
            cmd.stderr(Stdio::from(file));
        }

        let output = cmd.output()?;

        // Only print if not redirected
        if stdout_redirect.is_none() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if stderr_redirect.is_none() {
            print!("{}", String::from_utf8_lossy(&output.stderr));
        }
    } else {
        println!("{}: command not found", command);
    }
    Ok(())
}

struct ParsedCommand {
    args: Vec<String>,
    stdout_redirect: Option<String>,
    stderr_redirect: Option<String>,
}

fn parse_arguments(input: &str) -> ParsedCommand {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut stdout_redirect = None;
    let mut stderr_redirect = None;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' if in_single_quote => {
                // Inside single quotes, backslash is literal
                current_arg.push(ch);
            }
            '\\' if in_double_quote => {
                // Inside double quotes, backslash only escapes special characters
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        '"' | '\\' | '$' | '`' | '\n' => {
                            // These characters can be escaped inside double quotes
                            chars.next();
                            current_arg.push(next_ch);
                        }
                        _ => {
                            // For other characters, backslash is literal
                            current_arg.push(ch);
                        }
                    }
                } else {
                    current_arg.push(ch);
                }
            }
            '\\' => {
                // Outside quotes, backslash escapes the next character
                if let Some(next_ch) = chars.next() {
                    current_arg.push(next_ch);
                } else {
                    current_arg.push(ch);
                }
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '>' if !in_single_quote && !in_double_quote => {
                // Handle redirection
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }

                // Check if it's 1> or 2>
                let fd = if let Some(last_arg) = args.last() {
                    if last_arg == "1" {
                        args.pop();
                        1
                    } else if last_arg == "2" {
                        args.pop();
                        2
                    } else {
                        1 // default to stdout
                    }
                } else {
                    1 // default to stdout
                };

                // Skip whitespace after >
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == ' ' || next_ch == '\t' {
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Parse the filename
                let mut filename = String::new();
                let mut file_in_single_quote = false;
                let mut file_in_double_quote = false;

                while let Some(ch) = chars.next() {
                    match ch {
                        '\\' if file_in_single_quote => {
                            filename.push(ch);
                        }
                        '\\' if file_in_double_quote => {
                            if let Some(&next_ch) = chars.peek() {
                                match next_ch {
                                    '"' | '\\' | '$' | '`' | '\n' => {
                                        chars.next();
                                        filename.push(next_ch);
                                    }
                                    _ => {
                                        filename.push(ch);
                                    }
                                }
                            } else {
                                filename.push(ch);
                            }
                        }
                        '\\' => {
                            if let Some(next_ch) = chars.next() {
                                filename.push(next_ch);
                            } else {
                                filename.push(ch);
                            }
                        }
                        '\'' if !file_in_double_quote => {
                            file_in_single_quote = !file_in_single_quote;
                        }
                        '"' if !file_in_single_quote => {
                            file_in_double_quote = !file_in_double_quote;
                        }
                        ' ' | '\t' if !file_in_single_quote && !file_in_double_quote => {
                            break;
                        }
                        _ => {
                            filename.push(ch);
                        }
                    }
                }

                if fd == 1 {
                    stdout_redirect = Some(filename);
                } else if fd == 2 {
                    stderr_redirect = Some(filename);
                }
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

    ParsedCommand {
        args,
        stdout_redirect,
        stderr_redirect,
    }
}

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
