use std::path::PathBuf;
pub mod error;
pub mod failure;
pub mod incident;
pub mod input;
pub mod repository;

pub use error::RootProofError;
pub use failure::{FailureSignature, StackFrame};
pub use incident::Incident;
pub use input::read_incident_input;
pub use repository::{Language, RepositoryInfo};

#[derive(Debug, Clone)]
pub struct Repository {
    pub path: PathBuf,
}

impl Repository {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_stores_path() {
        let repo = Repository::new("./example");

        assert_eq!(repo.path, PathBuf::from("./example"));
    }
}
