use crate::models::log_entry::LogEntry;
use anyhow::Result;
use std::fs;

pub fn export(path: &str, logs: &[LogEntry]) -> Result<()> {
    let json = serde_json::to_string_pretty(logs)?;
    fs::write(path, json)?;
    Ok(())
}

