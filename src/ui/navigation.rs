use strum_macros::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display, EnumIter)]
pub enum SidebarSection {
    #[default]
    #[strum(serialize = "All Logs")]
    AllLogs,
    #[strum(serialize = "Live Logs")]
    LiveLogs,
    #[strum(serialize = "Boot Logs")]
    BootLogs,
    #[strum(serialize = "Kernel")]
    Kernel,
    #[strum(serialize = "Security")]
    Security,
    #[strum(serialize = "Services")]
    Services,
    #[strum(serialize = "Storage")]
    Storage,
    #[strum(serialize = "Networking")]
    Networking,
    #[strum(serialize = "Critical + Errors")]
    Critical,
    #[strum(serialize = "Power Audit")]
    PowerAudit,
}

impl SidebarSection {
    pub fn icon_name(&self) -> &'static str {
        match self {
            SidebarSection::AllLogs => "view-list-symbolic",
            SidebarSection::LiveLogs => "media-record-symbolic",
            SidebarSection::BootLogs => "system-reboot-symbolic",
            SidebarSection::Kernel => "computer-symbolic",
            SidebarSection::Security => "security-high-symbolic",
            SidebarSection::Services => "preferences-system-symbolic",
            SidebarSection::Storage => "drive-harddisk-symbolic",
            SidebarSection::Networking => "network-wireless-symbolic",
            SidebarSection::Critical => "dialog-error-symbolic",
            SidebarSection::PowerAudit => "system-shutdown-symbolic",
        }
    }

    pub fn all() -> &'static [SidebarSection] {
        &[
            SidebarSection::AllLogs,
            SidebarSection::LiveLogs,
            SidebarSection::BootLogs,
            SidebarSection::Kernel,
            SidebarSection::Security,
            SidebarSection::Services,
            SidebarSection::Storage,
            SidebarSection::Networking,
            SidebarSection::Critical,
            SidebarSection::PowerAudit,
        ]
    }

    pub fn from_index(index: i32) -> Self {
        match index {
            0 => SidebarSection::AllLogs,
            1 => SidebarSection::LiveLogs,
            2 => SidebarSection::BootLogs,
            3 => SidebarSection::Kernel,
            4 => SidebarSection::Security,
            5 => SidebarSection::Services,
            6 => SidebarSection::Storage,
            7 => SidebarSection::Networking,
            8 => SidebarSection::Critical,
            9 => SidebarSection::PowerAudit,
            _ => SidebarSection::AllLogs,
        }
    }

    pub fn is_power_audit(&self) -> bool {
        matches!(self, SidebarSection::PowerAudit)
    }

    pub fn is_critical(&self) -> bool {
        matches!(self, SidebarSection::Critical)
    }
}

