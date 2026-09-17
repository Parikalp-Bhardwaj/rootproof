use rootproof_ai::OpenRouterProvider;
use rootproof_core::{CandidateFix, EvidenceBundle, Hypothesis, Incident, SourceAnalysis};
use schemars::JsonSchema;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CandidateFixOutput {
    pub target_file: String,
    pub original_code: String,
    pub replacement_code: String,
    pub rationale: String,
}

pub async fn generate_candidate_fix(
    provider: &OpenRouterProvider,
    incident: &Incident,
    evidence: &EvidenceBundle,
    source_analysis: &SourceAnalysis,
    hypothesis: &Hypothesis,
) -> Result<CandidateFix, Box<dyn std::error::Error>> {
    let input = build_fix_input(incident, evidence, source_analysis, hypothesis);

    let preamble = r#"
        You are RootProof's Candidate Fix Agent.

        Your job is to propose ONE minimal fix for the supplied,
        strongly reproduced hypothesis.

        STRICT RULES:

        1. Use only the supplied incident, deterministic evidence,
        source findings, and hypothesis.

        2. Produce exactly one minimal source-code replacement.

        3. target_file must be repository-relative.

        4. original_code must be copied exactly from code present in
        the supplied evidence.

        5. replacement_code must contain only the smallest code change
        required to address the hypothesis.

        6. Do not rewrite entire files.

        7. Do not add unrelated refactoring.

        8. Do not add dependencies.

        9. Do not invent APIs, functions, structs, fields, modules,
        behavior, or files.

        10. Do not use shell commands.

        11. Do not use unsafe Rust unless the existing code already
            requires it and the supplied evidence explicitly supports it.

        12. Do not claim the fix is validated.

        13. Do not include Markdown code fences in original_code or
            replacement_code.

        14. The proposed change must be testable by the existing
            repository test suite.

        15. If the hypothesis concerns an incorrect test expectation,
            modify only the incorrect expectation.

        16. If the evidence is insufficient to safely propose a fix,
            do not fabricate implementation details.

        Return one structured candidate fix.
        "#;

    let output: CandidateFixOutput = provider.extract(&input, preamble).await?;

    Ok(CandidateFix {
        hypothesis_id: hypothesis.id.clone(),
        target_file: PathBuf::from(output.target_file),
        original_code: output.original_code,
        replacement_code: output.replacement_code,
        rationale: output.rationale,
    })
}

fn build_fix_input(
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
    input.push_str("\nSELECTED HYPOTHESIS\n");
    input.push_str(&format!("ID: {}\n", hypothesis.id));
    input.push_str(&format!("Description: {}\n", hypothesis.description));

    input.push_str("\nEVIDENCE\n");

    for item in &evidence.evidence {
        input.push_str(&format!("\nEvidence ID: {}\n", item.id));

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
        input.push_str(&format!("\nType: {}\n", finding.finding_type));

        if let Some(expression) = &finding.expression {
            input.push_str(&format!("Expression: {expression}\n"));
        }

        input.push_str(&format!("Reason: {}\n", finding.reason));
    }

    input
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_fix_output_deserializes() {
        let value = serde_json::json!({
            "target_file": "src/lib.rs",
            "original_code": "assert_eq!(divide(10, 2), 10);",
            "replacement_code": "assert_eq!(divide(10, 2), 5);",
            "rationale": "Correct the expected division result."
        });

        let output: CandidateFixOutput = serde_json::from_value(value).expect("deserialize fix");
        assert_eq!(output.target_file, "src/lib.rs");
        assert_eq!(output.replacement_code, "assert_eq!(divide(10, 2), 5);");
    }
}
