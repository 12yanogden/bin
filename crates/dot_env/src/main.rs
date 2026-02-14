use clap::Parser;
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Read or write values in a .env file")]
struct Cli {
    /// Key pattern to search for
    key_pattern: String,

    /// New value to set (omit to read)
    new_value: Option<String>,
}

fn find_matches(contents: &str, pattern: &str) -> Vec<(usize, String, String)> {
    let mut matches = Vec::new();
    for (idx, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            if key.contains(pattern) {
                matches.push((idx, key.to_string(), value.to_string()));
            }
        }
    }
    matches
}

fn replace_value(contents: &str, line_index: usize, key: &str, new_value: &str) -> String {
    let mut lines: Vec<&str> = contents.lines().collect();
    let replacement = format!("{}={}", key, new_value);
    let owned;
    if line_index < lines.len() {
        owned = replacement;
        lines[line_index] = &owned;
    }
    let mut result = lines.join("\n");
    if contents.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let contents = match std::fs::read_to_string(".env") {
        Ok(c) => c,
        Err(_) => {
            eprintln!("error: no .env file in current directory");
            return ExitCode::from(1);
        }
    };

    let matches = find_matches(&contents, &cli.key_pattern);

    match matches.len() {
        0 => {
            eprintln!(
                "error: pattern '{}' did not match any keys",
                cli.key_pattern
            );
            ExitCode::from(1)
        }
        1 => {
            let (line_index, key, value) = &matches[0];
            match cli.new_value {
                None => {
                    println!("{}", value);
                    ExitCode::SUCCESS
                }
                Some(new_value) => {
                    let updated = replace_value(&contents, *line_index, key, &new_value);
                    if let Err(e) = std::fs::write(".env", updated) {
                        eprintln!("error: failed to write .env: {}", e);
                        return ExitCode::from(1);
                    }
                    ExitCode::SUCCESS
                }
            }
        }
        count => {
            eprintln!(
                "error: pattern '{}' matched {} keys",
                cli.key_pattern, count
            );
            for (_, key, value) in &matches {
                eprintln!("  {}={}", key, value);
            }
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_matches_single_match() {
        let contents = "DATABASE_URL=postgres://localhost/db\nPORT=5432\n";
        let matches = find_matches(contents, "DATABASE_URL");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].1, "DATABASE_URL");
        assert_eq!(matches[0].2, "postgres://localhost/db");
    }

    #[test]
    fn find_matches_no_match() {
        let contents = "DATABASE_URL=postgres://localhost/db\n";
        let matches = find_matches(contents, "MISSING_KEY");
        assert!(matches.is_empty());
    }

    #[test]
    fn find_matches_skips_comments() {
        let contents = "# FOO=bar\nFOO=baz\n";
        let matches = find_matches(contents, "FOO");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].2, "baz");
    }

    #[test]
    fn find_matches_value_with_equals() {
        let contents = "URL=http://x?a=b\n";
        let matches = find_matches(contents, "URL");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].2, "http://x?a=b");
    }

    #[test]
    fn replace_value_target_line_only() {
        let contents = "FOO=old\nBAR=keep\n";
        let result = replace_value(contents, 0, "FOO", "new");
        assert_eq!(result, "FOO=new\nBAR=keep\n");
    }

    #[test]
    fn replace_value_preserves_other_equals() {
        let contents = "URL=http://x?a=b\nFOO=bar\n";
        let result = replace_value(contents, 1, "FOO", "baz");
        assert_eq!(result, "URL=http://x?a=b\nFOO=baz\n");
    }
}
