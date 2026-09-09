use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rootproof_core::Language;
use rootproof_language::inspect_repository;

#[derive(Debug, Parser)]
#[command(name = "description")]
#[command(version)]
#[command(about = "Prove the root cause. Reproduce the failure. Validate the fix.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Investigate a production log or failure evidence.
    Investigate {
        input: Option<PathBuf>,

        /// Repository to investigate.
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },

    /// Configure RootProof
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Model,
}

fn main() {
    let cli = Cli::parse();

    if let Err(error) = run(cli) {
        eprintln!("RootProof error: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Investigate { input, repo } => {
            investigate(input, repo)?;
        }

        Commands::Config { command } => run_config(command),
    }

    Ok(())
}

fn investigate(input: Option<PathBuf>, repo: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let info = inspect_repository(&repo)?;

    println!("RootProof");
    println!();

    println!("Repository: {}", info.path.display());

    println!("Git repository: {}", yes_no(info.is_git_repository));

    match info.language {
        Some(Language::Rust) => {
            println!("Language: Rust");
            println!("Cargo project: yes");
        }

        None => {
            println!("Language: unsupported");
            println!("Cargo project: no");
        }
    }

    if let Some(input) = input {
        println!("Incident input: {}", input.display());
    }

    println!();
    println!("Repository inspection complete.");

    Ok(())
}

fn run_config(command: ConfigCommand) {
    match command {
        ConfigCommand::Model => {
            println!("Model configuration is not implemented yet.");
        }
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yes_no_returns_yes_for_true() {
        assert_eq!(yes_no(true), "yes");
    }

    #[test]
    fn yes_no_returns_no_for_false() {
        assert_eq!(yes_no(false), "no");
    }
}
