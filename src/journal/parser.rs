use crate::models::log_entry::{LogEntry, Severity};
use chrono::{TimeZone, Utc};
use serde_json::Value;
use smallvec::SmallVec;

pub(super) fn parse_entry(v: Value) -> Option<LogEntry> {
    let message = v["MESSAGE"].as_str()?.to_string();
    let prio = v["PRIORITY"]
        .as_str()
        .unwrap_or("6")
        .parse::<u8>()
        .unwrap_or(6);
    let severity = Severity::from_priority(prio);

    let realtime = v["__REALTIME_TIMESTAMP"]
        .as_str()
        .unwrap_or("0")
        .parse::<i64>()
        .unwrap_or(0);

    let timestamp = Utc
        .timestamp_micros(realtime)
        .single()
        .unwrap_or_else(Utc::now);

    let process = v["_COMM"]
        .as_str()
        .or_else(|| v["SYSLOG_IDENTIFIER"].as_str())
        .unwrap_or("unknown")
        .to_string();

    let mut tags: SmallVec<[String; 4]> = SmallVec::new();
    if v["_SYSTEMD_UNIT"].as_str().is_some() {
        tags.push("systemd".to_string());
    }
    if prio <= 3 {
        tags.push("alert".to_string());
    }

    Some(LogEntry::new_with_seq(
        timestamp,
        prio,
        process,
        v["_PID"].as_str().and_then(|x| x.parse().ok()),
        v["_SYSTEMD_UNIT"].as_str().map(str::to_string),
        v["_TRANSPORT"].as_str().map(str::to_string),
        v["_HOSTNAME"].as_str().map(str::to_string),
        v["_EXE"].as_str().map(str::to_string),
        message,
        severity,
        tags,
    ))
}
