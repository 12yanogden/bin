use bin_lib::{fmt, sets};
use clap::Parser;
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Find the intersection of two sets")]
struct Cli {
    /// First input set (space-delimited string, file path, or '-' for stdin)
    input1: String,

    /// Second input set (space-delimited string, file path, or '-' for stdin)
    input2: String,
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    let mut stdin_used = false;
    let a = sets::resolve_input(&cli.input1, &mut stdin_used)?;
    let b = sets::resolve_input(&cli.input2, &mut stdin_used)?;

    let result = sets::intersect(&a, &b);
    for line in &result {
        println!("{}", line);
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            fmt::fail(&e);
            ExitCode::from(1)
        }
    }
}
