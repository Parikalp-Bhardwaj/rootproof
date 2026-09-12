use std::io;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("OPENROUTER_API_KEY environment variable is not set")]
    MissingApiKey,

    #[error("AI model is not configured")]
    MissingModel,

    #[error("OpenRouter request failed: {0}")]
    Provider(String),

    #[error("RootProof config directory is unavailable")]
    ConfigDirectoryUnavailable,

    #[error("failed to access config path {path}: {source}")]
    ConfigIo {
        path: PathBuf,

        #[source]
        source: io::Error,
    },

    #[error("failed to serialize RootProof config: {0}")]
    ConfigSerialize(String),

    #[error("failed to parse RootProof config: {0}")]
    ConfigParse(String),
}
