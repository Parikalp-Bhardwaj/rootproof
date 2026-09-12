use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceContext {
    pub file: PathBuf,
    pub target_line: u32,
    pub start_line: u32,
    pub end_line: u32,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFinding {
    pub finding_type: String,
    pub expression: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAnalysis {
    pub file: PathBuf,
    pub findings: Vec<SourceFinding>,
}
