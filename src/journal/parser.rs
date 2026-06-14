use crate::models::log_entry::{LogEntry, Severity};
use chrono::{TimeZone, Utc};
use serde_json::Value;

pub(super) fn parse_entry(v: Value) -> Option<LogEntry> {
    let message = v["MESSAGE"].as_str()?.to_string();
    let prio = v["PRIORITY"]
        .as_str()
        .unwrap_or("6")
        .parse::<u8>()
        .unwrap_or(6);
    let severity = match prio {
        0..=2 => Severity::Critical,
        3 => Severity::Error,
        4 => Severity::Warning,
        5..=6 => Severity::Info,
        _ => Severity::Debug,
    };
    let realtime = v["__REALTIME_TIMESTAMP"]
        .as_str()
        .unwrap_or("0")
        .parse::<i64>()
        .unwrap_or(0);
    Some(LogEntry {
        timestamp: Utc.timestamp_micros(realtime).unwrap(),
        priority: prio,
        process: v["_COMM"].as_str().unwrap_or("unknown").to_string(),
        pid: v["_PID"].as_str().and_then(|x| x.parse().ok()),
        systemd_unit: v["_SYSTEMD_UNIT"].as_str().map(str::to_string),
        transport: v["_TRANSPORT"].as_str().map(str::to_string),
        hostname: v["_HOSTNAME"].as_str().map(str::to_string),
        executable: v["_EXE"].as_str().map(str::to_string),
        message,
        severity,
        raw: v,
    })
}
