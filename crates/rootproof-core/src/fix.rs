use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateFix {
    pub hypothesis_id: String,
    pub target_file: PathBuf,
    pub original_code: String,
    pub replacement_code: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixValidationStatus {
    Validated,
    Rejected,
}
