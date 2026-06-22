use crate::models::log_entry::LogEntry;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SortKey {
    #[default]
    TimeNewest,
    TimeOldest,
    SeverityHighLow,
    SeverityLowHigh,
    ProcessAZ,
    ProcessZA,
    UnitAZ,
    UnitZA,
    HostnameAZ,
    HostnameZA,
    PidAscending,
    PidDescending,
}

impl SortKey {
    pub fn label(&self) -> &'static str {
        match self {
            SortKey::TimeNewest => "Time: Newest First",
            SortKey::TimeOldest => "Time: Oldest First",
            SortKey::SeverityHighLow => "Severity: High → Low",
            SortKey::SeverityLowHigh => "Severity: Low → High",
            SortKey::ProcessAZ => "Process: A → Z",
            SortKey::ProcessZA => "Process: Z → A",
            SortKey::UnitAZ => "Unit: A → Z",
            SortKey::UnitZA => "Unit: Z → A",
            SortKey::HostnameAZ => "Hostname: A → Z",
            SortKey::HostnameZA => "Hostname: Z → A",
            SortKey::PidAscending => "PID: Ascending",
            SortKey::PidDescending => "PID: Descending",
        }
    }

    pub fn action_name(&self) -> &'static str {
        match self {
            SortKey::TimeNewest => "win.sort_time_newest",
            SortKey::TimeOldest => "win.sort_time_oldest",
            SortKey::SeverityHighLow => "win.sort_sev_high_low",
            SortKey::SeverityLowHigh => "win.sort_sev_low_high",
            SortKey::ProcessAZ => "win.sort_process_az",
            SortKey::ProcessZA => "win.sort_process_za",
            SortKey::UnitAZ => "win.sort_unit_az",
            SortKey::UnitZA => "win.sort_unit_za",
            SortKey::HostnameAZ => "win.sort_hostname_az",
            SortKey::HostnameZA => "win.sort_hostname_za",
            SortKey::PidAscending => "win.sort_pid_asc",
            SortKey::PidDescending => "win.sort_pid_desc",
        }
    }

    pub fn all() -> &'static [SortKey] {
        &[
            SortKey::TimeNewest,
            SortKey::TimeOldest,
            SortKey::SeverityHighLow,
            SortKey::SeverityLowHigh,
            SortKey::ProcessAZ,
            SortKey::ProcessZA,
            SortKey::UnitAZ,
            SortKey::UnitZA,
            SortKey::HostnameAZ,
            SortKey::HostnameZA,
            SortKey::PidAscending,
            SortKey::PidDescending,
        ]
    }
}

pub fn apply_sort(logs: &mut Vec<LogEntry>, key: &SortKey) {
    match key {
        SortKey::TimeNewest => {
            logs.sort_unstable_by(|a, b| b.timestamp.cmp(&a.timestamp));
        }
        SortKey::TimeOldest => {
            logs.sort_unstable_by(|a, b| a.timestamp.cmp(&b.timestamp));
        }
        SortKey::SeverityHighLow => {
            logs.sort_unstable_by(|a, b| a.severity.cmp(&b.severity));
        }
        SortKey::SeverityLowHigh => {
            logs.sort_unstable_by(|a, b| b.severity.cmp(&a.severity));
        }
        SortKey::ProcessAZ => {
            logs.sort_unstable_by(|a, b| {
                a.process.to_ascii_lowercase().cmp(&b.process.to_ascii_lowercase())
            });
        }
        SortKey::ProcessZA => {
            logs.sort_unstable_by(|a, b| {
                b.process.to_ascii_lowercase().cmp(&a.process.to_ascii_lowercase())
            });
        }
        SortKey::UnitAZ => {
            logs.sort_unstable_by(|a, b| {
                let ak = a.systemd_unit.as_deref().unwrap_or("").to_ascii_lowercase();
                let bk = b.systemd_unit.as_deref().unwrap_or("").to_ascii_lowercase();
                ak.cmp(&bk)
            });
        }
        SortKey::UnitZA => {
            logs.sort_unstable_by(|a, b| {
                let ak = a.systemd_unit.as_deref().unwrap_or("").to_ascii_lowercase();
                let bk = b.systemd_unit.as_deref().unwrap_or("").to_ascii_lowercase();
                bk.cmp(&ak)
            });
        }
        SortKey::HostnameAZ => {
            logs.sort_unstable_by(|a, b| {
                let ak = a.hostname.as_deref().unwrap_or("").to_ascii_lowercase();
                let bk = b.hostname.as_deref().unwrap_or("").to_ascii_lowercase();
                ak.cmp(&bk)
            });
        }
        SortKey::HostnameZA => {
            logs.sort_unstable_by(|a, b| {
                let ak = a.hostname.as_deref().unwrap_or("").to_ascii_lowercase();
                let bk = b.hostname.as_deref().unwrap_or("").to_ascii_lowercase();
                bk.cmp(&ak)
            });
        }
        SortKey::PidAscending => {
            logs.sort_unstable_by(|a, b| a.pid.cmp(&b.pid));
        }
        SortKey::PidDescending => {
            logs.sort_unstable_by(|a, b| b.pid.cmp(&a.pid));
        }
    }
}
