use bin_lib::{fmt, tags};
use clap::Parser;
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "Add a tag to one or more commands in tags.json")]
struct Cli {
    /// Tag name
    tag: String,

    /// Command names to tag
    #[arg(required = true)]
    cmd_names: Vec<String>,
}

fn run() -> Result<String, String> {
    let cli = Cli::parse();
    let path = tags::tags_json_path()?;
    let mut map = tags::read_tags(&path)?;

    let entry = map.entry(cli.tag.clone()).or_insert_with(Vec::new);
    let mut added = Vec::new();

    for cmd in &cli.cmd_names {
        if !entry.contains(cmd) {
            entry.push(cmd.clone());
            added.push(cmd.as_str());
        }
    }

    entry.sort();
    tags::write_tags(&path, &map)?;

    if added.is_empty() {
        Ok(format!("All commands already tagged '{}'", cli.tag))
    } else {
        Ok(format!("Tagged '{}': {}", cli.tag, added.join(", ")))
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
