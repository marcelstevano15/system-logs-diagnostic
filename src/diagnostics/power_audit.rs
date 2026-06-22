use crate::errors::{AppError, AppResult};
use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Utc};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use super::policy::DiagnosticPolicy;
use super::result::{DiagnosticResult, HealthStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerEventKind {
    Reboot,
    Shutdown,
}

impl std::fmt::Display for PowerEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerEventKind::Reboot => write!(f, "reboot"),
            PowerEventKind::Shutdown => write!(f, "shutdown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PowerCycleEntry {
    pub kind: PowerEventKind,
    pub user: String,
    pub terminal: String,
    pub timestamp: DateTime<Utc>,
    pub raw_line: String,
    pub is_unclean: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PowerAuditResult {
    pub title: String,
    pub description: String,
    pub entries: Vec<PowerCycleEntry>,
    pub unclean_count: usize,
    pub total_count: usize,
}

pub fn fetch_power_cycles() -> AppResult<Vec<PowerCycleEntry>> {
    let mut child = Command::new("last")
        .arg("-x")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| AppError::Journal(format!("Failed to spawn last: {}", e)))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Journal("Failed to capture last stdout".to_string()))?;

    let reader = BufReader::with_capacity(16384, stdout);
    let mut lines: Vec<String> = Vec::new();

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.starts_with("reboot") || trimmed.starts_with("shutdown") {
                    lines.push(line);
                }
            }
            Err(e) => {
                tracing::warn!("Error reading last output: {:?}", e);
                break;
            }
        }
    }

    let _ = child.wait();

    let mut entries = parse_power_lines(&lines);
    mark_unclean_shutdowns(&mut entries, &lines);

    Ok(entries)
}

fn parse_power_lines(lines: &[String]) -> Vec<PowerCycleEntry> {
    lines
        .iter()
        .filter_map(|line| parse_single_line(line))
        .collect()
}

fn parse_single_line(line: &str) -> Option<PowerCycleEntry> {
    let mut parts = line.split_whitespace();

    let user = parts.next()?.to_string();
    let terminal = parts.next().unwrap_or("system").to_string();

    let kind = match user.as_str() {
        "reboot" => PowerEventKind::Reboot,
        "shutdown" => PowerEventKind::Shutdown,
        _ => return None,
    };

    let _event_word = parts.next();
    let _kernel_version = parts.next();

    let _weekday = parts.next().unwrap_or("");
    let month = parts.next().unwrap_or("").to_string();
    let day = parts.next().unwrap_or("").to_string();
    let time = parts.next().unwrap_or("00:00").to_string();

    let remaining: Vec<&str> = parts.collect();
    let year: String = remaining
        .iter()
        .find(|p| p.len() == 4 && p.chars().all(|c| c.is_ascii_digit()))
        .map(|s| s.to_string())
        .unwrap_or_else(|| Local::now().year().to_string());

    let timestamp = parse_timestamp(&month, &day, &time, &year);

    Some(PowerCycleEntry {
        kind,
        user: user.clone(),
        terminal,
        timestamp,
        raw_line: line.to_string(),
        is_unclean: false,
    })
}

fn parse_timestamp(month: &str, day: &str, time: &str, year: &str) -> DateTime<Utc> {
    let month_num = match month {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => 1,
    };

    let day_num: u32 = day.parse().unwrap_or(1);
    let year_num: i32 = year.parse().unwrap_or_else(|_| Local::now().year());

    let mut time_parts = time.splitn(2, ':');
    let hour: u32 = time_parts.next().unwrap_or("0").parse().unwrap_or(0);
    let minute: u32 = time_parts.next().unwrap_or("0").parse().unwrap_or(0);

    let naive = NaiveDateTime::new(
        chrono::NaiveDate::from_ymd_opt(year_num, month_num, day_num)
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        chrono::NaiveTime::from_hms_opt(hour, minute, 0)
            .unwrap_or_else(|| chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
    );

    Local
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc.from_utc_datetime(&naive))
}

fn mark_unclean_shutdowns(entries: &mut Vec<PowerCycleEntry>, raw_lines: &[String]) {
    for i in 1..raw_lines.len().saturating_sub(1) {
        let current = &raw_lines[i];
        if current.trim_start().starts_with("reboot") {
            let prev = raw_lines[i - 1].as_str();
            if !prev.trim_start().starts_with("shutdown") {
                if let Some(entry) = entries
                    .iter_mut()
                    .find(|e| e.kind == PowerEventKind::Reboot && e.raw_line == *current)
                {
                    entry.is_unclean = true;
                }
            }
        }
    }
}

pub fn analyze_power_cycles(entries: &[PowerCycleEntry]) -> PowerAuditResult {
    let total_count = entries.len();
    let unclean_count = entries.iter().filter(|e| e.is_unclean).count();

    let (title, description) = if total_count == 0 {
        (
            "No Power Cycle Data".to_string(),
            "No reboot or shutdown records found in system history.".to_string(),
        )
    } else if unclean_count > 0 {
        (
            format!("Unclean Shutdown Detected ({})", unclean_count),
            format!(
                "Power Audit detected {} unclean shutdown event(s) out of {} total power cycles.",
                unclean_count, total_count
            ),
        )
    } else {
        (
            "System Operational Integrity: Verified".to_string(),
            format!(
                "All {} historical power cycles conform to standard shutdown protocols.",
                total_count
            ),
        )
    };

    PowerAuditResult {
        title,
        description,
        entries: entries.to_vec(),
        unclean_count,
        total_count,
    }
}

pub fn power_audit_diagnostic(
    audit: &PowerAuditResult,
    policy: &DiagnosticPolicy,
    now: DateTime<Utc>,
) -> DiagnosticResult {
    if audit.entries.is_empty() {
        return DiagnosticResult::empty();
    }

    let mut satisfied_weight = 0.0f64;
    let mut unclean_weight = 0.0f64;
    let mut total_weight = 0.0f64;

    for entry in &audit.entries {
        let age_seconds = (now - entry.timestamp).num_seconds();
        if age_seconds < 0 || age_seconds > policy.power_audit_window_seconds {
            continue;
        }

        let weight = decay_weight(age_seconds as f64, policy.power_audit_half_life_hours);
        total_weight += weight;

        if entry.is_unclean {
            unclean_weight += weight;
        } else {
            satisfied_weight += weight;
        }
    }

    if total_weight <= 0.0 {
        return DiagnosticResult::empty();
    }

    let apdex = satisfied_weight / total_weight;
    let unclean_ratio = unclean_weight / total_weight;

    let critical_penalty = policy.max_critical_penalty
        * (1.0 - (-policy.critical_decay_rate * policy.critical_weight * unclean_ratio).exp());

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

    let status = HealthStatus::from_score(final_score);

    DiagnosticResult {
        title: audit.title.clone(),
        description: audit.description.clone(),
        score: final_score,
        critical_count: audit.unclean_count,
        error_count: 0,
        warning_count: 0,
        info_count: audit.total_count.saturating_sub(audit.unclean_count),
        debug_count: 0,
        total_count: audit.total_count,
        status,
    }
}

fn decay_weight(age_seconds: f64, half_life_hours: f64) -> f64 {
    if half_life_hours <= 0.0 {
        return 1.0;
    }
    let half_life_seconds = half_life_hours * 3600.0;
    0.5_f64.powf(age_seconds / half_life_seconds)
}
