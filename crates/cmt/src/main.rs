use bin_lib::git;
use bin_lib::ticket::extract_ticket;
use clap::Parser;
use shell_executor::{execute, fail, pass};
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Git add, commit, and push in one step")]
struct Cli {
    /// Commit message
    message: String,
}

fn sh_quote(s: &str) -> String {
    let escaped = s.replace('\'', "'\\''");
    format!("'{}'", escaped)
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
            fail(&format!("{}", e));
            return ExitCode::FAILURE;
        }
    };

    match git::is_dirty(&repo) {
        Ok(false) => {
            pass("working tree clean");
            return ExitCode::SUCCESS;
        }
        Ok(true) => {}
        Err(e) => {
            fail(&format!("{}", e));
            return ExitCode::FAILURE;
        }
    }

    let message = build_message(
        &cli.message,
        git::current_branch(&repo).ok().as_deref(),
    );

    let commit_cmd = format!("git commit -m {}", sh_quote(&message));
    let steps = ["git add .", &commit_cmd, "git push"];

    for cmd in steps {
        let report = execute(cmd).run_report();
        if !report.status.is_success() {
            let code = u8::try_from(report.exit_code).unwrap_or(1);
            return ExitCode::from(if code == 0 { 1 } else { code });
        }
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

    #[test]
    fn sh_quote_wraps_plain_text() {
        assert_eq!(sh_quote("hello world"), "'hello world'");
    }

    #[test]
    fn sh_quote_escapes_single_quotes() {
        assert_eq!(sh_quote("it's fine"), "'it'\\''s fine'");
    }

    #[test]
    fn sh_quote_handles_double_quotes_unchanged() {
        assert_eq!(sh_quote("a \"q\" b"), "'a \"q\" b'");
    }
}
