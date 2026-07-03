use std::process::{Command, Stdio};

const INSTALL_URL: &str =
    "https://github.com/12yanogden/bin/releases/latest/download/install.sh";

fn main() {
    let mut curl = Command::new("curl")
        .args([
            "--proto",
            "=https",
            "--tlsv1.2",
            "-fsSL",
            INSTALL_URL,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|err| {
            eprintln!("Failed to run curl: {err}");
            std::process::exit(1);
        });

    let mut bash = Command::new("bash")
        .arg("-s")
        .args(std::env::args().skip(1))
        .stdin(curl.stdout.take().expect("curl stdout was piped"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|err| {
            eprintln!("Failed to run bash: {err}");
            let _ = curl.kill();
            std::process::exit(1);
        });

    let curl_status = curl.wait().unwrap_or_else(|err| {
        eprintln!("Failed to wait for curl: {err}");
        let _ = bash.kill();
        std::process::exit(1);
    });

    if !curl_status.success() {
        let _ = bash.kill();
        std::process::exit(curl_status.code().unwrap_or(1));
    }

    let bash_status = bash.wait().unwrap_or_else(|err| {
        eprintln!("Failed to wait for installer: {err}");
        std::process::exit(1);
    });

    std::process::exit(bash_status.code().unwrap_or(1));
}
