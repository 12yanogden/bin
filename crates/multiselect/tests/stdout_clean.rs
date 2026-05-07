use expectrl::{spawn, ControlCode, Eof};
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

#[test]
fn tui_does_not_pollute_stdout() {
    let bin = env!("CARGO_BIN_EXE_multiselect");
    let dir = tempfile::tempdir().unwrap();
    let items = dir.path().join("items.tsv");
    let stdout_log = dir.path().join("stdout.log");
    let stderr_log = dir.path().join("stderr.log");
    let script = dir.path().join("run.sh");

    std::fs::write(&items, "alpha\tAlpha\t\t1\nbeta\tBeta\t\t1\n").unwrap();
    std::fs::write(
        &script,
        format!(
            "#!/bin/bash\nexec {} < {} > {} 2> {}\n",
            bin,
            items.display(),
            stdout_log.display(),
            stderr_log.display(),
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

    let mut session = spawn(script.to_str().unwrap()).unwrap();
    session.set_expect_timeout(Some(Duration::from_secs(5)));

    session.expect("Alpha").unwrap();
    session.send(ControlCode::CarriageReturn).unwrap();
    session.expect(Eof).unwrap();

    let captured = std::fs::read(&stdout_log).unwrap();
    let stderr_data = std::fs::read_to_string(&stderr_log).unwrap_or_default();
    assert!(
        !captured.contains(&0x1b),
        "stdout contained ESC byte (TUI escape codes leaked to captured stdout): {:?}\nstderr: {}",
        String::from_utf8_lossy(&captured),
        stderr_data,
    );
    assert_eq!(
        String::from_utf8(captured).unwrap().lines().collect::<Vec<_>>(),
        vec!["alpha", "beta"],
        "stderr was: {}",
        stderr_data,
    );
}
