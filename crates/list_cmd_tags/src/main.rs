use bin_lib::{fmt, tags};
use std::process::ExitCode;

fn run() -> Result<(), String> {
    let path = tags::tags_json_path()?;
    let map = tags::read_tags(&path)?;

    let col_width = map
        .values()
        .flat_map(|cmds| cmds.iter())
        .map(|cmd| cmd.len())
        .max()
        .unwrap_or(0)
        + 1;

    for (tag, cmds) in &map {
        println!("{}:", tag);
        for cmd in cmds {
            let status = if tags::is_enabled(cmd)? {
                format!("{}[enabled]{}", fmt::GREEN, fmt::RESET)
            } else if tags::is_disabled(cmd)? {
                format!("{}[disabled]{}", fmt::RED, fmt::RESET)
            } else {
                "[not found]".to_string()
            };
            println!("  {:<width$}{}", cmd, status, width = col_width);
        }
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
