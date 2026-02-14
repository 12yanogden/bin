use regex::Regex;
use std::collections::HashSet;
use std::process::Command;

pub fn parse_branches(output: &str, strip_remote_prefix: bool) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            if trimmed.contains("->") {
                return None;
            }
            let name = trimmed.strip_prefix("* ").unwrap_or(trimmed);
            if strip_remote_prefix {
                name.split_once('/').map(|(_, rest)| rest.to_string())
            } else {
                Some(name.to_string())
            }
        })
        .collect()
}

fn git_branches(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run git: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Search for branches matching a pattern (substring or regex).
/// Checks local branches first, then falls back to remote branches.
pub fn search_branches(pattern: &str) -> Result<Vec<String>, String> {
    let regex = Regex::new(pattern).map_err(|e| format!("invalid pattern: {}", e))?;

    // Search local branches first
    let local_output = git_branches(&["branch", "--list"])?;
    let local_branches = parse_branches(&local_output, false);

    let mut seen = HashSet::new();
    let matched: Vec<String> = local_branches
        .into_iter()
        .filter(|b| regex.is_match(b))
        .filter(|b| seen.insert(b.clone()))
        .collect();

    if !matched.is_empty() {
        return Ok(matched);
    }

    // Fall back to remote branches
    let remote_output = git_branches(&["branch", "-r"])?;
    let remote_branches = parse_branches(&remote_output, true);

    seen.clear();
    let matched: Vec<String> = remote_branches
        .into_iter()
        .filter(|b| regex.is_match(b))
        .filter(|b| seen.insert(b.clone()))
        .collect();

    Ok(matched)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_local_branches_strips_marker_and_whitespace() {
        let output = "* main\n  feature/a\n  feature/b\n";
        let branches = parse_branches(output, false);
        assert_eq!(branches, vec!["main", "feature/a", "feature/b"]);
    }

    #[test]
    fn parse_remote_branches_strips_origin_prefix() {
        let output = "  origin/main\n  origin/feature/a\n  origin/HEAD -> origin/main\n";
        let branches = parse_branches(output, true);
        assert_eq!(branches, vec!["main", "feature/a"]);
    }

    #[test]
    fn parse_empty_output() {
        let branches = parse_branches("", false);
        assert!(branches.is_empty());
    }

    #[test]
    fn deduplication_preserves_order() {
        let input = vec!["main", "feature/a", "main", "feature/b", "feature/a"];
        let mut seen = HashSet::new();
        let deduped: Vec<&str> = input
            .into_iter()
            .filter(|b| seen.insert(*b))
            .collect();
        assert_eq!(deduped, vec!["main", "feature/a", "feature/b"]);
    }

    #[test]
    fn search_branches_invalid_pattern() {
        let result = search_branches("[invalid");
        assert!(result.is_err());
    }
}
