use anyhow::Result;
use std::fs::OpenOptions;
use std::os::unix::process::CommandExt;
use std::process::{Command as ProcessCommand, Stdio};

use crate::parser::RedirectMode;
use crate::path::find_in_path;

/// Execute an external command with optional output redirection
pub fn execute_external(
    command: &str,
    args: &[&str],
    stdout_redirect: &Option<(String, RedirectMode)>,
    stderr_redirect: &Option<(String, RedirectMode)>,
) -> Result<()> {
    if let Some(path) = find_in_path(command) {
        let mut cmd = ProcessCommand::new(path);
        cmd.arg0(command).args(args);

        // Set up stdout redirection
        if let Some((stdout_file, mode)) = stdout_redirect {
            let file = match mode {
                RedirectMode::Write => OpenOptions::new().write(true).create(true).truncate(true).open(stdout_file)?,
                RedirectMode::Append => OpenOptions::new().write(true).create(true).append(true).open(stdout_file)?,
            };
            cmd.stdout(Stdio::from(file));
        }

        // Set up stderr redirection
        if let Some((stderr_file, mode)) = stderr_redirect {
            let file = match mode {
                RedirectMode::Write => OpenOptions::new().write(true).create(true).truncate(true).open(stderr_file)?,
                RedirectMode::Append => OpenOptions::new().write(true).create(true).append(true).open(stderr_file)?,
            };
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
