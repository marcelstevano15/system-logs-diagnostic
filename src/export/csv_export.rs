use crate::errors::{AppError, AppResult};
use crate::models::log_entry::LogEntry;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;

pub fn export(path: &Path, logs: &[LogEntry]) -> AppResult<()> {
    let mut output = String::with_capacity(logs.len() * 200);

    writeln!(
        output,
        "timestamp,priority,severity,process,pid,systemd_unit,hostname,message"
    )
    .map_err(|e| AppError::Export(e.to_string()))?;

    for log in logs {
        let timestamp = log.timestamp.to_rfc3339();
        let pid = log.pid.map(|p| p.to_string()).unwrap_or_default();
        let unit = log.systemd_unit.as_deref().unwrap_or("");
        let hostname = log.hostname.as_deref().unwrap_or("");
        let message = log.message.replace('"', "\"\"");

        writeln!(
            output,
            "{},{},{},{},{},{},{},\"{}\"",
            timestamp,
            log.priority,
            log.severity,
            csv_escape(&log.process),
            pid,
            csv_escape(unit),
            csv_escape(hostname),
            message
        )
        .map_err(|e| AppError::Export(e.to_string()))?;
    }

    fs::write(path, output)
        .map_err(|e| AppError::Export(format!("Failed to write CSV: {}", e)))?;

    Ok(())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

