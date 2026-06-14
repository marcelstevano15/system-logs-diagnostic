#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarSection {
    #[default]
    LiveLogs,
    BootLogs,
    Kernel,
    Security,
    Services,
    Storage,
    Networking,
}

