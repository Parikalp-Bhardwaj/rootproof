use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    pub function: Option<String>,
    pub file: Option<PathBuf>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureSignature {
    pub error_type: Option<String>,
    pub message: Option<String>,
    pub file: Option<PathBuf>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub stack_frames: Vec<StackFrame>,
}

impl FailureSignature {
    pub fn empty() -> Self {
        Self {
            error_type: None,
            message: None,
            file: None,
            line: None,
            column: None,
            stack_frames: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.error_type.is_none()
            && self.message.is_none()
            && self.file.is_none()
            && self.line.is_none()
            && self.column.is_none()
            && self.stack_frames.is_empty()
    }
}
