use std::path::Path;

use rootproof_core::{RepositoryInfo, RootProofError};
use rootproof_git::is_git_repository;

use crate::detect_language;

pub fn inspect_repository(path: &Path) -> Result<RepositoryInfo, RootProofError> {
    if !path.exists() {
        return Err(RootProofError::RepositoryNotFound(path.to_path_buf()));
    }

    if !path.is_dir() {
        return Err(RootProofError::RepositoryNotDirectory(path.to_path_buf()));
    }

    let canonical_path =
        path.canonicalize()
            .map_err(|source| RootProofError::ResolveRepository {
                path: path.to_path_buf(),
                source,
            })?;

    let language = detect_language(&canonical_path);

    let is_git_repository = is_git_repository(&canonical_path);

    Ok(RepositoryInfo {
        path: canonical_path,
        is_git_repository,
        language,
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use rootproof_core::{Language, RootProofError};

    use super::*;

    #[test]
    fn rejects_missing_respository() {
        let path = Path::new("/definitely/not/a/rootproof/repository");

        let result = inspect_repository(path);
        assert!(matches!(result, Err(RootProofError::RepositoryNotFound(_))))
    }

    #[test]
    fn rejects_regular_file_as_respository() {
        let temp = tempfile::tempdir().expect("create temporary directory");

        let file = temp.path().join("production.log");
        fs::write(&file, "panic").expect("write file");

        let result = inspect_repository(&file);

        assert!(matches!(
            result,
            Err(RootProofError::RepositoryNotDirectory(_))
        ))
    }

    #[test]
    fn resolves_repository_to_absolute_path() {
        let temp = tempfile::tempdir().expect("create temporary directory");

        let info = inspect_repository(temp.path()).expect("inspect repository");

        assert!(info.path.is_absolute())
    }

    #[test]
    fn detects_rust_repository() {
        let temp = tempfile::tempdir().expect("create temporary repository");

        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
                [package]
                name = "example"
                version = "0.1.0"
                edition = "2024"
                "#,
        )
        .expect("write Cargo.toml");

        let info = inspect_repository(temp.path()).expect("inspect repository");

        assert_eq!(info.language, Some(Language::Rust))
    }

    #[test]
    fn detects_git_repository() {
        let temp = tempfile::tempdir().expect("create temporary directory");

        fs::create_dir(temp.path().join(".git")).expect("create .git directory");

        let info = inspect_repository(temp.path()).expect("inspect repository");

        assert!(info.is_git_repository)
    }

    #[test]
    fn reports_non_git_directory() {
        let temp = tempfile::tempdir().expect("create temporary directory");

        let info = inspect_repository(temp.path()).expect("inspect repository");

        assert!(!info.is_git_repository);
    }
}
