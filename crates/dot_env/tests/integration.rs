use std::fs;
use std::process::Command;

fn dot_env_bin() -> String {
    let mut path = std::path::PathBuf::from(env!("CARGO_BIN_EXE_dot_env"));
    // Resolve the path to make sure it's absolute
    path = fs::canonicalize(&path).unwrap_or(path);
    path.to_string_lossy().to_string()
}

fn run(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(dot_env_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to execute dot_env")
}

#[test]
fn read_existing_key() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".env"), "DATABASE_URL=postgres://localhost/db\n").unwrap();

    let output = run(tmp.path(), &["DATABASE_URL"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "postgres://localhost/db"
    );
}

#[test]
fn write_existing_key() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".env"), "DATABASE_URL=postgres://localhost/db\n").unwrap();

    let output = run(tmp.path(), &["DATABASE_URL", "postgres://new"]);
    assert!(output.status.success());

    let contents = fs::read_to_string(tmp.path().join(".env")).unwrap();
    assert!(contents.contains("DATABASE_URL=postgres://new"));
}

#[test]
fn read_missing_key() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".env"), "DATABASE_URL=postgres://localhost/db\n").unwrap();

    let output = run(tmp.path(), &["MISSING_KEY"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"));
}

#[test]
fn no_arguments() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".env"), "KEY=val\n").unwrap();

    let output = run(tmp.path(), &[]);
    assert!(!output.status.success());
}

#[test]
fn value_with_equals() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".env"), "URL=http://x?a=b\n").unwrap();

    let output = run(tmp.path(), &["URL"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "http://x?a=b"
    );
}

#[test]
fn skip_commented_line() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".env"), "# FOO=bar\nFOO=baz\n").unwrap();

    let output = run(tmp.path(), &["FOO"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "baz");
}

#[test]
fn no_env_file() {
    let tmp = tempfile::tempdir().unwrap();

    let output = run(tmp.path(), &["KEY"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"));
}

#[test]
fn write_value_with_equals() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".env"), "URL=http://old\n").unwrap();

    let output = run(tmp.path(), &["URL", "http://x?a=b"]);
    assert!(output.status.success());

    let contents = fs::read_to_string(tmp.path().join(".env")).unwrap();
    assert!(contents.contains("URL=http://x?a=b"));
}

#[test]
fn multiple_matches() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".env"), "DB_HOST=x\nDB_PORT=5432\n").unwrap();

    let output = run(tmp.path(), &["DB_"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("matched"));
}

#[test]
fn empty_value_read() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".env"), "KEY=\n").unwrap();

    let output = run(tmp.path(), &["KEY"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "");
}
