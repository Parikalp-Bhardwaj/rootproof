#[derive(Debug, Clone, PartialEq)]
pub struct Hypothesis {
    pub id: String,
    pub description: String,
    pub confidence: f32,
    pub evidence_ids: Vec<String>,
}
