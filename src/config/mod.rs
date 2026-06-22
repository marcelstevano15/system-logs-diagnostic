use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

const CONFIG_FILE_NAME: &str = "system-logs-diagnostic.json";
const DEFAULT_MAX_LOG_ENTRIES: usize = 100_000;
const DEFAULT_SEARCH_DEBOUNCE_MS: u64 = 300;
const DEFAULT_LIVE_BATCH_INTERVAL_MS: u64 = 500;
const DEFAULT_WINDOW_WIDTH: i32 = 1200;
const DEFAULT_WINDOW_HEIGHT: i32 = 800;
pub const MIN_WINDOW_WIDTH: i32 = 640;
pub const MIN_WINDOW_HEIGHT: i32 = 480;
const MAX_LOG_ENTRIES_LIMIT: usize = 1_000_000;
const MIN_LOG_ENTRIES_LIMIT: usize = 100;
const MAX_JOURNAL_BOOT_LIMIT: usize = 500_000;
const MIN_JOURNAL_BOOT_LIMIT: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub max_log_entries: usize,
    pub search_debounce_ms: u64,
    pub live_batch_interval_ms: u64,
    pub window_width: i32,
    pub window_height: i32,
    pub window_maximized: bool,
    pub sidebar_width: i32,
    pub show_debug_logs: bool,
    pub auto_scroll: bool,
    pub export_directory: Option<String>,
    pub color_scheme: ColorScheme,
    pub journal_boot_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ColorScheme {
    #[default]
    Default,
    Light,
    Dark,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_log_entries: DEFAULT_MAX_LOG_ENTRIES,
            search_debounce_ms: DEFAULT_SEARCH_DEBOUNCE_MS,
            live_batch_interval_ms: DEFAULT_LIVE_BATCH_INTERVAL_MS,
            window_width: DEFAULT_WINDOW_WIDTH,
            window_height: DEFAULT_WINDOW_HEIGHT,
            window_maximized: false,
            sidebar_width: 220,
            show_debug_logs: false,
            auto_scroll: true,
            export_directory: None,
            color_scheme: ColorScheme::Default,
            journal_boot_limit: 5000,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let base = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("system-logs-diagnostic").join(CONFIG_FILE_NAME)
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
                    Ok(cfg) => {
                        let validated = cfg.validated();
                        info!("Loaded config from {:?}", path);
                        return validated;
                    }
                    Err(e) => {
                        warn!("Failed to parse config: {:?}. Using defaults.", e);
                    }
                },
                Err(e) => {
                    warn!("Failed to read config: {:?}. Using defaults.", e);
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                warn!("Failed to create config directory: {:?}", e);
                return;
            }
        }

        let content = match serde_json::to_string_pretty(self) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to serialize config: {:?}", e);
                return;
            }
        };

        let tmp_path = path.with_extension("json.tmp");

        if let Err(e) = std::fs::write(&tmp_path, &content) {
            warn!("Failed to write config to temp file: {:?}", e);
            return;
        }

        if let Err(e) = std::fs::rename(&tmp_path, &path) {
            warn!("Failed to atomically replace config file: {:?}", e);
            let _ = std::fs::remove_file(&tmp_path);
            return;
        }

        info!("Config saved atomically to {:?}", path);
    }

    fn validated(mut self) -> Self {
        if self.max_log_entries < MIN_LOG_ENTRIES_LIMIT
            || self.max_log_entries > MAX_LOG_ENTRIES_LIMIT
        {
            warn!(
                "max_log_entries {} out of range [{}, {}], using default",
                self.max_log_entries, MIN_LOG_ENTRIES_LIMIT, MAX_LOG_ENTRIES_LIMIT
            );
            self.max_log_entries = DEFAULT_MAX_LOG_ENTRIES;
        }

        if self.journal_boot_limit < MIN_JOURNAL_BOOT_LIMIT
            || self.journal_boot_limit > MAX_JOURNAL_BOOT_LIMIT
        {
            warn!(
                "journal_boot_limit {} out of range [{}, {}], using default",
                self.journal_boot_limit, MIN_JOURNAL_BOOT_LIMIT, MAX_JOURNAL_BOOT_LIMIT
            );
            self.journal_boot_limit = 5000;
        }

        if self.window_width < MIN_WINDOW_WIDTH {
            warn!(
                "window_width {} below minimum {}, clamping",
                self.window_width, MIN_WINDOW_WIDTH
            );
            self.window_width = MIN_WINDOW_WIDTH;
        }

        if self.window_height < MIN_WINDOW_HEIGHT {
            warn!(
                "window_height {} below minimum {}, clamping",
                self.window_height, MIN_WINDOW_HEIGHT
            );
            self.window_height = MIN_WINDOW_HEIGHT;
        }

        if self.search_debounce_ms == 0 {
            warn!("search_debounce_ms is 0, using default");
            self.search_debounce_ms = DEFAULT_SEARCH_DEBOUNCE_MS;
        }

        if self.live_batch_interval_ms == 0 {
            warn!("live_batch_interval_ms is 0, using default");
            self.live_batch_interval_ms = DEFAULT_LIVE_BATCH_INTERVAL_MS;
        }

        if self.sidebar_width < 100 || self.sidebar_width > 600 {
            self.sidebar_width = 220;
        }

        self
    }
}
