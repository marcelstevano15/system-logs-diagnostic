use crate::models::log_entry::{LogEntry, Severity};
use chrono::{DateTime, Utc};
use super::policy::DiagnosticPolicy;
use super::result::{DiagnosticResult, HealthStatus};

struct WindowCounts {
    critical: usize,
    error: usize,
    warning: usize,
    info: usize,
    debug: usize,
    total: usize,
}

fn decay_weight(age_seconds: f64, half_life_hours: f64) -> f64 {
    if half_life_hours <= 0.0 {
        return 1.0;
    }
    let half_life_seconds = half_life_hours * 3600.0;
    0.5_f64.powf(age_seconds / half_life_seconds)
}

fn score_window(
    logs: &[LogEntry],
    policy: &DiagnosticPolicy,
    now: DateTime<Utc>,
    window_seconds: i64,
) -> (u8, WindowCounts) {
    let mut satisfied_weight = 0.0f64;
    let mut tolerating_weight = 0.0f64;
    let mut critical_weight_sum = 0.0f64;
    let mut total_weight = 0.0f64;

    let mut counts = WindowCounts {
        critical: 0,
        error: 0,
        warning: 0,
        info: 0,
        debug: 0,
        total: 0,
    };

    for log in logs {
        let age_seconds = (now - log.timestamp).num_seconds();
        if age_seconds < 0 || age_seconds > window_seconds {
            continue;
        }

        let weight = decay_weight(age_seconds as f64, policy.half_life_hours);
        total_weight += weight;
        counts.total += 1;

        match log.severity {
            Severity::Critical => {
                critical_weight_sum += weight;
                counts.critical += 1;
            }
            Severity::Error => {
                counts.error += 1;
            }
            Severity::Warning => {
                tolerating_weight += weight;
                counts.warning += 1;
            }
            Severity::Info => {
                satisfied_weight += weight;
                counts.info += 1;
            }
            Severity::Debug => {
                satisfied_weight += weight;
                counts.debug += 1;
            }
        }
    }

    if total_weight <= 0.0 {
        return (100, counts);
    }

    let apdex = (satisfied_weight + tolerating_weight * 0.5) / total_weight;
    let critical_ratio = critical_weight_sum / total_weight;

    let critical_penalty = policy.max_critical_penalty
        * (1.0 - (-policy.critical_decay_rate * policy.critical_weight * critical_ratio).exp());

    let mut score = (apdex * 100.0 - critical_penalty).clamp(0.0, 100.0);

    if total_weight < policy.scarcity_threshold {
        let confidence = (total_weight / policy.scarcity_threshold).sqrt().clamp(0.4, 1.0);
        score = 100.0 - ((100.0 - score) * confidence);
    }

    let final_score = if score <= policy.min_allowed_score as f64 {
        policy.min_allowed_score
    } else if score >= 100.0 || score.is_nan() || score.is_infinite() {
        100
    } else {
        score.round() as u8
    };

    (final_score, counts)
}

pub fn analyze(logs: &[LogEntry], policy: &DiagnosticPolicy, now: DateTime<Utc>) -> DiagnosticResult {
    if logs.is_empty() {
        return DiagnosticResult::empty();
    }

    let (short_score, _) = score_window(logs, policy, now, policy.short_window_seconds);
    let (long_score, long_counts) = score_window(logs, policy, now, policy.long_window_seconds);

    let final_score = short_score.min(long_score);

let status = if long_counts.critical > 0 {
    HealthStatus::Critical
} else if long_counts.error > 0 {
    HealthStatus::Degraded
} else if long_counts.warning > 0 {
    HealthStatus::Warning
} else {
    HealthStatus::Healthy
};

let title = match &status {
    HealthStatus::Critical => "Critical Failures Detected",
    HealthStatus::Degraded => "Operational Errors Detected",
    HealthStatus::Warning => "Warnings Recorded",
    HealthStatus::Healthy => "System Integrity Verified",
};  

    let description = format!(
        "Critical: {} | Errors: {} | Warnings: {} | Info: {} | Total: {} | Health: {}% (short:{}% / long:{}%)",
        long_counts.critical,
        long_counts.error,
        long_counts.warning,
        long_counts.info,
        long_counts.total,
        final_score,
        short_score,
        long_score,
    );

    DiagnosticResult {
        title: title.to_string(),
        description,
        score: final_score,
        critical_count: long_counts.critical,
        error_count: long_counts.error,
        warning_count: long_counts.warning,
        info_count: long_counts.info,
        debug_count: long_counts.debug,
        total_count: long_counts.total,
        status,
    }
}
