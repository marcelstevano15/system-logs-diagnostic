use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use strum_macros::{Display, EnumIter, EnumString};
use memchr::memmem;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Display, EnumIter, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Severity {
    #[strum(serialize = "critical")]
    Critical,
    #[strum(serialize = "error")]
    Error,
    #[strum(serialize = "warning")]
    Warning,
    #[strum(serialize = "info")]
    Info,
    #[strum(serialize = "debug")]
    Debug,
}

impl Severity {
    pub fn from_priority(prio: u8) -> Self {
        match prio {
            0..=2 => Severity::Critical,
            3 => Severity::Error,
            4 => Severity::Warning,
            5..=6 => Severity::Info,
            _ => Severity::Debug,
        }
    }

    pub fn priority_value(&self) -> u8 {
        match self {
            Severity::Critical => 2,
            Severity::Error => 3,
            Severity::Warning => 4,
            Severity::Info => 6,
            Severity::Debug => 7,
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Severity::Critical => "severity-critical",
            Severity::Error => "severity-error",
            Severity::Warning => "severity-warning",
            Severity::Info => "severity-info",
            Severity::Debug => "severity-debug",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Severity::Critical => "dialog-error-symbolic",
            Severity::Error => "dialog-error-symbolic",
            Severity::Warning => "dialog-warning-symbolic",
            Severity::Info => "emblem-ok-symbolic",
            Severity::Debug => "system-run-symbolic",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub seq_id: u64,
    pub timestamp: DateTime<Utc>,
    pub priority: u8,
    pub process: String,
    pub pid: Option<u32>,
    pub systemd_unit: Option<String>,
    pub transport: Option<String>,
    pub hostname: Option<String>,
    pub executable: Option<String>,
    pub message: String,
    pub severity: Severity,
    pub tags: SmallVec<[String; 4]>,
    process_lc: String,
    message_lc: String,
}

impl LogEntry {
    pub fn new_with_seq(
        timestamp: DateTime<Utc>,
        priority: u8,
        process: String,
        pid: Option<u32>,
        systemd_unit: Option<String>,
        transport: Option<String>,
        hostname: Option<String>,
        executable: Option<String>,
        message: String,
        severity: Severity,
        tags: SmallVec<[String; 4]>,
    ) -> Self {
        let process_lc = process.to_ascii_lowercase();
        let message_lc = message.to_ascii_lowercase();
        Self {
            seq_id: SEQ_COUNTER.fetch_add(1, Ordering::Relaxed),
            timestamp,
            priority,
            process,
            pid,
            systemd_unit,
            transport,
            hostname,
            executable,
            message,
            severity,
            tags,
            process_lc,
            message_lc,
        }
    }

    #[inline]
    pub fn process_lower(&self) -> &str {
        &self.process_lc
    }

    #[inline]
    pub fn message_lower(&self) -> &str {
        &self.message_lc
    }

    pub fn search_bytes_match(&self, finder: &memmem::Finder) -> bool {
        if finder.find(self.process_lc.as_bytes()).is_some() {
            return true;
        }
        if finder.find(self.message_lc.as_bytes()).is_some() {
            return true;
        }
        if let Some(ref u) = self.systemd_unit {
            let u_lower = u.to_ascii_lowercase();
            if finder.find(u_lower.as_bytes()).is_some() {
                return true;
            }
        }
        if let Some(ref h) = self.hostname {
            let h_lower = h.to_ascii_lowercase();
            if finder.find(h_lower.as_bytes()).is_some() {
                return true;
            }
        }
        if let Some(ref e) = self.executable {
            let e_lower = e.to_ascii_lowercase();
            if finder.find(e_lower.as_bytes()).is_some() {
                return true;
            }
        }
        false
    }

    pub fn to_tantivy_doc_with_fields(
        &self,
        fields: &crate::core::search::SearchFields,
    ) -> tantivy::TantivyDocument {
        use tantivy::TantivyDocument;
        let mut doc = TantivyDocument::new();

        doc.add_u64(fields.seq_id, self.seq_id);
        let tantivy_date =
            tantivy::DateTime::from_timestamp_micros(self.timestamp.timestamp_micros());
        doc.add_date(fields.timestamp, tantivy_date);
        doc.add_u64(fields.priority, self.priority as u64);
        doc.add_text(fields.process, &self.process);
        doc.add_text(fields.message, &self.message);
        doc.add_text(fields.severity, self.severity.to_string().as_str());

        if let Some(ref unit) = self.systemd_unit {
            doc.add_text(fields.unit, unit);
        }
        if let Some(ref hostname) = self.hostname {
            doc.add_text(fields.hostname, hostname);
        }
        if let Some(ref exe) = self.executable {
            doc.add_text(fields.executable, exe);
        }

        doc
    }

    pub fn matches_text(&self, query: &str) -> bool {
        let q = query.to_ascii_lowercase();
        self.process_lc.contains(&q)
            || self.message_lc.contains(&q)
            || self.systemd_unit.as_deref().unwrap_or("").to_ascii_lowercase().contains(&q)
            || self.hostname.as_deref().unwrap_or("").to_ascii_lowercase().contains(&q)
            || self.executable.as_deref().unwrap_or("").to_ascii_lowercase().contains(&q)
    }
}
