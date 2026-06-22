use crate::models::filter::FilterState;
use crate::models::log_entry::{LogEntry, Severity};
use crate::ui::navigation::SidebarSection;
use memchr::memmem;
use rayon::prelude::*;
use regex::Regex;

pub fn apply_filter(logs: &[LogEntry], state: &FilterState) -> Vec<LogEntry> {
    let query_lower = state.query.to_lowercase();
    let query_bytes = query_lower.as_bytes().to_vec();

    let compiled_regex: Option<Regex> = if state.regex_mode && !state.query.is_empty() {
        Regex::new(&state.query).ok()
    } else {
        None
    };

    let finder = if !query_lower.is_empty() && compiled_regex.is_none() {
        Some(memmem::Finder::new(&query_bytes).into_owned())
    } else {
        None
    };

    let proc_filter_lower: Option<String> = state.process.as_ref().map(|s| s.to_lowercase());
    let unit_filter_lower: Option<String> = state.unit.as_ref().map(|s| s.to_lowercase());
    let host_filter_lower: Option<String> = state.hostname.as_ref().map(|s| s.to_lowercase());

    logs.par_iter()
        .filter(|entry| {
            if !matches_section(entry, state.section) {
                return false;
            }

            if let Some(ref sev_filter) = state.severity {
                if &entry.severity != sev_filter {
                    return false;
                }
            }

            if let Some(ref proc_f) = proc_filter_lower {
                if !entry.process_lower().contains(proc_f.as_str()) {
                    return false;
                }
            }

            if let Some(ref unit_f) = unit_filter_lower {
                let unit_match = entry
                    .systemd_unit
                    .as_deref()
                    .map(|u| u.to_lowercase().contains(unit_f.as_str()))
                    .unwrap_or(false);
                if !unit_match {
                    return false;
                }
            }

            if let Some(ref host_f) = host_filter_lower {
                let host_match = entry
                    .hostname
                    .as_deref()
                    .map(|h| h.to_lowercase().contains(host_f.as_str()))
                    .unwrap_or(false);
                if !host_match {
                    return false;
                }
            }

            if !query_lower.is_empty() {
                if let Some(ref re) = compiled_regex {
                    let mut buf = String::with_capacity(
                        entry.process.len()
                            + entry.message.len()
                            + entry.systemd_unit.as_deref().map_or(0, str::len)
                            + entry.hostname.as_deref().map_or(0, str::len)
                            + entry.executable.as_deref().map_or(0, str::len)
                            + 4,
                    );
                    buf.push_str(&entry.process);
                    buf.push(' ');
                    buf.push_str(&entry.message);
                    buf.push(' ');
                    buf.push_str(entry.systemd_unit.as_deref().unwrap_or(""));
                    buf.push(' ');
                    buf.push_str(entry.hostname.as_deref().unwrap_or(""));
                    buf.push(' ');
                    buf.push_str(entry.executable.as_deref().unwrap_or(""));
                    if !re.is_match(&buf) {
                        return false;
                    }
                } else if let Some(ref f) = finder {
                    if !entry.search_bytes_match(f) {
                        return false;
                    }
                }
            }

            true
        })
        .cloned()
        .collect()
}

fn matches_section(entry: &LogEntry, section: SidebarSection) -> bool {
    match section {
        SidebarSection::AllLogs => true,

        SidebarSection::PowerAudit => false,

        SidebarSection::LiveLogs => entry
            .transport
            .as_deref()
            .map(|t| t.eq_ignore_ascii_case("journal") || t.eq_ignore_ascii_case("syslog"))
            .unwrap_or(false),

        SidebarSection::BootLogs => {
            entry.process.eq_ignore_ascii_case("kernel")
                || entry
                    .transport
                    .as_deref()
                    .is_some_and(|t| t.eq_ignore_ascii_case("kernel"))
                || entry.systemd_unit.as_deref().is_some_and(|u| {
                    u.starts_with("systemd-") || u == "init.scope" || u == "system.slice"
                })
        }

        SidebarSection::Kernel => {
            entry.process.eq_ignore_ascii_case("kernel")
                || entry
                    .transport
                    .as_deref()
                    .is_some_and(|t| t.eq_ignore_ascii_case("kernel"))
        }

        SidebarSection::Security => {
            let p = entry.process.as_str();
            let m = entry.message.as_str();
            contains_any_ignore_ascii_case(p, &["sudo", "sshd", "polkit", "audit", "pam", "login", "passwd"])
                || contains_any_ignore_ascii_case(m, &["authentication", "permission denied", "unauthorized", "access denied"])
        }

        SidebarSection::Services => entry.systemd_unit.is_some(),

        SidebarSection::Storage => {
            let p = entry.process.as_str();
            let m = entry.message.as_str();
            let keywords = &["disk", "mount", "btrfs", "ext4", "xfs", "lvm", "udisks", "nvme", "sata", "scsi", "raid"];
            contains_any_ignore_ascii_case(p, keywords) || contains_any_ignore_ascii_case(m, keywords)
        }

        SidebarSection::Networking => {
            let p = entry.process.as_str();
            let m = entry.message.as_str();
            let keywords = &["network", "dhcp", "dns", "wpa", "networkmanager", "systemd-networkd", "ethernet", "wifi", "bluetooth", "firewall", "iptables", "nftables"];
            contains_any_ignore_ascii_case(p, keywords) || contains_any_ignore_ascii_case(m, keywords)
        }

        SidebarSection::Critical => {
            matches!(entry.severity, Severity::Critical | Severity::Error)
        }
    }
}

#[inline]
fn contains_any_ignore_ascii_case(haystack: &str, needles: &[&str]) -> bool {
    let lower = haystack.to_ascii_lowercase();
    needles.iter().any(|n| lower.contains(n))
}
