use rustyline::completion::{Completer, Pair};
use rustyline::{Context, Result};

pub struct ShellCompleter {
    builtins: Vec<String>,
}

impl ShellCompleter {
    pub fn new() -> Self {
        Self {
            builtins: vec![
                "echo".to_string(),
                "exit".to_string(),
                "type".to_string(),
                "pwd".to_string(),
                "cd".to_string(),
            ],
        }
    }
}

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>)> {
        let line_before_cursor = &line[..pos];

        // Only complete if we're at the start (completing the command itself)
        if !line_before_cursor.contains(' ') && !line_before_cursor.is_empty() {
            let mut candidates = Vec::new();

            for builtin in &self.builtins {
                if builtin.starts_with(line_before_cursor) {
                    candidates.push(Pair {
                        display: builtin.clone(),
                        replacement: format!("{} ", builtin), // Add space after completion
                    });
                }
            }

            Ok((0, candidates))
        } else {
            Ok((0, vec![]))
        }
    }
}
