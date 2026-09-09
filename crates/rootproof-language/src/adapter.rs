use rootproof_core::Language;
use std::path::Path;

pub trait LanguageAdapter {
    fn language(&self) -> Language;
    fn detect(&self, repo: &Path) -> bool;
}
