use anyhow::Result;
use std::io::{self, Write};
use std::env;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

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

fn main() -> Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();

        if input == "exit 0" {
            std::process::exit(0);
        } else if input.starts_with("echo ") {
            println!("{}", &input[5..]);
        } else if input.starts_with("type ") {
            let arg = &input[5..];
            match arg {
                "exit" | "echo" | "type" => println!("{arg} is a shell builtin"),
                _ => {
                    if let Some(path) = find_in_path(arg) {
                        println!("{arg} is {}", path);
                    } else {
                        println!("{arg}: not found");
                    }
                }
            }
        } else {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let command = parts[0];
            let args = &parts[1..];

            if let Some(path) = find_in_path(command) {
                let output = Command::new(path)
                    .args(args)
                    .output()?;

                print!("{}", String::from_utf8_lossy(&output.stdout));
                print!("{}", String::from_utf8_lossy(&output.stderr));
            } else {
                println!("{}: command not found", command);
            }
        }
    }
}
