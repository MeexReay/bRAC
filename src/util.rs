use std::io::{stdin, stdout, BufRead, Write};

use regex::Regex;


pub fn sanitize_text(input: &str) -> String {
    let ansi_regex = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap();
    let control_chars_regex = Regex::new(r"[\x00-\x1F\x7F]").unwrap();
    let without_ansi = ansi_regex.replace_all(input, "");
    let cleaned_text = control_chars_regex.replace_all(&without_ansi, "");
    cleaned_text.into_owned()
}

pub fn get_input(prompt: impl ToString) -> Option<String> {
    let mut out = stdout().lock();
    out.write_all(prompt.to_string().as_bytes()).ok()?;
    out.flush().ok()?;
    let input = stdin().lock().lines().next()
        .map(|o| o.ok())
        .flatten()?;

    if input.is_empty() { 
        None 
    } else { 
        Some(input.to_string())
    }
}