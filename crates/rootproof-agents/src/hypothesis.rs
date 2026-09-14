use rootproof_ai::OpenRouterProvider;

use rootproof_core::{EvidenceBundle, Hypothesis, Incident, SourceAnalysis};

use schemars::JsonSchema;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HypothesisOutput {
    pub hypotheses: Vec<HypothesisItemOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HypothesisItemOutput {
    pub description: String,

    /// Ranking signal between 0.0 and 1.0.
    ///
    /// This is NOT proof.
    pub confidence: f32,

    /// IDs of deterministic evidence supporting
    /// this hypothesis.
    pub evidence_ids: Vec<String>,
}

pub async fn generate_hypotheses(
    provider: &OpenRouterProvider,
    incident: &Incident,
    evidence: &EvidenceBundle,
    source_analysis: &SourceAnalysis,
) -> Result<Vec<Hypothesis>, Box<dyn std::error::Error>> {
    let input = build_hypothesis_input(incident, evidence, source_analysis);

    let preamble = r#"
                    You are RootProof's Hypothesis Agent.

                    Your job is to generate possible root-cause hypotheses
                    from deterministic evidence that has already been collected.

                    Rules:

                    1. Use only the supplied incident, evidence, and source findings.
                    2. Do not invent files, functions, values, logs, stack frames,
                    runtime state, environmental behavior, or concurrency behavior.
                    3. Do not generate fixes.
                    4. Do not generate reproduction tests.
                    5. Do not claim that any hypothesis is proven.
                    6. Confidence is only a ranking signal, not probability of truth.
                    7. Every hypothesis must reference evidence IDs that actually support it.
                    8. Generate only distinct hypotheses with materially different root causes.
                    9. Do not create multiple hypotheses that are just different wording
                    of the same underlying cause.
                    10. Do not generate speculative alternatives merely to increase the count.
                    11. If the evidence strongly supports only one hypothesis, return only one.
                    12. If there are two meaningful alternatives, return only two.
                    13. Never generate a hypothesis unsupported by the supplied evidence.
                    14. Prefer fewer high-quality hypotheses over many weak hypotheses.
                    15. A hypothesis must be specific enough to test experimentally later.

                    Normally return 1 to 3 hypotheses.

                    Return more than 3 only when the evidence genuinely supports
                    more than 3 materially distinct explanations.

                    Rank the returned hypotheses from strongest to weakest.
                    "#;

    let output: HypothesisOutput = provider.extract(&input, preamble).await?;

    let mut hypotheses = Vec::new();

    for (index, item) in output.hypotheses.into_iter().enumerate() {
        hypotheses.push(Hypothesis {
            id: format!("H{}", index + 1),

            description: item.description,
            confidence: item.confidence.clamp(0.0, 1.0),
            evidence_ids: item.evidence_ids,
        });
    }

    hypotheses.sort_by(|left, right| {
        right
            .confidence
            .partial_cmp(&left.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (index, hypothesis) in hypotheses.iter_mut().enumerate() {
        hypothesis.id = format!("H{}", index + 1);
    }

    Ok(hypotheses)
}

fn build_hypothesis_input(
    incident: &Incident,
    evidence: &EvidenceBundle,
    source_analysis: &SourceAnalysis,
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
        input.push_str(&format!("\nFinding type: {}\n", finding.finding_type));

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
    fn hypothesis_output_deserializes() {
        let json = r#"
        {
            "hypotheses": [
                {
                    "description": "The test expectation is incorrect.",
                    "confidence": 0.92,
                    "evidence_ids": ["E1", "E2"]
                }
            ]
        }
        "#;

        let output: HypothesisOutput =
            serde_json::from_str(json).expect("deserialize hypothesis output");

        assert_eq!(output.hypotheses.len(), 1);

        assert_eq!(output.hypotheses[0].confidence, 0.92);

        assert_eq!(
            output.hypotheses[0].evidence_ids,
            vec!["E1".to_owned(), "E2".to_owned()]
        );
    }
}
