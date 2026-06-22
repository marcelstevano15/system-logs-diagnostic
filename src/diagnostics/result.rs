use strum_macros::Display;

#[derive(Clone, Debug, Display)]
pub enum HealthStatus {
    #[strum(serialize = "Critical")]
    Critical,
    #[strum(serialize = "Degraded")]
    Degraded,
    #[strum(serialize = "Warning")]
    Warning,
    #[strum(serialize = "Healthy")]
    Healthy,
}

impl HealthStatus {
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=30 => HealthStatus::Critical,
            31..=60 => HealthStatus::Degraded,
            61..=85 => HealthStatus::Warning,
            _ => HealthStatus::Healthy,
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            HealthStatus::Critical => "health-critical",
            HealthStatus::Degraded => "health-degraded",
            HealthStatus::Warning => "health-warning",
            HealthStatus::Healthy => "health-ok",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            HealthStatus::Critical => "dialog-error-symbolic",
            HealthStatus::Degraded => "dialog-warning-symbolic",
            HealthStatus::Warning => "dialog-information-symbolic",
            HealthStatus::Healthy => "emblem-ok-symbolic",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DiagnosticResult {
    pub title: String,
    pub description: String,
    pub score: u8,
    pub critical_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub debug_count: usize,
    pub total_count: usize,
    pub status: HealthStatus,
}

impl DiagnosticResult {
    pub fn empty() -> Self {
        Self {
            title: "No Data Available".to_string(),
            description: "System diagnostics inactive due to empty log stream.".to_string(),
            score: 100,
            critical_count: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            debug_count: 0,
            total_count: 0,
            status: HealthStatus::Healthy,
        }
    }
}

pub fn aggregate_worst_case(results: &[DiagnosticResult]) -> DiagnosticResult {
    if results.is_empty() {
        return DiagnosticResult::empty();
    }

    let worst_index = results
        .iter()
        .enumerate()
        .min_by_key(|(_, r)| r.score)
        .map(|(i, _)| i)
        .unwrap_or(0);

    let worst = results[worst_index].clone();

    let critical_count: usize = results.iter().map(|r| r.critical_count).sum();
    let error_count: usize = results.iter().map(|r| r.error_count).sum();
    let warning_count: usize = results.iter().map(|r| r.warning_count).sum();
    let info_count: usize = results.iter().map(|r| r.info_count).sum();
    let debug_count: usize = results.iter().map(|r| r.debug_count).sum();
    let total_count: usize = results.iter().map(|r| r.total_count).sum();

    DiagnosticResult {
        title: worst.title,
        description: worst.description,
        score: worst.score,
        critical_count,
        error_count,
        warning_count,
        info_count,
        debug_count,
        total_count,
        status: worst.status,
    }
}
