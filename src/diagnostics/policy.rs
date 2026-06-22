#[derive(Clone, Debug)]
pub struct DiagnosticPolicy {
    pub short_window_seconds: i64,
    pub long_window_seconds: i64,
    pub half_life_hours: f64,
    pub critical_weight: f64,
    pub max_critical_penalty: f64,
    pub critical_decay_rate: f64,
    pub scarcity_threshold: f64,
    pub min_allowed_score: u8,
    pub power_audit_window_seconds: i64,
    pub power_audit_half_life_hours: f64,
}

impl Default for DiagnosticPolicy {
    fn default() -> Self {
        Self {
            short_window_seconds: 7_200,
            long_window_seconds: 86_400,
            half_life_hours: 6.0,
            critical_weight: 1.0,
            max_critical_penalty: 100.0,
            critical_decay_rate: 4.0,
            scarcity_threshold: 30.0,
            min_allowed_score: 5,
            power_audit_window_seconds: 2_592_000,
            power_audit_half_life_hours: 168.0,
        }
    }
}
