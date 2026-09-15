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

        Your ONLY goal is to reproduce the OBSERVED PRODUCTION FAILURE.

        You are NOT validating the proposed correct behavior.
        You are NOT testing a fix.
        You are NOT writing what the code should do.

        The reproduction test must attempt to trigger the same failure
        signature observed in the incident.

        STRICT RULES:

        1. Use only the supplied incident, deterministic evidence,
        source findings, and selected hypothesis.

        2. Generate exactly one #[test] function.

        3. The test must exercise EXISTING repository code.

        4. NEVER redefine, reimplement, mock, shadow, duplicate, or replace
        production functions, structs, enums, traits, modules, constants,
        statics, or types.

        5. If the incident contains a concrete failing operation or assertion,
        preserve that failing behavior in the reproduction.

        6. Do NOT change an observed failing assertion into the expected
        correct behavior.

        For example, if the incident contains:

            assert_eq!(divide(10, 2), 10);

        do NOT generate:

            assert_eq!(divide(10, 2), 5);

        because that tests corrected behavior rather than reproducing
        the observed failure.

        7. The generated test should attempt to fail in the same way
        as the incident.

        8. Do not generate a fix.

        9. Do not generate a regression test for the proposed fix.

        10. Do not invent repository behavior or claim an actual return
            value unless deterministic evidence supplies it.

        11. Do not add dependencies.

        12. Do not use shell commands.

        13. Do not use network access.

        14. Do not use unsafe Rust.

        15. test_code must contain raw Rust source only.

        16. Do not wrap test_code in Markdown fences.

        17. If there is insufficient evidence to reproduce the failure
            without inventing details, say so through the rationale and
            generate only what can be supported by the supplied evidence.

        The purpose of this agent is FAILURE REPRODUCTION, not correctness validation.
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

pub async fn regenerate_reproduction(
    provider: &OpenRouterProvider,
    incident: &Incident,
    evidence: &EvidenceBundle,
    source_analysis: &SourceAnalysis,
    hypothesis: &Hypothesis,
    previous_candidate: &ReproductionCandidate,
    rejection_reason: &str,
) -> Result<ReproductionCandidate, Box<dyn std::error::Error>> {
    let mut input = build_reproduction_input(incident, evidence, source_analysis, hypothesis);

    input.push_str("\nPREVIOUS REPRODUCTION WAS REJECTED\n");

    input.push_str("\nPrevious test code:\n");

    input.push_str(&previous_candidate.test_code);

    input.push_str("\n\nDeterministic rejection reason:\n");

    input.push_str(rejection_reason);

    input.push_str(
        r#"

        Generate a corrected reproduction.

        You MUST correct the deterministic validation error.

        Do not work around the validator.

        Do not redefine any repository function or production implementation.

        The reproduction must exercise the existing repository code.
        "#,
    );

    let preamble = r#"
        You are RootProof's Rust Reproduction Agent.

        Your ONLY goal is to reproduce the OBSERVED PRODUCTION FAILURE.

        You are NOT validating the proposed correct behavior.
        You are NOT testing a fix.
        You are NOT writing what the code should do.

        The reproduction test must attempt to trigger the same failure
        signature observed in the incident.

        STRICT RULES:

        1. Use only the supplied incident, deterministic evidence,
        source findings, and selected hypothesis.

        2. Generate exactly one #[test] function.

        3. The test must exercise EXISTING repository code.

        4. NEVER redefine, reimplement, mock, shadow, duplicate, or replace
        production functions, structs, enums, traits, modules, constants,
        statics, or types.

        5. If the incident contains a concrete failing operation or assertion,
        preserve that failing behavior in the reproduction.

        6. Do NOT change an observed failing assertion into the expected
        correct behavior.

        For example, if the incident contains:

            assert_eq!(divide(10, 2), 10);

        do NOT generate:

            assert_eq!(divide(10, 2), 5);

        because that tests corrected behavior rather than reproducing
        the observed failure.

        7. The generated test should attempt to fail in the same way
        as the incident.

        8. Do not generate a fix.

        9. Do not generate a regression test for the proposed fix.

        10. Do not invent repository behavior or claim an actual return
            value unless deterministic evidence supplies it.

        11. Do not add dependencies.

        12. Do not use shell commands.

        13. Do not use network access.

        14. Do not use unsafe Rust.

        15. test_code must contain raw Rust source only.

        16. Do not wrap test_code in Markdown fences.

        17. If there is insufficient evidence to reproduce the failure
            without inventing details, say so through the rationale and
            generate only what can be supported by the supplied evidence.

        The purpose of this agent is FAILURE REPRODUCTION, not correctness validation.
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
        let value = serde_json::json!({
            "test_name":
                "reproduces_divide_expectation",

            "test_code":
                r#"#[test]
                    fn reproduces_divide_expectation() {
                        assert_eq!(divide(10, 2), 10);
                    }"#,

            "rationale":
                "Replays the observed failing assertion using the existing divide function.",

            "expected_failure":
                "The assertion left == right fails."
        });

        let output: ReproductionOutput =
            serde_json::from_value(value).expect("deserialize reproduction output");

        assert_eq!(output.test_name, "reproduces_divide_expectation");

        assert!(output.test_code.contains("#[test]"));

        assert!(output.test_code.contains("assert_eq!(divide(10, 2), 10)"));

        assert_eq!(
            output.rationale,
            "Replays the observed failing assertion using the existing divide function."
        );

        assert_eq!(
            output.expected_failure,
            "The assertion left == right fails."
        );
    }
}
