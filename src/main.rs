#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;

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

fn main() {
    // Uncomment this block to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        match input.trim() {
            "exit 0" => std::process::exit(0),
            input if input.starts_with("echo ") => println!("{}", &input[5..]),
            input if input.starts_with("type ") => {
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
            }
            input => println!("{}: command not found", input),
        };
    }
}
