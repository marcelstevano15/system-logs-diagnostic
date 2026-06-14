use crate::core::search::LogSearchEngine;
use crate::models::filter::FilterState;
use crate::models::log_entry::LogEntry;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone)]
pub struct LogCache {
    inner: Arc<Mutex<Vec<LogEntry>>>,
}

impl LogCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn push(&self, entry: LogEntry) {
        self.inner.lock().push(entry);
    }

    pub fn all(&self) -> Vec<LogEntry> {
        self.inner.lock().clone()
    }

    pub fn replace(&self, entries: Vec<LogEntry>) {
        let mut lock = self.inner.lock();
        *lock = entries;
    }
}

#[derive(Clone)]
pub struct AppState {
    pub search_engine: Arc<LogSearchEngine>,
    pub cache: LogCache,
    pub filters: Arc<Mutex<FilterState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            search_engine: Arc::new(LogSearchEngine::new().unwrap()),
            cache: LogCache::new(),
            filters: Arc::new(Mutex::new(FilterState::default())),
        }
    }
}

