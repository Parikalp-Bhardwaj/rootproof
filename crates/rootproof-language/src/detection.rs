use crate::{LanguageAdapter, RustAdapter};
use rootproof_core::Language;
use std::path::Path;

pub fn detect_language(repo: &Path) -> Option<Language> {
    let rust = RustAdapter;

    if rust.detect(repo) {
        return Some(rust.language());
    }

    None
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn detects_rust_language() {
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

        assert_eq!(detect_language(temp.path()), Some(Language::Rust));
    }

    #[test]
    fn returns_none_for_unknown_project() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        assert_eq!(detect_language(temp.path()), None);
    }
}
