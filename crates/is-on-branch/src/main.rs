use bin_lib::git;
use regex::Regex;
use std::process::ExitCode;

fn check_branch_matches(branch: &str, pattern: &str) -> Result<bool, String> {
    let regex = Regex::new(pattern).map_err(|e| format!("invalid pattern: {}", e))?;
    Ok(regex.is_match(branch))
}

fn main() -> ExitCode {
    let pattern = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: is_on_branch <pattern>");
            return ExitCode::FAILURE;
        }
    };

    let branch = match git::get_current_branch() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("is_on_branch: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match check_branch_matches(&branch, &pattern) {
        Ok(true) => println!("1"),
        Ok(false) => println!("0"),
        Err(e) => {
            eprintln!("is_on_branch: {}", e);
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert_eq!(check_branch_matches("main", "main"), Ok(true));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(check_branch_matches("develop", "main"), Ok(false));
    }

    #[test]
    fn test_regex_match() {
        assert_eq!(check_branch_matches("feature/x", "feature.*"), Ok(true));
    }

    #[test]
    fn test_invalid_regex() {
        assert!(check_branch_matches("main", "[invalid").is_err());
    }

    #[test]
    fn test_partial_match() {
        assert_eq!(check_branch_matches("main", "mai"), Ok(true));
    }
}
