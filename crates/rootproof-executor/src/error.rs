use std::{
    io,
    path::PathBuf,
    time::Duration,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error(
        "working directory does not exist: {0}"
    )]
    WorkingDirectoryNotFound(PathBuf),

    #[error(
        "working directory is not a directory: {0}"
    )]
    WorkingDirectoryNotDirectory(PathBuf),

    #[error(
        "failed to start command `{program}`: {source}"
    )]
    Spawn {
        program: String,

        #[source]
        source: io::Error,
    },

    #[error(
        "command `{program}` timed out after {timeout:?}"
    )]
    Timeout {
        program: String,
        timeout: Duration,
    },
}