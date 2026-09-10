use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rootproof_core::{Incident, Language, read_incident_input};
use rootproof_language::{LanguageAdapter, RustAdapter, inspect_repository, parse_rust_failure};

#[derive(Debug, Parser)]
#[command(name = "rootproof")]
#[command(version)]
#[command(about = "Prove the root cause. Reproduce the failure. Validate the fix.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Investigate a production failure.
    Investigate {
        input: Option<PathBuf>,

        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },

    /// Run the Rust repository test suite.
    Test {
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },

    /// Configure RootProof.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Configure the AI model.
    Model,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(error) = run(cli).await {
        eprintln!("RootProof error: {error}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Investigate { input, repo } => {
            investigate(input, repo).await?;
        }

        Commands::Test { repo } => {
            run_tests(repo).await?;
        }

        Commands::Config { command } => {
            run_config(command);
        }
    }

    Ok(())
}

async fn investigate(
    input: Option<PathBuf>,
    repo: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let info = inspect_repository(&repo)?;

    println!("RootProof");
    println!();

    println!("Repository: {}", info.path.display());

    println!("Git repository: {}", yes_no(info.is_git_repository));

    match info.language {
        Some(Language::Rust) => {
            println!("Language: Rust");
        }

        None => {
            println!("Language: unsupported");

            return Ok(());
        }
    }

    let Some(input_path) = input else {
        println!();
        println!("No incident input provided.");

        return Ok(());
    };

    println!("Incident input: {}", input_path.display());

    let raw_input = read_incident_input(&input_path)?;

    let failure = parse_rust_failure(&raw_input, "");

    println!();

    match failure {
        Some(failure) => {
            println!("Production failure detected");
            println!();

            if let Some(error_type) = &failure.error_type {
                println!("Type: {error_type}");
            }

            if let Some(message) = &failure.message {
                println!("Message: {message}");
            }

            if let Some(file) = &failure.file {
                println!("File: {}", file.display());
            }

            if let Some(line) = failure.line {
                println!("Line: {line}");
            }

            if let Some(column) = failure.column {
                println!("Column: {column}");
            }

            let _incident = Incident {
                id: "RP-0001".to_owned(),
                repository: info.path.clone(),
                input_path: Some(input_path),
                raw_input,
                failure,
            };
        }

        None => {
            println!("No supported Rust failure detected.");
        }
    }

    Ok(())
}

fn run_config(command: ConfigCommand) {
    match command {
        ConfigCommand::Model => {
            println!("Model configuration is not implemented yet.");
        }
    }
}

async fn run_tests(repo: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let info = inspect_repository(&repo)?;

    if info.language != Some(Language::Rust) {
        return Err("repository is not a supported Rust project".into());
    }

    println!("RootProof");
    println!();

    println!("Repository: {}", info.path.display());

    println!("Language: Rust");
    println!();

    println!("Running: cargo test");
    println!();

    let adapter = RustAdapter;

    let result = adapter.test(&info.path).await?;

    println!("Status: {}", if result.success() { "PASS" } else { "FAIL" });

    println!("Exit code: {}", format_exit_code(result.exit_code));

    println!("Duration: {:.2?}", result.duration);

    if result.success() {
        println!();
        println!("Test suite completed successfully.");

        return Ok(());
    }

    if let Some(failure) = parse_rust_failure(&result.stdout, &result.stderr) {
        println!();
        println!("Failure detected:");

        println!(
            "Type: {}",
            failure.error_type.as_deref().unwrap_or("unknown")
        );

        println!(
            "Message: {}",
            failure.message.as_deref().unwrap_or("unknown")
        );

        if let Some(file) = &failure.file {
            print!("Location: {}", file.display());

            if let Some(line) = failure.line {
                print!(":{line}");

                if let Some(column) = failure.column {
                    print!(":{column}");
                }
            }

            println!();
        }

        if !failure.stack_frames.is_empty() {
            println!();
            println!("Stack frames: {}", failure.stack_frames.len());
        }
    } else {
        println!();
        println!("Test failed, but RootProof could not parse a supported Rust failure.");
    }

    Ok(())
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn format_exit_code(exit_code: Option<i32>) -> String {
    match exit_code {
        Some(code) => code.to_string(),
        None => "terminated by signal".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yes_no_formats_boolean() {
        assert_eq!(yes_no(true), "yes");
        assert_eq!(yes_no(false), "no");
    }

    #[test]
    fn formats_normal_exit_code() {
        assert_eq!(format_exit_code(Some(0)), "0");
    }

    #[test]
    fn formats_missing_exit_code() {
        assert_eq!(format_exit_code(None), "terminated by signal");
    }
}
