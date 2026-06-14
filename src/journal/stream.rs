use crate::models::log_entry::LogEntry;
use async_channel::Sender;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use super::parser::parse_entry;

pub fn start_live_stream(sender: Sender<LogEntry>) {
    std::thread::spawn(move || {
        let mut child = Command::new("journalctl")
            .args(["-f", "-o", "json"])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        for line in reader.lines().flatten() {
            if let Ok(v) = serde_json::from_str::<Value>(&line) {
                if let Some(entry) = parse_entry(v) {
                    let _ = sender.send_blocking(entry);
                }
            }
        }
    });
}
