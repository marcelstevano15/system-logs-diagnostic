use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::Mutex;

#[derive(Clone)]
pub struct Debouncer {
    last_trigger: Arc<Mutex<Option<Instant>>>,
    delay: Duration,
}

impl Debouncer {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            last_trigger: Arc::new(Mutex::new(None)),
            delay: Duration::from_millis(delay_ms),
        }
    }

    pub fn trigger(&self) {
        let now = Instant::now();
        let mut last = self.last_trigger.lock();
        *last = Some(now);
    }

    pub fn is_settled(&self) -> bool {
        let guard = self.last_trigger.lock();
        if let Some(last) = *guard {
            last.elapsed() >= self.delay
        } else {
            false
        }
    }

    pub fn reset(&self) {
        let mut guard = self.last_trigger.lock();
        *guard = None;
    }
}
