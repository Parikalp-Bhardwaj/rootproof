use rootproof_core::{
    EvidenceBundle,
    SourceAnalysis,
    SourceFinding,
};

use serde::{
    Deserialize,
    Serialize,
};

use schemars::JsonSchema;

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    JsonSchema,
)]
pub struct SourceAnalysisOutput {
    pub findings:
        Vec<SourceFindingOutput>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    JsonSchema,
)]
pub struct SourceFindingOutput {
    pub finding_type: String,
    pub expression: Option<String>,
    pub reason: String,
}

use rootproof_ai::OpenRouterProvider;

pub async fn analyze_source(provider: &OpenRouterProvider, evidence: &EvidenceBundle,) -> Result<SourceAnalysis,Box<dyn std::error::Error>> {
    let mut input = String::new();

    for item in &evidence.evidence { input.push_str(
            &format!("Evidence {}:\n{}\n\n", item.id, item.content));
    }

    let preamble = r#"
            You are RootProof's Source Analysis Agent.

            Analyze only the evidence provided.

            Do not invent files, functions, values, or runtime behavior.

            Your job is to identify concrete source-code observations
            that may explain the observed failure.

            Do not generate a fix.
            Do not claim a root cause is proven.
            Do not create hypotheses yet.

            Return only structured findings.
            "#;

    let output:
        SourceAnalysisOutput = provider.extract(&input, preamble).await?;

    let findings = output.findings.into_iter().map(|finding| {
                SourceFinding {
                    finding_type:
                        finding.finding_type,
                    expression:
                        finding.expression,
                    reason:
                        finding.reason,
                }
            })
            .collect();

    let file = evidence.evidence.iter().find_map(
                |item| item.file.clone()
            )
            .unwrap_or_default();

    Ok(SourceAnalysis {file, findings})
}