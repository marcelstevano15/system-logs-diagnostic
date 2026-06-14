use crate::models::filter::FilterState;
use crate::models::log_entry::{LogEntry, Severity};
use crate::ui::navigation::SidebarSection;

pub fn apply_filter(logs: &[LogEntry], state: &FilterState) -> Vec<LogEntry> {
    let query = state.query.to_lowercase();
    logs.iter()
        .filter(|entry| {
            let mut matched = true;

            match state.section {
                SidebarSection::LiveLogs => {
                    if let Some(ref transport) = entry.transport {
                        matched &= transport != "kernel";
                    }
                    if let Some(ref unit) = entry.systemd_unit {
                        matched &= !unit.starts_with("systemd-journald");
                    }
                }
                SidebarSection::BootLogs => {
                    matched &= entry.process.eq_ignore_ascii_case("kernel")
                        || entry.systemd_unit.is_some()
                        || entry.transport.as_deref().is_some_and(|t| t == "kernel");
                }
                SidebarSection::Kernel => {
                    matched &= entry.process.eq_ignore_ascii_case("kernel")
                        || entry
                            .transport
                            .as_deref()
                            .is_some_and(|v| v.eq_ignore_ascii_case("kernel"));
                }
                SidebarSection::Security => {
                    let process = entry.process.to_lowercase();
                    let message = entry.message.to_lowercase();
                    matched &= process.contains("sudo")
                        || process.contains("sshd")
                        || process.contains("polkit")
                        || process.contains("audit")
                        || message.contains("authentication")
                        || message.contains("permission denied");
                }
                SidebarSection::Services => {
                    matched &= entry.systemd_unit.is_some();
                }
                SidebarSection::Storage => {
                    let text = format!(
                        "{} {}",
                        entry.process.to_lowercase(),
                        entry.message.to_lowercase()
                    );
                    matched &= text.contains("disk")
                        || text.contains("mount")
                        || text.contains("btrfs")
                        || text.contains("ext4")
                        || text.contains("xfs")
                        || text.contains("lvm")
                        || text.contains("udisks");
                }
                SidebarSection::Networking => {
                    let text = format!(
                        "{} {}",
                        entry.process.to_lowercase(),
                        entry.message.to_lowercase()
                    );
                    matched &= text.contains("network")
                        || text.contains("dhcp")
                        || text.contains("dns")
                        || text.contains("wpa")
                        || text.contains("networkmanager")
                        || text.contains("systemd-networkd");
                }
            }

            if !query.is_empty() {
                let process = entry.process.to_lowercase();
                let message = entry.message.to_lowercase();
                let systemd_unit = entry
                    .systemd_unit
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase();
                let hostname = entry
                    .hostname
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase();
                let executable = entry
                    .executable
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase();
                matched &= process.contains(&query)
                    || message.contains(&query)
                    || systemd_unit.contains(&query)
                    || hostname.contains(&query)
                    || executable.contains(&query);
            }

            if let Some(ref severity) = state.severity {
                matched &= match severity.as_str() {
                    "critical" => matches!(entry.severity, Severity::Critical),
                    "error" => matches!(entry.severity, Severity::Error),
                    "warning" => matches!(entry.severity, Severity::Warning),
                    "info" => matches!(entry.severity, Severity::Info),
                    "debug" => matches!(entry.severity, Severity::Debug),
                    _ => true,
                };
            }

            matched
        })
        .cloned()
        .collect()
}

