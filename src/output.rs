use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;
use crate::parser::RedirectMode;

/// Output writer that handles redirection
pub struct OutputWriter {
    stdout_file: Option<std::fs::File>,
    stderr_file: Option<std::fs::File>,
}

impl OutputWriter {
    pub fn new(
        stdout_redirect: &Option<(String, RedirectMode)>,
        stderr_redirect: &Option<(String, RedirectMode)>,
    ) -> Result<Self> {
        let stdout_file = if let Some((path, mode)) = stdout_redirect {
            let file = match mode {
                RedirectMode::Write => OpenOptions::new().write(true).create(true).truncate(true).open(path)?,
                RedirectMode::Append => OpenOptions::new().write(true).create(true).append(true).open(path)?,
            };
            Some(file)
        } else {
            None
        };

        let stderr_file = if let Some((path, mode)) = stderr_redirect {
            let file = match mode {
                RedirectMode::Write => OpenOptions::new().write(true).create(true).truncate(true).open(path)?,
                RedirectMode::Append => OpenOptions::new().write(true).create(true).append(true).open(path)?,
            };
            Some(file)
        } else {
            None
        };

        Ok(Self {
            stdout_file,
            stderr_file,
        })
    }

    pub fn write_stdout(&mut self, content: &str) -> Result<()> {
        if let Some(ref mut file) = self.stdout_file {
            writeln!(file, "{}", content)?;
        } else {
            println!("{}", content);
        }
        Ok(())
    }

    pub fn write_stderr(&mut self, content: &str) -> Result<()> {
        if let Some(ref mut file) = self.stderr_file {
            writeln!(file, "{}", content)?;
        } else {
            eprintln!("{}", content);
        }
        Ok(())
    }
}
