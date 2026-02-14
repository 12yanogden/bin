use bin_lib::branch;
use clap::Parser;
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Search for git branches matching a pattern")]
struct Cli {
    /// Branch name pattern (substring or regex)
    pattern: String,
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            eprint!("{}", e);
            std::process::exit(1);
        }
    };

    let matched = match branch::search_branches(&cli.pattern) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::from(1);
        }
    };

    if matched.is_empty() {
        return ExitCode::from(1);
    }

    for b in &matched {
        println!("{}", b);
    }
    ExitCode::SUCCESS
}
