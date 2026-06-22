use crate::errors::{AppError, AppResult};
use crate::models::log_entry::LogEntry;
use std::fs;
use std::path::Path;

pub fn export(path: &Path, logs: &[LogEntry]) -> AppResult<()> {
    let json = serde_json::to_string_pretty(logs)
        .map_err(|e| AppError::Export(format!("JSON serialization failed: {}", e)))?;
    fs::write(path, json)
        .map_err(|e| AppError::Export(format!("Failed to write file: {}", e)))?;
    Ok(())
}

