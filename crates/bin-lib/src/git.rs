use git2::Repository;
use std::fmt;

#[derive(Debug)]
pub enum GitError {
    NotARepo(git2::Error),
    DetachedHead,
    Other(String),
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::NotARepo(_) => write!(f, "not a git repository"),
            GitError::DetachedHead => write!(f, "HEAD is detached"),
            GitError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for GitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GitError::NotARepo(e) => Some(e),
            _ => None,
        }
    }
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self {
        if e.class() == git2::ErrorClass::Repository && e.code() == git2::ErrorCode::NotFound {
            GitError::NotARepo(e)
        } else {
            GitError::Other(e.message().to_string())
        }
    }
}

pub fn open_repo() -> Result<Repository, GitError> {
    Ok(Repository::discover(".")?)
}

pub fn current_branch(repo: &Repository) -> Result<String, GitError> {
    let head = repo.head().map_err(|e| GitError::Other(e.message().to_string()))?;
    if !head.is_branch() {
        return Err(GitError::DetachedHead);
    }
    head.shorthand()
        .map(|s| s.to_string())
        .ok_or_else(|| GitError::Other("branch name is not valid UTF-8".to_string()))
}

pub fn get_current_branch() -> Result<String, GitError> {
    let repo = open_repo()?;
    current_branch(&repo)
}

pub fn is_dirty(repo: &Repository) -> Result<bool, GitError> {
    let statuses = repo
        .statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(true)
                .recurse_untracked_dirs(true),
        ))
        .map_err(|e| GitError::Other(e.message().to_string()))?;
    Ok(!statuses.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_a_repo_display() {
        let err = GitError::NotARepo(git2::Error::from_str("test"));
        assert_eq!(err.to_string(), "not a git repository");
    }

    #[test]
    fn detached_head_display() {
        let err = GitError::DetachedHead;
        assert_eq!(err.to_string(), "HEAD is detached");
    }

    #[test]
    fn clean_repo_is_not_dirty() {
        let td = std::env::temp_dir().join("bin-lib-test-clean");
        let _ = std::fs::remove_dir_all(&td);
        let repo = Repository::init(&td).unwrap();

        // empty repo with no files is clean
        assert!(!is_dirty(&repo).unwrap());

        std::fs::remove_dir_all(&td).unwrap();
    }

    #[test]
    fn repo_with_untracked_file_is_dirty() {
        let td = std::env::temp_dir().join("bin-lib-test-dirty");
        let _ = std::fs::remove_dir_all(&td);
        let repo = Repository::init(&td).unwrap();

        std::fs::write(td.join("new.txt"), "hello").unwrap();
        assert!(is_dirty(&repo).unwrap());

        std::fs::remove_dir_all(&td).unwrap();
    }
}
