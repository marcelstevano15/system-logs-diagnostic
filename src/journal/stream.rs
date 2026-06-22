use crate::models::log_entry::LogEntry;
use async_channel::Sender;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use super::parser::parse_entry;

const BATCH_INTERVAL_MS: u64 = 500;
const BATCH_MAX_SIZE: usize = 100;

pub struct LiveStreamHandle {
    running: Arc<AtomicBool>,
}

impl LiveStreamHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }
}

impl Drop for LiveStreamHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn start_live_stream(sender: Sender<Vec<LogEntry>>) -> LiveStreamHandle {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    std::thread::Builder::new()
        .name("journal-live-stream".to_string())
        .spawn(move || {
            run_stream_loop(sender, running_clone);
        })
        .expect("Failed to spawn journal stream thread");

    LiveStreamHandle { running }
}

fn run_stream_loop(sender: Sender<Vec<LogEntry>>, running: Arc<AtomicBool>) {
    while running.load(Ordering::Acquire) {
        info!("Starting journalctl live stream");

        match spawn_journalctl() {
            Ok(mut child) => {
                let stdout = match child.stdout.take() {
                    Some(s) => s,
                    None => {
                        error!("Failed to capture journalctl stdout");
                        std::thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                };

                let reader = BufReader::with_capacity(65536, stdout);
                let mut batch: Vec<LogEntry> = Vec::with_capacity(BATCH_MAX_SIZE);
                let mut last_flush = Instant::now();

                for line_result in reader.lines() {
                    if !running.load(Ordering::Acquire) {
                        break;
                    }

                    match line_result {
                        Ok(line) => {
                            if let Ok(v) = serde_json::from_str::<Value>(&line) {
                                if let Some(entry) = parse_entry(v) {
                                    batch.push(entry);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Error reading journal stream line: {:?}", e);
                            break;
                        }
                    }

                    let should_flush = batch.len() >= BATCH_MAX_SIZE
                        || last_flush.elapsed() >= Duration::from_millis(BATCH_INTERVAL_MS);

                    if should_flush && !batch.is_empty() {
                        let to_send = std::mem::take(&mut batch);
                        if sender.send_blocking(to_send).is_err() {
                            info!("Live stream channel closed, stopping");
                            running.store(false, Ordering::Release);
                            break;
                        }
                        last_flush = Instant::now();
                    }
                }

                if !batch.is_empty() && running.load(Ordering::Acquire) {
                    let _ = sender.send_blocking(batch);
                }

                let _ = child.wait();
            }
            Err(e) => {
                error!("Failed to spawn journalctl: {:?}", e);
                std::thread::sleep(Duration::from_secs(5));
            }
        }

        if running.load(Ordering::Acquire) {
            warn!("journalctl stream ended unexpectedly, restarting in 2s");
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    info!("Journal live stream stopped");
}

fn spawn_journalctl() -> std::io::Result<Child> {
    Command::new("journalctl")
        .args(["-f", "-o", "json", "--no-pager"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
}
