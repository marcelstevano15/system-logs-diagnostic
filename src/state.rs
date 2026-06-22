use crate::config::AppConfig;
use crate::core::cache::LogCache;
use crate::core::search::LogSearchEngine;
use crate::core::sort::SortKey;
use crate::diagnostics::DiagnosticResult;
use crate::errors::AppResult;
use crate::journal::LiveStreamHandle;
use crate::models::filter::FilterState;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub search_engine: Arc<LogSearchEngine>,
    pub cache: LogCache,
    pub filters: Arc<Mutex<FilterState>>,
    pub config: Arc<Mutex<AppConfig>>,
    pub is_loading: Arc<Mutex<bool>>,
    pub live_paused: Arc<Mutex<bool>>,
    pub current_sort: Arc<Mutex<SortKey>>,
    pub log_diagnostic_result: Arc<Mutex<DiagnosticResult>>,
    pub power_audit_result: Arc<Mutex<DiagnosticResult>>,
    stream_handle: Arc<Mutex<Option<LiveStreamHandle>>>,
}

impl AppState {
    pub fn new() -> AppResult<Self> {
        let config = AppConfig::load();
        let cache_cap = config.max_log_entries;

        Ok(Self {
            search_engine: Arc::new(LogSearchEngine::new()?),
            cache: LogCache::with_capacity(cache_cap),
            filters: Arc::new(Mutex::new(FilterState::default())),
            config: Arc::new(Mutex::new(config)),
            is_loading: Arc::new(Mutex::new(false)),
            live_paused: Arc::new(Mutex::new(false)),
            current_sort: Arc::new(Mutex::new(SortKey::default())),
            log_diagnostic_result: Arc::new(Mutex::new(DiagnosticResult::empty())),
            power_audit_result: Arc::new(Mutex::new(DiagnosticResult::empty())),
            stream_handle: Arc::new(Mutex::new(None)),
        })
    }

    pub fn set_loading(&self, loading: bool) {
        *self.is_loading.lock() = loading;
    }

    pub fn is_loading(&self) -> bool {
        *self.is_loading.lock()
    }

    pub fn toggle_live_pause(&self) -> bool {
        let mut paused = self.live_paused.lock();
        *paused = !*paused;
        *paused
    }

    pub fn is_live_paused(&self) -> bool {
        *self.live_paused.lock()
    }

    pub fn set_sort(&self, key: SortKey) {
        *self.current_sort.lock() = key;
    }

    pub fn get_sort(&self) -> SortKey {
        self.current_sort.lock().clone()
    }

    pub fn set_log_diagnostic_result(&self, result: DiagnosticResult) {
        *self.log_diagnostic_result.lock() = result;
    }

    pub fn get_log_diagnostic_result(&self) -> DiagnosticResult {
        self.log_diagnostic_result.lock().clone()
    }

    pub fn set_power_audit_result(&self, result: DiagnosticResult) {
        *self.power_audit_result.lock() = result;
    }

    pub fn get_power_audit_result(&self) -> DiagnosticResult {
        self.power_audit_result.lock().clone()
    }

    pub fn store_stream_handle(&self, handle: LiveStreamHandle) {
        *self.stream_handle.lock() = Some(handle);
    }
}
