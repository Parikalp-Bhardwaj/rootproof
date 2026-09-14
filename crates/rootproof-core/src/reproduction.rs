#[derive(Debug, Clone, PartialEq)]
pub struct ReproductionCandidate {
    pub hypothesis_id: String,
    pub test_name: String,
    pub test_code: String,
    pub rationale: String,
    pub expected_failure: String,
}
