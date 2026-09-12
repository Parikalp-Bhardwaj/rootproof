use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceKind {
    Panic,
    Source,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    pub id: String,
    pub kind: EvidenceKind,
    pub file: Option<PathBuf>,
    pub line: Option<u32>,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct EvidenceBundle {
    pub evidence: Vec<Evidence>,
}

impl EvidenceBundle {
    pub fn new() -> Self {
        Self {
            evidence: Vec::new(),
        }
    }

    pub fn push(&mut self, evidence: Evidence) {
        self.evidence.push(evidence);
    }
}
