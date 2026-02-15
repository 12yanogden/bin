use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

/// Resolve a CLI argument into a Vec<String>.
/// - `"-"` reads lines from stdin.
/// - An existing file path reads lines from the file.
/// - Otherwise, splits on whitespace.
pub fn resolve_input(arg: &str, stdin_used: &mut bool) -> Result<Vec<String>, String> {
    if arg == "-" {
        if *stdin_used {
            return Err("only one argument may be '-' (stdin)".to_string());
        }
        *stdin_used = true;
        let stdin = io::stdin();
        let lines: Vec<String> = stdin
            .lock()
            .lines()
            .map(|l| l.map_err(|e| format!("failed to read stdin: {}", e)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(lines)
    } else if Path::new(arg).is_file() {
        let content =
            fs::read_to_string(arg).map_err(|e| format!("failed to read file '{}': {}", arg, e))?;
        Ok(content.lines().map(|l| l.to_string()).collect())
    } else {
        Ok(arg.split_whitespace().map(|s| s.to_string()).collect())
    }
}

/// Return elements present in both `a` and `b`, deduplicated, preserving order from `a`.
pub fn intersect(a: &[String], b: &[String]) -> Vec<String> {
    let set_b: HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in a {
        if set_b.contains(item.as_str()) && seen.insert(item.as_str()) {
            result.push(item.clone());
        }
    }
    result
}

/// Return elements in `a` that are not in `b`, deduplicated, preserving order from `a`.
pub fn subtract(a: &[String], b: &[String]) -> Vec<String> {
    let set_b: HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in a {
        if !set_b.contains(item.as_str()) && seen.insert(item.as_str()) {
            result.push(item.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect_basic() {
        let a = vec!["a".into(), "b".into(), "c".into()];
        let b = vec!["b".into(), "c".into(), "d".into()];
        assert_eq!(intersect(&a, &b), vec!["b", "c"]);
    }

    #[test]
    fn subtract_basic() {
        let a = vec!["a".into(), "b".into(), "c".into()];
        let b = vec!["b".into(), "c".into(), "d".into()];
        assert_eq!(subtract(&a, &b), vec!["a"]);
    }

    #[test]
    fn both_empty() {
        let a: Vec<String> = vec![];
        let b: Vec<String> = vec![];
        assert_eq!(intersect(&a, &b), Vec::<String>::new());
        assert_eq!(subtract(&a, &b), Vec::<String>::new());
    }

    #[test]
    fn no_overlap() {
        let a = vec!["a".into(), "b".into()];
        let b = vec!["c".into(), "d".into()];
        assert_eq!(intersect(&a, &b), Vec::<String>::new());
        assert_eq!(subtract(&a, &b), vec!["a", "b"]);
    }

    #[test]
    fn deduplicates_first_input() {
        let a = vec!["a".into(), "b".into(), "a".into(), "b".into()];
        let b = vec!["a".into(), "b".into()];
        assert_eq!(intersect(&a, &b), vec!["a", "b"]);
    }

    #[test]
    fn deduplicates_subtract() {
        let a = vec!["a".into(), "b".into(), "a".into()];
        let b = vec!["b".into()];
        assert_eq!(subtract(&a, &b), vec!["a"]);
    }

    #[test]
    fn preserves_order_from_first() {
        let a = vec!["c".into(), "b".into(), "a".into()];
        let b = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(intersect(&a, &b), vec!["c", "b", "a"]);
    }

    #[test]
    fn resolve_input_splits_whitespace() {
        let mut stdin_used = false;
        let result = resolve_input("a b c", &mut stdin_used).unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);
        assert!(!stdin_used);
    }
}
