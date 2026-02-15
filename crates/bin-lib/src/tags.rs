use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub type TagMap = BTreeMap<String, Vec<String>>;

/// Resolve the project root by finding the binary's parent directory
/// (either `enabled/` or `disabled/`) and going one level up.
pub fn project_root() -> Result<PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("failed to determine executable path: {}", e))?;
    let bin_dir = exe
        .parent()
        .ok_or("failed to determine binary directory")?;
    let root = bin_dir
        .parent()
        .ok_or("failed to determine project root")?;
    Ok(root.to_path_buf())
}

pub fn tags_json_path() -> Result<PathBuf, String> {
    Ok(project_root()?.join("tags.json"))
}

pub fn enabled_dir() -> Result<PathBuf, String> {
    Ok(project_root()?.join("enabled"))
}

pub fn disabled_dir() -> Result<PathBuf, String> {
    Ok(project_root()?.join("disabled"))
}

pub fn read_tags(path: &Path) -> Result<TagMap, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    let map: TagMap = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {}", path.display(), e))?;
    Ok(map)
}

pub fn write_tags(path: &Path, tags: &TagMap) -> Result<(), String> {
    let json = serde_json::to_string_pretty(tags)
        .map_err(|e| format!("failed to serialize tags: {}", e))?;
    fs::write(path, json + "\n")
        .map_err(|e| format!("failed to write {}: {}", path.display(), e))?;
    Ok(())
}

/// Check if a command binary exists in the enabled directory.
pub fn is_enabled(cmd: &str) -> Result<bool, String> {
    Ok(enabled_dir()?.join(cmd).exists())
}

/// Check if a command binary exists in the disabled directory.
pub fn is_disabled(cmd: &str) -> Result<bool, String> {
    Ok(disabled_dir()?.join(cmd).exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn read_tags_valid() {
        let dir = env::temp_dir().join("bin_test_read_valid");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tags.json");
        fs::write(&path, r#"{"git": ["bs", "cb"], "util": ["x"]}"#).unwrap();

        let tags = read_tags(&path).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags["git"], vec!["bs", "cb"]);
        assert_eq!(tags["util"], vec!["x"]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn read_tags_missing_file() {
        let path = env::temp_dir().join("bin_test_nonexistent/tags.json");
        assert!(read_tags(&path).is_err());
    }

    #[test]
    fn write_then_read_roundtrip() {
        let dir = env::temp_dir().join("bin_test_roundtrip");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tags.json");

        let mut tags = TagMap::new();
        tags.insert("beta".to_string(), vec!["b1".to_string(), "b2".to_string()]);
        tags.insert("alpha".to_string(), vec!["a1".to_string()]);

        write_tags(&path, &tags).unwrap();
        let loaded = read_tags(&path).unwrap();
        assert_eq!(tags, loaded);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn btreemap_sorted_keys() {
        let dir = env::temp_dir().join("bin_test_sorted");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tags.json");

        let mut tags = TagMap::new();
        tags.insert("zebra".to_string(), vec!["z".to_string()]);
        tags.insert("alpha".to_string(), vec!["a".to_string()]);

        write_tags(&path, &tags).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let alpha_pos = content.find("alpha").unwrap();
        let zebra_pos = content.find("zebra").unwrap();
        assert!(alpha_pos < zebra_pos);

        fs::remove_dir_all(&dir).unwrap();
    }
}
