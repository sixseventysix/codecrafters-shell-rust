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
        let (cmd, args) = input.trim().split_once(" ").unwrap();

        match cmd {
            "exit" => { std::process::exit(args.parse::<i32>().unwrap()); }
            "echo" => { println!("{}", args); }
            _ => {
                println!("{}: command not found", input.trim());
            }
        }
    }
}
