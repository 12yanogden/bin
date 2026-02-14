use regex::Regex;
use std::sync::LazyLock;

static TICKET_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Z]+-\d+").unwrap());

pub fn extract_ticket(branch: &str) -> Option<&str> {
    TICKET_RE.find(branch).map(|m| m.as_str())
}
