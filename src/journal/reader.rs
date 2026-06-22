use crate::errors::{AppError, AppResult};
use crate::models::log_entry::LogEntry;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use tracing::{debug, info, warn};

use super::parser::parse_entry;

static JOURNALCTL_PATH: OnceLock<Option<String>> = OnceLock::new();

fn journalctl_path() -> AppResult<&'static str> {
    let cached = JOURNALCTL_PATH.get_or_init(|| {
        for candidate in &["/usr/bin/journalctl", "/bin/journalctl", "journalctl"] {
            if std::fs::metadata(candidate).is_ok() {
                return Some(candidate.to_string());
            }
        }
        None
    });

    match cached {
        Some(p) => Ok(p.as_str()),
        None => Err(AppError::Journal(
            "journalctl not found. This application requires systemd.".to_string(),
        )),
    }
}

pub fn fetch_boot_logs(limit: usize) -> AppResult<Vec<LogEntry>> {
    info!("Fetching boot logs (limit={})", limit);

    let journalctl = journalctl_path()?;

    let mut child = Command::new(journalctl)
        .args(["-b", "0", "-n", &limit.to_string(), "-o", "json", "--no-pager"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| AppError::Journal(format!("Failed to spawn journalctl: {}", e)))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Journal("Failed to capture journalctl stdout".to_string()))?;

    let reader = BufReader::with_capacity(65536, stdout);
    let mut entries = Vec::with_capacity(limit.min(10_000));

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Value>(&line) {
                    Ok(v) => {
                        if let Some(entry) = parse_entry(v) {
                            entries.push(entry);
                        }
                    }
                    Err(e) => {
                        debug!("Failed to parse journal line: {:?}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Error reading journal line: {:?}", e);
                break;
            }
        }
    }

    let _ = child.wait();

    info!("Fetched {} log entries", entries.len());
    Ok(entries)
}

pub fn fetch_previous_boot_logs(boot_index: i32, limit: usize) -> AppResult<Vec<LogEntry>> {
    let journalctl = journalctl_path()?;
    let boot_arg = format!("{}", boot_index);

    let mut child = Command::new(journalctl)
        .args(["-b", &boot_arg, "-n", &limit.to_string(), "-o", "json", "--no-pager"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| AppError::Journal(format!("Failed to spawn journalctl: {}", e)))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Journal("Failed to capture journalctl stdout".to_string()))?;

    let reader = BufReader::with_capacity(65536, stdout);
    let mut entries = Vec::with_capacity(limit.min(10_000));

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<Value>(&line) {
                    if let Some(entry) = parse_entry(v) {
                        entries.push(entry);
                    }
                }
            }
            Err(e) => {
                warn!("Error reading journal line: {:?}", e);
                break;
            }
        }
    }

    let _ = child.wait();
    Ok(entries)
}

pub fn check_journalctl_available() -> bool {
    journalctl_path().is_ok()
}

pub fn get_boot_list() -> AppResult<Vec<(i32, String)>> {
    let journalctl = journalctl_path()?;

    let mut child = Command::new(journalctl)
        .args(["--list-boots", "--no-pager", "-o", "json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| AppError::Journal(format!("Failed to spawn journalctl: {}", e)))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Journal("Failed to capture journalctl stdout".to_string()))?;

    let reader = BufReader::with_capacity(16384, stdout);
    let mut boots: Vec<(i32, String)> = Vec::new();

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                if let Ok(v) = serde_json::from_str::<Value>(&line) {
                    if let (Some(idx), Some(ts)) = (
                        v["index"].as_i64(),
                        v["first_entry"].as_str().or_else(|| v["boot_id"].as_str()),
                    ) {
                        boots.push((idx as i32, ts.to_string()));
                    }
                }
            }
            Err(e) => {
                warn!("Error reading boot list line: {:?}", e);
                break;
            }
        }
    }

    let _ = child.wait();
    Ok(boots)
}
