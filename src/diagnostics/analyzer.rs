use crate::models::log_entry::{LogEntry, Severity};
use super::policy::DiagnosticPolicy;
use super::result::DiagnosticResult;

pub fn analyze(
    logs: &[LogEntry], 
    policy: &DiagnosticPolicy, 
    expected_throughput_baseline: Option<f64>
) -> DiagnosticResult {
    let total_count = logs.len();
    
    if total_count == 0 {
        return DiagnosticResult {
            title: "No Data Available".to_string(),
            description: "System diagnostics inactive due to empty log stream.".to_string(),
            score: 100,
            critical_count: 0,
            error_count: 0,
            warning_count: 0,
        };
    }

    let mut critical_count = 0;
    let mut error_count = 0;
    let mut warning_count = 0;

    for log in logs {
        match log.severity {
            Severity::Critical => critical_count += 1,
            Severity::Error => error_count += 1,
            Severity::Warning => warning_count += 1,
            _ => {}
        }
    }

    let normalization_factor = expected_throughput_baseline.unwrap_or(total_count as f64);
    let n = if normalization_factor <= 0.0 { total_count as f64 } else { normalization_factor };
    let mut total_penalty = 0.0;

    if critical_count > 0 {
        let incremental_critical = (critical_count as f64 * 5.0).min(policy.max_critical_penalty);
        total_penalty += policy.base_critical_penalty + incremental_critical;
    }

    if error_count > 0 {
        let error_ratio = error_count as f64 / n;
        let error_penalty = (error_ratio * policy.max_error_penalty).min(policy.max_error_penalty);
        total_penalty += error_penalty;
    }

    if warning_count > 0 {
        let warning_ratio = warning_count as f64 / n;
        let warning_penalty = (warning_ratio * policy.max_warning_penalty).min(policy.max_warning_penalty);
        total_penalty += warning_penalty;
    }

    let mut score = 100.0 - total_penalty;
    
    if total_count < policy.scarcity_threshold && critical_count == 0 {
        let data_scarcity_factor = (total_count as f64 / policy.scarcity_threshold as f64).max(0.4);
        score = 100.0 - ((100.0 - score) * data_scarcity_factor);
    }

    let final_score = if score <= policy.min_allowed_score as f64 {
        policy.min_allowed_score
    } else if score >= 100.0 || score.is_nan() {
        100
    } else {
        score.round() as u8
    };

    let title = match final_score {
        0..=30 => "Critical Failures Detected",
        31..=70 => "Operational Errors Detected",
        71..=92 => "Warnings Recorded",
        _ => "System Integrity Verified",
    };

    let description = format!(
        "Critical: {} | Errors: {} | Warnings: {} | Total: {} | Health Score: {}%",
        critical_count, error_count, warning_count, total_count, final_score
    );

    DiagnosticResult {
        title: title.to_string(),
        description,
        score: final_score,
        critical_count,
        error_count,
        warning_count,
    }
}
