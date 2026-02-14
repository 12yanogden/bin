use bin_lib::git;
use std::process::ExitCode;

fn main() -> ExitCode {
    let repo = match git::open_repo() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("is_dirty: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match git::is_dirty(&repo) {
        Ok(dirty) => {
            if dirty {
                println!("1");
            } else {
                println!("0");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("is_dirty: {}", e);
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use bin_lib::git;

    #[test]
    fn test_not_a_repo() {
        let td = std::env::temp_dir().join("is-dirty-test-not-a-repo");
        let _ = std::fs::remove_dir_all(&td);
        std::fs::create_dir_all(&td).unwrap();

        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(&td).unwrap();
        let result = git::open_repo();
        std::env::set_current_dir(original).unwrap();

        assert!(result.is_err());
        std::fs::remove_dir_all(&td).unwrap();
    }
}
