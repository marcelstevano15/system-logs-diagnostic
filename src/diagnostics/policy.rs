#[derive(Clone, Debug)]
pub struct DiagnosticPolicy {
    pub base_critical_penalty: f64,
    pub max_critical_penalty: f64,
    pub max_error_penalty: f64,
    pub max_warning_penalty: f64,
    pub scarcity_threshold: usize,
    pub min_allowed_score: u8,
}

impl Default for DiagnosticPolicy {
    fn default() -> Self {
        Self {
            base_critical_penalty: 40.0,
            max_critical_penalty: 45.0,
            max_error_penalty: 45.0,
            max_warning_penalty: 15.0,
            scarcity_threshold: 50,
            min_allowed_score: 5,
        }
    }
}
