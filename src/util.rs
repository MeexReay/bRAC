use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ANSI_REGEX: Regex = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap();
    static ref CONTROL_CHARS_REGEX: Regex = Regex::new(r"[\x00-\x1F\x7F]").unwrap();
}

pub fn sanitize_text(input: &str) -> String {
    let without_ansi = ANSI_REGEX.replace_all(input, "");
    let cleaned_text = CONTROL_CHARS_REGEX.replace_all(&without_ansi, "");
    cleaned_text.into_owned()
}

/// `socks5://user:pass@127.0.0.1:12345/path -> ("127.0.0.1:12345", ("user", "pass"))` \
/// `socks5://127.0.0.1:12345 -> ("127.0.0.1:12345", None)` \
/// `https://127.0.0.1:12345 -> ("127.0.0.1:12345", None)` \
/// `127.0.0.1:12345 -> ("127.0.0.1:12345", None)` \
/// `user:pass@127.0.0.1:12345 -> ("127.0.0.1:12345", ("user", "pass"))`
pub fn parse_socks5_url(url: &str) -> Option<(String, Option<(String, String)>)> {
    let (_, url) = url.split_once("://").unwrap_or(("", url));
    let (url, _) = url.split_once("/").unwrap_or((url, ""));
    if let Some((auth, url)) = url.split_once("@") {
        let (user, pass) = auth.split_once(":")?;
        Some((url.to_string(), Some((user.to_string(), pass.to_string()))))
    } else {
        Some((url.to_string(), None))
    }
}