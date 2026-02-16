use bin_lib::{fmt, tags};
use clap::Parser;
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Enable all commands under a tag")]
struct Cli {
    /// Tag name
    tag: String,
}

fn run() -> Result<String, String> {
    let cli = Cli::parse();
    let path = tags::tags_json_path()?;
    let map = tags::read_tags(&path)?;

    let cmds = map
        .get(&cli.tag)
        .ok_or_else(|| format!("Tag '{}' not found", cli.tag))?;

    let enabled_dir = tags::enabled_dir()?;
    let all_dir = tags::all_dir()?;
    let mut enabled = Vec::new();

    for cmd in cmds {
        let enabled_path = enabled_dir.join(cmd);
        let all_path = all_dir.join(cmd);

        if enabled_path.symlink_metadata().is_ok() {
            // Already enabled (symlink exists), skip
        } else if all_path.exists() {
            std::os::unix::fs::symlink(&all_path, &enabled_path)
                .map_err(|e| format!("failed to enable '{}': {}", cmd, e))?;
            enabled.push(cmd.as_str());
        } else {
            eprintln!("warning: '{}' not found in all/", cmd);
        }
    }

    if enabled.is_empty() {
        Ok(format!("All commands in '{}' already enabled", cli.tag))
    } else {
        Ok(format!("Enabled '{}': {}", cli.tag, enabled.join(", ")))
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
