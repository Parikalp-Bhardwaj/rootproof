use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub program: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

impl CommandResult {
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }
}