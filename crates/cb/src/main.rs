use bin_lib::{branch, fmt, menu};
use clap::Parser;
use std::process::{Command, ExitCode};

#[derive(Parser)]
#[command(about = "Interactive fuzzy branch checkout")]
struct Cli {
    /// Branch name pattern (substring or regex)
    pattern: String,
}

fn git_fetch() -> Result<(), String> {
    let output = Command::new("git")
        .args(["fetch"])
        .output()
        .map_err(|e| format!("failed to run git fetch: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(())
}

fn git_checkout(branch: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["checkout", branch])
        .output()
        .map_err(|e| format!("failed to run git checkout: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(())
}

fn run() -> Result<String, String> {
    let cli = Cli::parse();

    // Search for matching branches
    let mut matches = branch::search_branches(&cli.pattern)?;

    // If no matches, fetch and retry
    if matches.is_empty() {
        git_fetch()?;
        matches = branch::search_branches(&cli.pattern)?;

        if matches.is_empty() {
            return Err(format!("no branches match the pattern: '{}'", cli.pattern));
        }
    }

    // Select branch
    let branch = if matches.len() == 1 {
        matches.into_iter().next().unwrap()
    } else {
        eprintln!("Multiple branches match the pattern: '{}'", cli.pattern);
        menu::prompt_menu(&matches)?
    };

    // Checkout
    git_checkout(&branch)?;

    Ok(branch)
}

fn main() -> ExitCode {
    match run() {
        Ok(branch) => {
            fmt::pass(&format!("Checked out {}", branch));
            ExitCode::SUCCESS
        }
        Err(e) => {
            fmt::fail(&e);
            ExitCode::from(1)
        }
    }
}
