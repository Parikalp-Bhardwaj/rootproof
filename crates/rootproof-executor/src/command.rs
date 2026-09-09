use std::{
    path::PathBuf,
    time::Duration,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec{
    pub program: String,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
    pub timeout: Duration
}

impl CommandSpec {
    pub fn new(
        program: impl Into<String>,
        working_directory: impl Into<PathBuf>) -> Self{
        
        Self { 
            program: program.into(), 
            args: Vec::new(), 
            working_directory: working_directory.into(), 
            timeout: Duration::from_secs(120) 
        }
    }

    pub fn arg(
        mut self,
        arg: impl Into<String>,
    ) -> Self{
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(
        mut self,
        args: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: Into<String>,
    {
        self.args.extend(
            args.into_iter().map(Into::into),
        );

        self
    }

    pub fn timeout(
        mut self,
        timeout: Duration,
    ) -> Self {
        self.timeout = timeout;
        self
    }
}