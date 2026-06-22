use crate::models::log_entry::LogEntry;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;

const DEFAULT_CAPACITY: usize = 100_000;

#[derive(Clone)]
pub struct LogCache {
    inner: Arc<RwLock<VecDeque<LogEntry>>>,
    capacity: usize,
}

impl LogCache {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(VecDeque::with_capacity(
                capacity.min(DEFAULT_CAPACITY),
            ))),
            capacity,
        }
    }

    pub fn replace(&self, logs: Vec<LogEntry>) {
        let mut replacement = VecDeque::with_capacity(logs.len().min(self.capacity));
        replacement.extend(logs);
        let mut guard = self.inner.write();
        *guard = replacement;
    }

    pub fn push(&self, log: LogEntry) {
        let mut guard = self.inner.write();
        if guard.len() >= self.capacity {
            guard.pop_front();
        }
        guard.push_back(log);
    }

    pub fn push_batch(&self, logs: Vec<LogEntry>) {
        let mut guard = self.inner.write();
        for log in logs {
            if guard.len() >= self.capacity {
                guard.pop_front();
            }
            guard.push_back(log);
        }
    }

    pub fn all(&self) -> Vec<LogEntry> {
        self.inner.read().iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    pub fn clear(&self) {
        self.inner.write().clear();
    }

    pub fn snapshot_recent(&self, n: usize) -> Vec<LogEntry> {
        let guard = self.inner.read();
        let start = guard.len().saturating_sub(n);
        guard.range(start..).cloned().collect()
    }
}

impl Default for LogCache {
    fn default() -> Self {
        Self::new()
    }
}
