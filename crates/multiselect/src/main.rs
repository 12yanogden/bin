use multiselect::{Item, Multiselect};
use std::io::{self, Read};
use std::process;

fn print_help() {
    println!("Usage: multiselect [--prompt <text>] < items.tsv");
    println!();
    println!("Reads tab-separated items from stdin, displays an interactive");
    println!("multiselect TUI, and prints selected leaf ids to stdout (one per line).");
    println!();
    println!("Each input line: id<TAB>label<TAB>parent<TAB>selected");
    println!("  label    optional, defaults to id");
    println!("  parent   optional, empty for top-level");
    println!("  selected optional, '1'/'true'/'yes' for pre-selected");
    println!();
    println!("Exit codes: 0 confirmed, 1 cancelled, 2 invalid usage.");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut prompt = String::from("Select items:");
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--prompt" | "-p" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("--prompt requires a value");
                    process::exit(2);
                };
                prompt = v.clone();
                i += 2;
            }
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            other => {
                eprintln!("unknown argument: {}", other);
                process::exit(2);
            }
        }
    }

    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("failed to read stdin: {}", e);
        process::exit(1);
    }

    let mut items = Vec::new();
    for (lineno, line) in input.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split('\t');
        let id = parts.next().unwrap_or("").to_string();
        if id.is_empty() {
            eprintln!("line {}: empty id", lineno + 1);
            process::exit(1);
        }
        let label = parts
            .next()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| id.clone());
        let parent_str = parts.next().unwrap_or("");
        let parent = if parent_str.is_empty() {
            None
        } else {
            Some(parent_str.to_string())
        };
        let selected = matches!(parts.next().unwrap_or(""), "1" | "true" | "yes" | "y");
        items.push(Item {
            id,
            label,
            parent,
            selected,
        });
    }

    match Multiselect::new(prompt).items(items).run() {
        Ok(Some(ids)) => {
            for id in ids {
                println!("{}", id);
            }
        }
        Ok(None) => process::exit(1),
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
