use bin_lib::{branch, menu};
use clap::Parser;
use shell_executor::{execute, fail};
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Interactive fuzzy branch checkout")]
struct Cli {
    /// Branch name pattern (substring or regex)
    pattern: String,
}

fn sh_quote(s: &str) -> String {
    let escaped = s.replace('\'', "'\\''");
    format!("'{}'", escaped)
}

fn exit_code_from_report(exit_code: i32) -> ExitCode {
    let code = u8::try_from(exit_code).unwrap_or(1);
    ExitCode::from(if code == 0 { 1 } else { code })
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mut matches = match branch::search_branches(&cli.pattern) {
        Ok(m) => m,
        Err(e) => {
            fail(&e);
            return ExitCode::from(1);
        }
    };

    if matches.is_empty() {
        let report = execute("git fetch").run_report();
        if !report.status.is_success() {
            return exit_code_from_report(report.exit_code);
        }

        matches = match branch::search_branches(&cli.pattern) {
            Ok(m) => m,
            Err(e) => {
                fail(&e);
                return ExitCode::from(1);
            }
        };

        if matches.is_empty() {
            fail(&format!("no branches match the pattern: '{}'", cli.pattern));
            return ExitCode::from(1);
        }
    }

    let branch = if matches.len() == 1 {
        matches.into_iter().next().unwrap()
    } else {
        eprintln!("Multiple branches match the pattern: '{}'", cli.pattern);
        match menu::prompt_menu(&matches) {
            Ok(b) => b,
            Err(e) => {
                fail(&e);
                return ExitCode::from(1);
            }
        }
    };

    let cmd = format!("git checkout {}", sh_quote(&branch));
    let label = format!("Checked out {}", branch);
    let report = execute(&cmd).message(&label).run_report();
    if !report.status.is_success() {
        return exit_code_from_report(report.exit_code);
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sh_quote_wraps_plain_text() {
        assert_eq!(sh_quote("main"), "'main'");
    }

    #[test]
    fn sh_quote_escapes_single_quotes() {
        assert_eq!(sh_quote("feat/it's-broken"), "'feat/it'\\''s-broken'");
    }
}
