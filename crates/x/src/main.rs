use bin_lib::fmt;
use clap::Parser;
use std::io::{IsTerminal, Write};
use std::process::{Command, ExitCode};

#[derive(Parser)]
#[command(about = "Execute a command and display a color-coded PASS/FAIL result")]
struct Cli {
    /// The shell command to execute
    command: String,

    /// Custom display message (defaults to the command string)
    #[arg(short, long)]
    msg: Option<String>,

    /// A shell command that validates success (exit code 0 = valid)
    #[arg(short = 'v', long)]
    validator: Option<String>,

    /// Show command stdout/stderr on success (always shown on failure)
    #[arg(long)]
    verbose: bool,

    /// Suppress [ PASS ] / [ FAIL ] formatting, just run the command
    #[arg(short, long)]
    succinct: bool,

    /// Don't show a placeholder message while the command runs
    #[arg(long)]
    no_placeholder: bool,
}

fn print_indented(text: &str) {
    let trimmed = text.trim_end();
    if !trimmed.is_empty() {
        for line in trimmed.lines() {
            println!("    {}", line);
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let msg = cli.msg.as_deref().unwrap_or(&cli.command);

    // Show placeholder
    let show_placeholder =
        !cli.succinct && !cli.no_placeholder && std::io::stderr().is_terminal();
    if show_placeholder {
        let placeholder = format!("[      ] {}", msg);
        eprint!("{}", placeholder);
        let _ = std::io::stderr().flush();
    }

    // Execute command
    let output = match Command::new("sh").arg("-c").arg(&cli.command).output() {
        Ok(o) => o,
        Err(e) => {
            if show_placeholder {
                eprint!("\r{}\r", " ".repeat(msg.len() + 10));
            }
            fmt::fail(&format!("{}: {}", msg, e));
            return ExitCode::from(1);
        }
    };

    let mut exit_code: u8 = match output.status.code() {
        Some(c) => c.clamp(0, 255) as u8,
        None => 1,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Determine pass/fail
    let is_pass = if let Some(ref validator) = cli.validator {
        match Command::new("sh").arg("-c").arg(validator).output() {
            Ok(v) => {
                let valid = v.status.success();
                if !valid && exit_code == 0 {
                    exit_code = 1;
                }
                valid
            }
            Err(_) => {
                if exit_code == 0 {
                    exit_code = 1;
                }
                false
            }
        }
    } else {
        exit_code == 0
    };

    // Erase placeholder
    if show_placeholder {
        let clear_len = msg.len() + 10;
        eprint!("\r{}\r", " ".repeat(clear_len));
    }

    // Print result
    if cli.succinct {
        print!("{}", stdout);
        eprint!("{}", stderr);
    } else if is_pass {
        fmt::pass(msg);
        if cli.verbose {
            print_indented(&stdout);
            print_indented(&stderr);
        }
    } else {
        fmt::fail(msg);
        print_indented(&stdout);
        print_indented(&stderr);
    }

    ExitCode::from(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_indented_multiline() {
        // Just verify it doesn't panic; actual output goes to stdout
        print_indented("line1\nline2\nline3");
    }

    #[test]
    fn test_print_indented_empty() {
        print_indented("");
        print_indented("   ");
    }
}
