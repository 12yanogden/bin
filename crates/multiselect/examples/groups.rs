use multiselect::{Item, Multiselect};

fn item(id: &str, label: &str, parent: Option<&str>, selected: bool) -> Item {
    Item {
        id: id.to_string(),
        label: label.to_string(),
        parent: parent.map(|s| s.to_string()),
        selected,
    }
}

fn main() -> Result<(), String> {
    let items = vec![
        item("bin-admin", "bin-admin", None, false),
        item("cmd1", "command 1", Some("bin-admin"), false),
        item("cmd2", "command 2", Some("bin-admin"), true),
        item("cmd3", "command 3", Some("bin-admin"), false),
        item("git", "git", None, true),
        item("cmd4", "command 4", Some("git"), true),
        item("cmd5", "command 5", Some("git"), true),
        item("cmd6", "command 6", Some("git"), true),
        item("shell-util", "shell-util", None, false),
        item("cmd7", "command 7", Some("shell-util"), true),
        item("cmd8", "command 8", Some("shell-util"), true),
        item("cmd9", "command 9", Some("shell-util"), true),
        item("solo", "ungrouped item", None, false),
    ];

    match Multiselect::new("Pick commands:").items(items).run()? {
        Some(ids) => {
            println!("Selected:");
            for id in ids {
                println!("  {}", id);
            }
        }
        None => println!("cancelled"),
    }
    Ok(())
}
