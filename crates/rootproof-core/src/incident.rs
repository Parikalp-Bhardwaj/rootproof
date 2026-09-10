use std::path::PathBuf;

use crate::FailureSignature;

#[derive(Debug, Clone)]
pub struct Incident {
    pub id: String,
    pub repository: PathBuf,
    pub input_path: Option<PathBuf>,
    pub raw_input: String,
    pub failure: FailureSignature,
}
