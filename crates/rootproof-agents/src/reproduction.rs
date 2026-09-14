use rootproof_ai::OpenRouterProvider;

use rootproof_core::{EvidenceBundle, Hypothesis, Incident, ReproductionCandidate, SourceAnalysis};

use schemars::JsonSchema;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReproductionOutput {
    pub test_name: String,
    pub test_code: String,
    pub rationale: String,
    pub expected_failure: String,
}

pub async fn generate_reproduction(
    provider: &OpenRouterProvider,
    incident: &Incident,
    evidence: &EvidenceBundle,
    source_analysis: &SourceAnalysis,
    hypothesis: &Hypothesis,
) -> Result<ReproductionCandidate, Box<dyn std::error::Error>> {
    let input = build_reproduction_input(incident, evidence, source_analysis, hypothesis);

    let preamble = r#"
                You are RootProof's Rust Reproduction Agent.

                Your task is to generate one minimal Rust test that attempts
                to reproduce the supplied hypothesis.

                Important rules:

                1. Use only the supplied incident, evidence, source findings,
                and selected hypothesis.

                2. Do not invent APIs, structs, functions, modules, fields,
                dependencies, files, or runtime behavior that are not present
                in the supplied evidence.

                3. Generate exactly one Rust #[test] function.

                4. The generated test should be as small as possible.

                5. The purpose of the test is to reproduce the observed failure,
                not to fix it.

                6. Do not modify production code.

                7. Do not generate a patch.

                8. Do not use shell commands.

                9. Do not use network access.

                10. Do not use filesystem access unless the evidence explicitly
                    shows that filesystem behavior is required.

                11. Do not add dependencies.

                12. Do not use unsafe Rust.

                13. Do not claim the hypothesis is proven.

                14. The test_code field must contain raw Rust source only.

                15. Do not wrap test_code in Markdown code fences.

                16. expected_failure should describe what RootProof should expect
                    to observe if this hypothesis is correct.

                17. Prefer calling existing repository functions directly.

                18. If the available evidence is insufficient to create a valid
                    test without inventing repository details, generate the smallest
                    test possible using only known symbols and explain the limitation
                    in rationale.

                Return exactly one reproduction candidate.
                "#;

    let output: ReproductionOutput = provider.extract(&input, preamble).await?;
    Ok(ReproductionCandidate {
        hypothesis_id: hypothesis.id.clone(),
        test_name: output.test_name,
        test_code: output.test_code,
        rationale: output.rationale,
        expected_failure: output.expected_failure,
    })
}

fn build_reproduction_input(
    incident: &Incident,
    evidence: &EvidenceBundle,
    source_analysis: &SourceAnalysis,
    hypothesis: &Hypothesis,
) -> String {
    let mut input = String::new();

    input.push_str("INCIDENT\n");

    input.push_str(&format!("Incident ID: {}\n", incident.id));

    if let Some(error_type) = &incident.failure.error_type {
        input.push_str(&format!("Failure type: {error_type}\n"));
    }

    if let Some(message) = &incident.failure.message {
        input.push_str(&format!("Failure message: {message}\n"));
    }

    if let Some(file) = &incident.failure.file {
        input.push_str(&format!("Failure file: {}\n", file.display()));
    }

    if let Some(line) = incident.failure.line {
        input.push_str(&format!("Failure line: {line}\n"));
    }

    if let Some(column) = incident.failure.column {
        input.push_str(&format!("Failure column: {column}\n"));
    }

    input.push_str("\nSELECTED HYPOTHESIS\n");
    input.push_str(&format!("ID: {}\n", hypothesis.id));
    input.push_str(&format!("Description: {}\n", hypothesis.description));
    input.push_str(&format!("Confidence: {:.2}\n", hypothesis.confidence));

    input.push_str(&format!(
        "Supporting evidence: {}\n",
        hypothesis.evidence_ids.join(", ")
    ));

    input.push_str("\nEVIDENCE\n");

    for item in &evidence.evidence {
        input.push_str(&format!("\nEvidence ID: {}\n", item.id));

        input.push_str(&format!("Kind: {:?}\n", item.kind));

        if let Some(file) = &item.file {
            input.push_str(&format!("File: {}\n", file.display()));
        }

        if let Some(line) = item.line {
            input.push_str(&format!("Line: {line}\n"));
        }

        input.push_str("Content:\n");

        input.push_str(&item.content);
        input.push('\n');
    }

    input.push_str("\nSOURCE FINDINGS\n");

    for finding in &source_analysis.findings {
        input.push_str(&format!("\nFinding type: {}\n", finding.finding_type,));

        if let Some(expression) = &finding.expression {
            input.push_str(&format!("Expression: {expression}\n"));
        }

        input.push_str(&format!("Reason: {}\n", finding.reason,));
    }

    input
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reproduction_output_deserializes() {
        let json = r#"
        {
            "test_name": "reproduces_divide_expectation",
            "test_code": '#[test]n fn reproduces_divide_expectation() {\n    assert_eq!(divide(10, 2), 10);\n}',
            "rationale": "Replays the failing assertion.",
            "expected_failure": "assertion left == right fails"
        }"#;

        let output: ReproductionOutput =
            serde_json::from_str(json).expect("deserialize reproduction output");

        assert_eq!(output.test_name, "reproduces_divide_expectation");

        assert!(output.test_code.contains("#[test]"));
    }
}
