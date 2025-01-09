#[allow(unused_imports)]
use std::io::{self, Write};

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
                    _ => println!("{arg}: not found"),
                }
            }
            input => println!("{}: command not found", input),
        };
    }
}
