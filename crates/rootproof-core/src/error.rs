use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RootProofError {
    #[error("repository path does not exist: {0}")]
    RepositoryNotFound(PathBuf),

    #[error("repository path is not a directory: {0}")]
    RepositoryNotDirectory(PathBuf),

    #[error("falied to resolve repository path {path}: {source}")]
    ResolveRepository {
        path: PathBuf,

        #[source]
        source: std::io::Error,
    },

    #[error("failed to read incident input {path}: {source}")]
    ReadIncidentInput {
        path: PathBuf,

        #[source]
        source: std::io::Error,
    },
}
