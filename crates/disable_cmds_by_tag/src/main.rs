use bin_lib::{fmt, tags};
use clap::Parser;
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Disable all commands under a tag")]
struct Cli {
    /// Tag name
    tag: String,
}

fn run() -> Result<String, String> {
    let cli = Cli::parse();

    if cli.tag == "bin-admin" {
        return Err("Cannot disable the bin-admin tag".to_string());
    }

    let path = tags::tags_json_path()?;
    let map = tags::read_tags(&path)?;

    let cmds = map
        .get(&cli.tag)
        .ok_or_else(|| format!("Tag '{}' not found", cli.tag))?;

    let enabled_dir = tags::enabled_dir()?;
    let disabled_dir = tags::disabled_dir()?;
    let mut disabled = Vec::new();

    for cmd in cmds {
        let enabled_path = enabled_dir.join(cmd);
        let disabled_path = disabled_dir.join(cmd);

        if enabled_path.exists() {
            std::fs::rename(&enabled_path, &disabled_path)
                .map_err(|e| format!("failed to disable '{}': {}", cmd, e))?;
            disabled.push(cmd.as_str());
        } else if disabled_path.exists() {
            // Already disabled, skip
        } else {
            eprintln!("warning: '{}' not found in enabled/ or disabled/", cmd);
        }
    }

    if disabled.is_empty() {
        Ok(format!("All commands in '{}' already disabled", cli.tag))
    } else {
        Ok(format!("Disabled '{}': {}", cli.tag, disabled.join(", ")))
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(msg) => {
            fmt::pass(&msg);
            ExitCode::SUCCESS
        }
        Err(e) => {
            fmt::fail(&e);
            ExitCode::from(1)
        }
    }
}
