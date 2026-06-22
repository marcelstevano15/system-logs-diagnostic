use crate::ui::navigation::SidebarSection;
use crate::models::log_entry::Severity;

#[derive(Clone, Debug, Default)]
pub struct FilterState {
    pub query: String,
    pub severity: Option<Severity>,
    pub process: Option<String>,
    pub unit: Option<String>,
    pub hostname: Option<String>,
    pub section: SidebarSection,
    pub regex_mode: bool,
}

impl FilterState {
    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
            && self.severity.is_none()
            && self.process.is_none()
            && self.unit.is_none()
            && self.hostname.is_none()
    }
}

