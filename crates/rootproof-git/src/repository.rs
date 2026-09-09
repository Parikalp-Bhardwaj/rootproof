use std::path::Path;

/// Returns true when the provided path appears to be a Git repository root.
///
/// Stage 2 intentionally performs only filesystem-based detection.
/// Later, RootProof can use `git rev-parse --show-toplevel` through the
/// safe command execution layer.
pub fn is_git_repository(repo: &Path) -> bool {
    repo.join(".git").exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detect_directory_with_doc_git() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        fs::create_dir(temp.path().join(".git")).expect("create .git directory");
        assert!(is_git_repository(temp.path()));
    }

    #[test]
    fn does_not_detect_directory_without_dot_git() {
        let temp = tempfile::tempdir().expect("create temporary directory");

        assert!(!is_git_repository(temp.path()));
    }
}
