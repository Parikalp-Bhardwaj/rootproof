use rootproof_core::Language;
use rootproof_executor::{CommandResult, ExecutorError};
use std::path::Path;

pub trait LanguageAdapter {
    fn language(&self) -> Language;
    fn detect(&self, repo: &Path) -> bool;

    fn check(
        &self,
        repo: &Path,
    ) -> impl Future<Output = Result<CommandResult, ExecutorError>> + Send;

    fn test(
        &self,
        repo: &Path,
    ) -> impl Future<Output = Result<CommandResult, ExecutorError>> + Send;
}
