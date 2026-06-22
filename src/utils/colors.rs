use crate::diagnostics::result::HealthStatus;
use crate::models::log_entry::Severity;

pub fn severity_to_css_class(severity: &Severity) -> &'static str {
    severity.css_class()
}

pub fn priority_to_css_class(priority: u8) -> &'static str {
    Severity::from_priority(priority).css_class()
}

pub fn score_to_css_class(score: u8) -> &'static str {
    HealthStatus::from_score(score).css_class()
}
