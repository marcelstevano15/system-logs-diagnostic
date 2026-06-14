use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
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
    pub raw: serde_json::Value,
}

impl LogEntry {
    pub fn to_tantivy_doc(&self, schema: &tantivy::schema::Schema) -> tantivy::TantivyDocument {
        use tantivy::TantivyDocument;
        let mut doc = TantivyDocument::new();
        
        let f_timestamp = schema.get_field("timestamp").unwrap();
        let f_priority = schema.get_field("priority").unwrap();
        let f_process = schema.get_field("process").unwrap();
        let f_message = schema.get_field("message").unwrap();
        let f_severity = schema.get_field("severity").unwrap();

        let tantivy_date = tantivy::DateTime::from_timestamp_secs(self.timestamp.timestamp());
        doc.add_date(f_timestamp, tantivy_date);
        doc.add_u64(f_priority, self.priority as u64);
        doc.add_text(f_process, &self.process);
        doc.add_text(f_message, &self.message);
        
        let sev_str = match self.severity {
            Severity::Critical => "critical",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Debug => "debug",
        };
        doc.add_text(f_severity, sev_str);

        doc
    }
}

