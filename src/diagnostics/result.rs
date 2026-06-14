#[derive(Clone, Debug)]
pub struct DiagnosticResult {
    pub title: String,
    pub description: String,
    pub score: u8,
    pub critical_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
}
