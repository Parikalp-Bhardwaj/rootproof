use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rootproof_agents::{
    analyze_source, generate_hypotheses, generate_reproduction, regenerate_reproduction,
};
use rootproof_ai::{OpenRouterProvider, RootProofConfig, load_config, save_config};
use rootproof_core::{
    Evidence, EvidenceBundle, EvidenceKind, Incident, Language, ReproductionStatus,
    compare_failure_signatures, read_incident_input,
};
use rootproof_executor::{CommandSpec, create_isolated_repository, execute};
use rootproof_language::{
    LanguageAdapter, RustAdapter, inject_reproduction_test, inspect_repository,
    normalize_rust_reproduction_code, parse_rust_failure, prepare_rust_reproduction_code,
    read_source_context, resolve_failure_file, validate_rust_reproduction_code,
};

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

    /// Test the configured AI provider.
    AiTest,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Configure the AI model.
    Model {
        /// OpenRouter model identifier.
        #[arg(long)]
        model: String,
    },
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
            run_config(command).await?;
        }

        Commands::AiTest => {
            run_ai_test().await?;
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
            println!("Language: Rust")
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

    let Some(failure) = failure else {
        println!("No supported Rust failure detected.");
        return Ok(());
    };

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

    let incident = Incident {
        id: "RP-0001".to_owned(),
        repository: info.path.clone(),

        input_path: Some(input_path),
        raw_input,
        failure: failure.clone(),
    };

    let mut evidence = EvidenceBundle::new();
    evidence.push(Evidence {
        id: "E1".to_owned(),
        kind: EvidenceKind::Panic,
        file: failure.file.clone(),
        line: failure.line,
        content: failure.message.clone().unwrap_or_default(),
    });

    if let (Some(file), Some(line)) = (&failure.file, failure.line) {
        if let Some(relative_file) = resolve_failure_file(&info.path, file) {
            let source = read_source_context(&info.path, &relative_file, line, 5)?;
            println!();
            println!("Source context:");
            println!();

            println!("{}", source.content);

            evidence.push(Evidence {
                id: "E2".to_owned(),
                kind: EvidenceKind::Source,

                file: Some(source.file.clone()),

                line: Some(source.target_line),
                content: source.content.clone(),
            });
        } else {
            println!();

            println!("Source file could not be resolved.");
        }
    }

    println!();

    println!("Evidence collected: {}", evidence.evidence.len());
    let config = load_config()?;

    let provider = OpenRouterProvider::new(config.ai)?;

    println!();

    println!("Analyzing source evidence...");

    let source_analysis = analyze_source(&provider, &evidence).await?;
    println!();

    println!("Source findings:");

    if source_analysis.findings.is_empty() {
        println!("No source findings returned.");
    } else {
        for finding in &source_analysis.findings {
            println!();

            println!("- Type: {}", finding.finding_type);

            if let Some(expression) = &finding.expression {
                println!("  Expression: {expression}");
            }

            println!("  Reason: {}", finding.reason);
        }
    }
    println!();

    println!("Generating root-cause hypotheses...");

    let hypotheses = generate_hypotheses(&provider, &incident, &evidence, &source_analysis).await?;
    println!();

    println!("Hypotheses:");

    if hypotheses.is_empty() {
        println!("No hypotheses generated.");

        return Ok(());
    }

    for hypothesis in &hypotheses {
        println!();

        println!("{}", hypothesis.id);

        println!("Description: {}", hypothesis.description);

        println!("Confidence: {:.0}%", hypothesis.confidence * 100.0);

        if hypothesis.evidence_ids.is_empty() {
            println!("Evidence: none");
        } else {
            println!("Evidence: {}", hypothesis.evidence_ids.join(", "));
        }

        println!("Status: UNCONFIRMED");
    }

    let strongest_hypothesis = &hypotheses[0];

    println!();

    println!("Generating reproduction for {}...", strongest_hypothesis.id);

    let mut reproduction = generate_reproduction(
        &provider,
        &incident,
        &evidence,
        &source_analysis,
        strongest_hypothesis,
    )
    .await?;

    const MAX_REPRODUCTION_ATTEMPTS: usize = 3;

    let mut normalized_test_code = None;

    for attempt in 1..=MAX_REPRODUCTION_ATTEMPTS {
        println!();

        println!("Reproduction candidate:");

        println!();

        println!("Attempt: {attempt}/{MAX_REPRODUCTION_ATTEMPTS}");

        println!("Hypothesis: {}", reproduction.hypothesis_id);

        println!("Test: {}", reproduction.test_name);

        println!("Rationale: {}", reproduction.rationale);

        println!("Expected failure: {}", reproduction.expected_failure);

        println!();

        println!("Generated Rust:");

        println!();

        println!("{}", reproduction.test_code);

        println!();

        let normalized = match normalize_rust_reproduction_code(
            &reproduction.test_code,
            &reproduction.test_name,
        ) {
            Ok(code) => code,

            Err(error) => {
                println!("Reproduction validation: REJECTED");

                println!("Reason: {error}");

                if attempt == MAX_REPRODUCTION_ATTEMPTS {
                    println!("Reproduction status: REJECTED");

                    return Ok(());
                }

                println!();

                println!("Regenerating reproduction...");

                reproduction = regenerate_reproduction(
                    &provider,
                    &incident,
                    &evidence,
                    &source_analysis,
                    strongest_hypothesis,
                    &reproduction,
                    &error,
                )
                .await?;

                continue;
            }
        };

        match validate_rust_reproduction_code(&normalized) {
            Ok(()) => {
                normalized_test_code = Some(normalized);

                println!("Reproduction syntax: VALID");
                break;
            }

            Err(error) => {
                println!("Reproduction validation: REJECTED");

                println!("Reason: {error}");

                if attempt == MAX_REPRODUCTION_ATTEMPTS {
                    println!("Reproduction status: REJECTED");

                    return Ok(());
                }

                println!();

                println!("Regenerating reproduction...");

                reproduction = regenerate_reproduction(
                    &provider,
                    &incident,
                    &evidence,
                    &source_analysis,
                    strongest_hypothesis,
                    &reproduction,
                    &error,
                )
                .await?;
            }
        }
    }

    let Some(normalized_test_code) = normalized_test_code else {
        println!("Reproduction status: REJECTED");

        return Ok(());
    };

    println!("Reproduction syntax: VALID");

    let execution_test_name = format!(
        "rootproof_reproduction_{}",
        strongest_hypothesis.id.to_lowercase()
    );

    let prepared_code = prepare_rust_reproduction_code(&normalized_test_code, &execution_test_name)
        .map_err(std::io::Error::other)?;

    println!("Execution test: {}", execution_test_name);

    let Some(production_file) = &incident.failure.file else {
        println!("Reproduction status: UNCONFIRMED");

        println!("Reason: production failure has no source file");

        return Ok(());
    };

    let Some(relative_source_file) = resolve_failure_file(&info.path, production_file) else {
        println!("Reproduction status: UNCONFIRMED");

        println!("Reason: unable to resolve reproduction source file");

        return Ok(());
    };

    println!();

    println!("Creating isolated reproduction environment...");

    let isolated = create_isolated_repository(&info.path)?;

    println!("Isolation: READY");

    inject_reproduction_test(isolated.path(), &relative_source_file, &prepared_code)?;

    println!("Generated test injected into isolated repository.");

    println!();

    println!("Executing reproduction...");

    let command = CommandSpec::new("cargo", isolated.path())
        .arg("test")
        .arg(&execution_test_name)
        .arg("--")
        .arg("--nocapture");

    let result = execute(command).await?;

    println!(
        "Exit code: {}",
        result
            .exit_code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "unknown".to_owned())
    );

    if result.success() {
        println!("Generated reproduction did not fail.");
        println!("Reproduction status: UNCONFIRMED");

        return Ok(());
    }

    let reproduction_failure = parse_rust_failure(&result.stdout, &result.stderr);

    let Some(reproduction_failure) = reproduction_failure else {
        println!("Generated test failed, but RootProof could not parse a supported Rust failure.");
        println!("Reproduction status: UNCONFIRMED");

        return Ok(());
    };

    println!();

    println!("Reproduction failure:");

    if let Some(error_type) = &reproduction_failure.error_type {
        println!("Type: {error_type}");
    }

    if let Some(message) = &reproduction_failure.message {
        println!("Message: {message}");
    }

    if let Some(file) = &reproduction_failure.file {
        println!("File: {}", file.display());
    }

    if let Some(line) = reproduction_failure.line {
        println!("Line: {line}");
    }

    let failure_match = compare_failure_signatures(&incident.failure, &reproduction_failure);

    println!();

    println!("Failure signature comparison:");

    println!(
        "Error type match: {}",
        yes_no(failure_match.error_type_match)
    );

    println!("Message match: {}", yes_no(failure_match.message_match));

    println!("Match score: {:.0}%", failure_match.score * 100.0);

    println!();

    match failure_match.status {
        ReproductionStatus::StronglyReproduced => {
            println!("Reproduction status: STRONGLY_REPRODUCED");
        }

        ReproductionStatus::PartiallyReproduced => {
            println!("Reproduction status: PARTIALLY_REPRODUCED");
        }

        ReproductionStatus::Unconfirmed => {
            println!("Reproduction status: UNCONFIRMED");
        }
    }

    println!();

    println!(
        "RootProof reproduced the failure behavior; this does not yet prove the proposed fix."
    );

    Ok(())
}

async fn run_config(command: ConfigCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ConfigCommand::Model { model } => {
            let config = RootProofConfig::openrouter(model.clone());

            let path = save_config(&config)?;

            println!("RootProof AI");
            println!();

            println!("Provider: OpenRouter");

            println!("Model: {model}");

            println!();
            println!("Configuration saved.");

            println!("Config: {}", path.display());
        }
    }

    Ok(())
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

async fn run_ai_test() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;

    println!("RootProof AI");
    println!();

    println!("Provider: {}", config.ai.provider);

    println!("Model: {}", config.ai.model);

    println!();
    println!("Testing connection...");

    let provider = OpenRouterProvider::new(config.ai)?;

    let response = provider.prompt("Reply with exactly: ROOTPROOF_OK").await?;

    println!();
    println!("Response received:");
    println!("{response}");

    Ok(())
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
