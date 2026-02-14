use std::process::Command;

fn x_bin() -> std::path::PathBuf {
    env!("CARGO_BIN_EXE_x").into()
}

fn run_x(args: &[&str]) -> std::process::Output {
    Command::new(x_bin())
        .args(args)
        .output()
        .expect("failed to execute x binary")
}

#[test]
fn test_echo_hello_pass() {
    let output = run_x(&["echo hello", "--no-placeholder"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[ "));
    assert!(stdout.contains("PASS"));
    assert!(stdout.contains(" ] echo hello"));
    assert!(output.status.success());
}

#[test]
fn test_false_fail() {
    let output = run_x(&["false", "--no-placeholder"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[ "));
    assert!(stdout.contains("FAIL"));
    assert!(stdout.contains(" ] false"));
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_custom_message() {
    let output = run_x(&["echo hello", "-m", "Greeting", "--no-placeholder"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
    assert!(stdout.contains(" ] Greeting"));
    assert!(output.status.success());
}

#[test]
fn test_verbose_shows_output() {
    let output = run_x(&["echo hello", "--verbose", "--no-placeholder"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
    assert!(stdout.contains("hello"));
    assert!(output.status.success());
}

#[test]
fn test_succinct_no_wrapper() {
    let output = run_x(&["echo hello", "-s"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello"));
    assert!(!stdout.contains("PASS"));
    assert!(!stdout.contains("FAIL"));
    assert!(output.status.success());
}

#[test]
fn test_chained_commands() {
    let output = run_x(&["echo a && echo b", "--no-placeholder"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
    assert!(output.status.success());
}

#[test]
fn test_validator_pass_command_fail() {
    // Validator says pass, but command failed — keep command's exit code
    let output = run_x(&["false", "-v", "true", "--no-placeholder"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_validator_fail_command_pass() {
    // Validator says fail, command succeeded — override exit code to 1
    let output = run_x(&["echo hi", "-v", "false", "--no-placeholder"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FAIL"));
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_no_args() {
    let output = Command::new(x_bin())
        .output()
        .expect("failed to execute x binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage") || stderr.contains("usage") || stderr.contains("error"));
}

#[test]
fn test_exit_code_propagation() {
    let output = run_x(&["exit 42", "--no-placeholder"]);
    assert_eq!(output.status.code(), Some(42));
}

#[test]
fn test_no_placeholder_flag() {
    let output = run_x(&["echo hello", "--no-placeholder"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
    assert!(output.status.success());
}

#[test]
fn test_fail_shows_output() {
    let output = run_x(&["echo oops >&2 && false", "--no-placeholder"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("FAIL"));
    // oops goes to stderr of the child, which x captures and prints to its stdout (indented)
    assert!(stdout.contains("oops") || stderr.contains("oops"));
}

#[test]
fn test_chained_with_semicolon() {
    let output = run_x(&["echo a; echo b", "--no-placeholder"]);
    assert!(output.status.success());
}

#[test]
fn test_chained_with_or() {
    let output = run_x(&["false || echo recovered", "--no-placeholder"]);
    assert!(output.status.success());
}
