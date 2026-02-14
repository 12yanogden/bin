use bin_lib::git;
use std::process::ExitCode;

fn main() -> ExitCode {
    match git::get_current_branch() {
        Ok(branch) => {
            println!("{}", branch);
            ExitCode::SUCCESS
        }
        Err(git::GitError::DetachedHead) => {
            println!("HEAD");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("pwb: {}", e);
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use bin_lib::git::{self, GitError};
    use git2::Repository;

    fn init_repo_with_commit(path: &std::path::Path) -> Repository {
        let repo = Repository::init(path).unwrap();
        {
            let sig = git2::Signature::now("Test", "test@test.com").unwrap();
            let tree_id = repo.index().unwrap().write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
                .unwrap();
        }
        repo
    }

    #[test]
    fn prints_current_branch() {
        let td = std::env::temp_dir().join("pwb-test-current-branch");
        let _ = std::fs::remove_dir_all(&td);
        let repo = init_repo_with_commit(&td);

        repo.branch(
            "test-branch",
            &repo.head().unwrap().peel_to_commit().unwrap(),
            false,
        )
        .unwrap();
        repo.set_head("refs/heads/test-branch").unwrap();

        assert_eq!(git::current_branch(&repo).unwrap(), "test-branch");
        std::fs::remove_dir_all(&td).unwrap();
    }

    #[test]
    fn prints_feature_branch() {
        let td = std::env::temp_dir().join("pwb-test-feature-branch");
        let _ = std::fs::remove_dir_all(&td);
        let repo = init_repo_with_commit(&td);

        repo.branch(
            "feature/ABC-123-add-widget",
            &repo.head().unwrap().peel_to_commit().unwrap(),
            false,
        )
        .unwrap();
        repo.set_head("refs/heads/feature/ABC-123-add-widget")
            .unwrap();

        assert_eq!(
            git::current_branch(&repo).unwrap(),
            "feature/ABC-123-add-widget"
        );
        std::fs::remove_dir_all(&td).unwrap();
    }

    #[test]
    fn detached_head_returns_error() {
        let td = std::env::temp_dir().join("pwb-test-detached");
        let _ = std::fs::remove_dir_all(&td);
        let repo = init_repo_with_commit(&td);

        let head_commit = repo.head().unwrap().target().unwrap();
        repo.set_head_detached(head_commit).unwrap();

        assert!(matches!(
            git::current_branch(&repo),
            Err(GitError::DetachedHead)
        ));
        std::fs::remove_dir_all(&td).unwrap();
    }

    #[test]
    fn not_a_repo_returns_error() {
        let td = std::env::temp_dir().join("pwb-test-not-a-repo");
        let _ = std::fs::remove_dir_all(&td);
        std::fs::create_dir_all(&td).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&td).unwrap();
        let result = git::open_repo();
        std::env::set_current_dir(&original_dir).unwrap();

        assert!(matches!(result, Err(GitError::NotARepo(_))));
        std::fs::remove_dir_all(&td).unwrap();
    }
}
