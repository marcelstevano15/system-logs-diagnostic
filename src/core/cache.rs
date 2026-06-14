use crate::models::log_entry::LogEntry;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct LogCache {
    inner: Arc<RwLock<Vec<LogEntry>>>,
}

impl LogCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Vec::with_capacity(50000))),
        }
    }

    pub fn replace(&self, logs: Vec<LogEntry>) {
        let mut guard = self.inner.write();
        *guard = logs;
    }

    pub fn push(&self, log: LogEntry) {
        let mut guard = self.inner.write();
        if guard.len() >= 50000 {
            guard.remove(0);
        }
        guard.push(log);
    }

    pub fn all(&self) -> Vec<LogEntry> {
        self.inner.read().clone()
    }
}

