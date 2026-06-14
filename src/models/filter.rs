use crate::ui::navigation::SidebarSection;

#[derive(Clone)]
pub struct FilterState {
    pub query: String,
    pub severity: Option<String>,
    pub process: Option<String>,
    pub unit: Option<String>,
    pub section: SidebarSection,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            query: String::new(),
            severity: None,
            process: None,
            unit: None,
            section: SidebarSection::LiveLogs,
        }
    }
}

