use crate::errors::{AppError, AppResult};
use crate::models::log_entry::LogEntry;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tar::Builder;

pub fn export_archive(output_path: &Path, logs: &[LogEntry]) -> AppResult<()> {
    let file = File::create(output_path)
        .map_err(|e| AppError::Archive(format!("Failed to create archive: {}", e)))?;

    let encoder = GzEncoder::new(file, Compression::best());
    let mut tar = Builder::new(encoder);

    let json = serde_json::to_vec_pretty(logs)
        .map_err(|e| AppError::Archive(format!("JSON serialization failed: {}", e)))?;

    let mut header = tar::Header::new_gnu();
    header.set_size(json.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();

    tar.append_data(&mut header, "logs.json", json.as_slice())
        .map_err(|e| AppError::Archive(format!("Failed to append JSON: {}", e)))?;

    let mut csv_buf = Vec::with_capacity(logs.len() * 200);
    writeln!(csv_buf, "timestamp,priority,severity,process,pid,systemd_unit,hostname,message")
        .map_err(|e| AppError::Archive(e.to_string()))?;

    for log in logs {
        let timestamp = log.timestamp.to_rfc3339();
        let pid = log.pid.map(|p| p.to_string()).unwrap_or_default();
        let unit = log.systemd_unit.as_deref().unwrap_or("");
        let hostname = log.hostname.as_deref().unwrap_or("");
        let message = log.message.replace('"', "\"\"");

        let line = format!(
            "{},{},{},{},{},{},{},\"{}\"\n",
            timestamp,
            log.priority,
            log.severity,
            csv_escape(&log.process),
            pid,
            csv_escape(unit),
            csv_escape(hostname),
            message,
        );
        csv_buf.extend_from_slice(line.as_bytes());
    }

    let mut csv_header = tar::Header::new_gnu();
    csv_header.set_size(csv_buf.len() as u64);
    csv_header.set_mode(0o644);
    csv_header.set_cksum();

    tar.append_data(&mut csv_header, "logs.csv", csv_buf.as_slice())
        .map_err(|e| AppError::Archive(format!("Failed to append CSV: {}", e)))?;

    let gz = tar
        .into_inner()
        .map_err(|e| AppError::Archive(format!("Failed to finalize tar: {}", e)))?;

    gz.finish()
        .map_err(|e| AppError::Archive(format!("Failed to finalize gzip: {}", e)))?;

    Ok(())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
