use std::{collections::HashSet, io::{stdin, stdout, BufRead, Write}, ops::Range};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ANSI_REGEX: Regex = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap();
    static ref CONTROL_CHARS_REGEX: Regex = Regex::new(r"[\x00-\x1F\x7F]").unwrap();
}

fn get_matches(regex: &Regex, text: &str) -> Vec<Range<usize>> {
    regex.find_iter(text).map(|mat| mat.range()).collect()
}

pub fn string_chunks(text: &str, width: usize) -> Vec<(String, usize)> {
    let mut norm: Vec<bool> = vec![true; text.chars().count()];

    for range in get_matches(&ANSI_REGEX, text) {
        for i in range {
            if let Some(index) = text.char_indices().position(|x| x.0 == i) {
                norm[index] = false;
            }
        }
    }
    for range in get_matches(&CONTROL_CHARS_REGEX, text) {
        for i in range {
            if let Some(index) = text.char_indices().position(|x| x.0 == i) {
                norm[index] = false;
            }
        }
    }

    let mut now_chunk = String::new();
    let mut chunks = Vec::new();
    let mut length = 0;
    
    for (i, b) in norm.iter().enumerate() {
        if *b {
            length += 1;
        }

        now_chunk.push(text.chars().skip(i).next().unwrap());

        if length == width {
            chunks.push((now_chunk.clone(), length));
            now_chunk.clear();
            length = 0;
        }
    }
    if !now_chunk.is_empty() {
        chunks.push((now_chunk.clone(), length));
    }

    chunks
}

pub fn sanitize_text(input: &str) -> String {
    let without_ansi = ANSI_REGEX.replace_all(input, "");
    let cleaned_text = CONTROL_CHARS_REGEX.replace_all(&without_ansi, "");
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