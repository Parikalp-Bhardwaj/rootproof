use std::path::Path;

use crate::LanguageAdapter;
use rootproof_core::Language;

#[derive(Debug, Default)]
pub struct RustAdapter;

impl LanguageAdapter for RustAdapter {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn detect(&self, repo: &Path) -> bool {
        repo.join("Cargo.toml").is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detect_rust_project_when_cargo_toml_exists() {
        let temp = tempfile::tempdir().expect("create temporary directory");

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

        let adapter = RustAdapter;

        assert!(adapter.detect(temp.path()));
        assert_eq!(adapter.language(), Language::Rust)
    }

    #[test]
    fn does_not_detect_rust_without_cargo_toml() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        let adapter = RustAdapter;

        assert!(!adapter.detect(temp.path()))
    }

    #[test]
    fn cargo_toml_must_be_a_file() {
        let temp = tempfile::tempdir().expect("create temporary directory");

        fs::create_dir(temp.path().join("Cargo.toml")).expect("create Cargo.toml directory");

        let adapter = RustAdapter;

        assert!(!adapter.detect(temp.path()));
    }
}
