use bin_lib::fmt;
use bin_lib::git;
use bin_lib::ticket::extract_ticket;
use clap::Parser;
use std::process::{Command, ExitCode};

#[derive(Parser)]
#[command(about = "Git add, commit, and push in one step")]
struct Cli {
    /// Commit message
    message: String,
}

fn run_cmd(cmd: &str, args: &[&str]) -> bool {
    let label = format!("{} {}", cmd, args.join(" "));
    match Command::new(cmd).args(args).status() {
        Ok(s) if s.success() => {
            fmt::pass(&label);
            true
        }
        Ok(_) => {
            fmt::fail(&label);
            false
        }
        Err(e) => {
            fmt::fail(&format!("{}: {}", label, e));
            false
        }
    }
}

fn build_message(message: &str, branch: Option<&str>) -> String {
    match branch.and_then(|b| extract_ticket(b)) {
        Some(ticket) => format!("{} {}", ticket, message),
        None => message.to_string(),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let repo = match git::open_repo() {
        Ok(r) => r,
        Err(e) => {
            fmt::fail(&format!("{}", e));
            return ExitCode::FAILURE;
        }
    };

    match git::is_dirty(&repo) {
        Ok(false) => {
            fmt::pass("working tree clean");
            return ExitCode::SUCCESS;
        }
        Ok(true) => {}
        Err(e) => {
            fmt::fail(&format!("{}", e));
            return ExitCode::FAILURE;
        }
    }

    let message = build_message(
        &cli.message,
        git::current_branch(&repo).ok().as_deref(),
    );

    if !run_cmd("git", &["add", "."]) {
        return ExitCode::FAILURE;
    }

    if !run_cmd("git", &["commit", "-m", &message]) {
        return ExitCode::FAILURE;
    }

    if !run_cmd("git", &["push"]) {
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_with_ticket_branch() {
        let msg = build_message("fix login", Some("feature/PROJ-123-add-login"));
        assert_eq!(msg, "PROJ-123 fix login");
    }

    #[test]
    fn message_without_ticket() {
        let msg = build_message("fix login", Some("main"));
        assert_eq!(msg, "fix login");
    }

    #[test]
    fn message_with_no_branch() {
        let msg = build_message("fix login", None);
        assert_eq!(msg, "fix login");
    }

    #[test]
    fn message_preserves_original_text() {
        let msg = build_message("a \"quoted\" message", Some("BIN-9-rewrite"));
        assert_eq!(msg, "BIN-9 a \"quoted\" message");
    }
}
