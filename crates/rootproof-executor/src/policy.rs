use std::{fmt, path::Path, time::Duration};

use crate::CommandSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedCommand {
    CargoCheck,
    CargoTest,
    CargoClippy,
    CargoMetadata,
}

impl AllowedCommand {
    pub fn program(self) -> &'static str {
        "cargo"
    }

    pub fn args(self) -> &'static [&'static str] {
        match self {
            Self::CargoCheck => &["check"],
            Self::CargoTest => &["test"],
            Self::CargoClippy => &["clippy"],
            Self::CargoMetadata => &["metadata"],
        }
    }
}

impl fmt::Display for AllowedCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CargoCheck => write!(f, "cargo check"),
            Self::CargoTest => write!(f, "cargo test"),
            Self::CargoClippy => write!(f, "cargo clippy"),
            Self::CargoMetadata => write!(f, "cargo metadata"),
        }
    }
}

pub fn command_spec(command: AllowedCommand, repo: &Path) -> CommandSpec {
    CommandSpec::new(command.program(), repo)
        .args(command.args().iter().copied())
        .timeout(Duration::from_secs(120))
}
