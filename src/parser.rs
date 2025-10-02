/// Redirection mode
#[derive(Debug, Clone)]
pub enum RedirectMode {
    Write,
    Append,
}

/// Parsed command with arguments and redirections
pub struct ParsedCommand {
    pub args: Vec<String>,
    pub stdout_redirect: Option<(String, RedirectMode)>,
    pub stderr_redirect: Option<(String, RedirectMode)>,
}

/// Parse command line input into arguments and redirections
pub fn parse_arguments(input: &str) -> ParsedCommand {
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

                // Check if it's >> (append) or > (write)
                let is_append = if let Some(&next_ch) = chars.peek() {
                    if next_ch == '>' {
                        chars.next();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                let redirect_mode = if is_append {
                    RedirectMode::Append
                } else {
                    RedirectMode::Write
                };

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
                    stdout_redirect = Some((filename, redirect_mode));
                } else if fd == 2 {
                    stderr_redirect = Some((filename, redirect_mode));
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
