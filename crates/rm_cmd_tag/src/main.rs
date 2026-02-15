use bin_lib::{fmt, tags};
use clap::Parser;
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Remove a tag from one or more commands in tags.json")]
struct Cli {
    /// Tag name
    tag: String,

    /// Command names to untag
    #[arg(required = true)]
    cmd_names: Vec<String>,
}

fn run() -> Result<String, String> {
    let cli = Cli::parse();
    let path = tags::tags_json_path()?;
    let mut map = tags::read_tags(&path)?;

    let cmds = match map.get_mut(&cli.tag) {
        Some(cmds) => cmds,
        None => return Ok(format!("Tag '{}' does not exist, nothing to do", cli.tag)),
    };

    let mut removed = Vec::new();
    for cmd in &cli.cmd_names {
        if let Some(pos) = cmds.iter().position(|c| c == cmd) {
            cmds.remove(pos);
            removed.push(cmd.as_str());
        }
    }

    if cmds.is_empty() {
        map.remove(&cli.tag);
    }

    tags::write_tags(&path, &map)?;

    if removed.is_empty() {
        Ok(format!("No commands to remove from tag '{}'", cli.tag))
    } else {
        Ok(format!("Untagged '{}': {}", cli.tag, removed.join(", ")))
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
