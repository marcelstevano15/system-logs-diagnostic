use crate::models::log_entry::LogEntry;
use anyhow::Result;
use serde_json::Value;
use std::process::Command;

use super::parser::parse_entry;

pub fn fetch_boot_logs() -> Result<Vec<LogEntry>> {
    let output = Command::new("journalctl")
        .args(["-b", "0", "-n", "1000", "-o", "json"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut entries = Vec::new();
    for line in stdout.lines() {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if let Some(entry) = parse_entry(v) {
                entries.push(entry);
            }
        }
    }
    Ok(entries)
}
