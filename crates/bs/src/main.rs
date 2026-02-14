use clap::Parser;
use regex::Regex;
use std::collections::HashSet;
use std::process::{Command, ExitCode};

#[derive(Parser)]
#[command(about = "Search for git branches matching a pattern")]
struct Cli {
    /// Branch name pattern (substring or regex)
    pattern: String,
}

fn parse_branches(output: &str, strip_remote_prefix: bool) -> Vec<String> {
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

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            eprint!("{}", e);
            std::process::exit(1);
        }
    };

    let regex = match Regex::new(&cli.pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("invalid pattern: {}", e);
            return ExitCode::from(1);
        }
    };

    // Search local branches first
    let local_output = match git_branches(&["branch", "--list"]) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("error listing local branches: {}", e);
            return ExitCode::from(1);
        }
    };

    let local_branches = parse_branches(&local_output, false);
    let mut seen = HashSet::new();
    let matched: Vec<String> = local_branches
        .into_iter()
        .filter(|b| regex.is_match(b))
        .filter(|b| seen.insert(b.clone()))
        .collect();

    if !matched.is_empty() {
        for branch in &matched {
            println!("{}", branch);
        }
        return ExitCode::SUCCESS;
    }

    // Fall back to remote branches
    let remote_output = match git_branches(&["branch", "-r"]) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("error listing remote branches: {}", e);
            return ExitCode::from(1);
        }
    };

    let remote_branches = parse_branches(&remote_output, true);
    seen.clear();
    let matched: Vec<String> = remote_branches
        .into_iter()
        .filter(|b| regex.is_match(b))
        .filter(|b| seen.insert(b.clone()))
        .collect();

    if matched.is_empty() {
        return ExitCode::from(1);
    }

    for branch in &matched {
        println!("{}", branch);
    }
    ExitCode::SUCCESS
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
}
