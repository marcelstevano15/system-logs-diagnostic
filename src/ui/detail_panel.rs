use gtk::prelude::*;
use crate::models::log_entry::LogEntry;

pub struct DetailPanel {
    pub container: gtk::Box,
    title_label: gtk::Label,
    timestamp_value: gtk::Label,
    severity_value: gtk::Label,
    process_value: gtk::Label,
    pid_value: gtk::Label,
    unit_value: gtk::Label,
    hostname_value: gtk::Label,
    executable_value: gtk::Label,
    message_view: gtk::TextView,
}

impl DetailPanel {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_width_request(300);

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroll.set_vexpand(true);

        let inner = gtk::Box::new(gtk::Orientation::Vertical, 12);
        inner.set_margin_start(16);
        inner.set_margin_end(16);
        inner.set_margin_top(16);
        inner.set_margin_bottom(16);

        let title_label = gtk::Label::new(Some("Log Entry Details"));
        title_label.add_css_class("heading");
        title_label.set_xalign(0.0);
        inner.append(&title_label);

        inner.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        let timestamp_value = make_field_row(&inner, "Timestamp");
        let severity_value = make_field_row(&inner, "Severity");
        let process_value = make_field_row(&inner, "Process");
        let pid_value = make_field_row(&inner, "PID");
        let unit_value = make_field_row(&inner, "Systemd Unit");
        let hostname_value = make_field_row(&inner, "Hostname");
        let executable_value = make_field_row(&inner, "Executable");

        let msg_label = gtk::Label::new(Some("Message"));
        msg_label.add_css_class("log-detail-title");
        msg_label.set_xalign(0.0);
        inner.append(&msg_label);

        let message_view = gtk::TextView::new();
        message_view.set_editable(false);
        message_view.set_cursor_visible(false);
        message_view.set_wrap_mode(gtk::WrapMode::WordChar);
        message_view.set_monospace(true);
        message_view.add_css_class("card");
        message_view.set_left_margin(8);
        message_view.set_right_margin(8);
        message_view.set_top_margin(6);
        message_view.set_bottom_margin(6);
        inner.append(&message_view);

        scroll.set_child(Some(&inner));
        container.append(&scroll);

        Self {
            container,
            title_label,
            timestamp_value,
            severity_value,
            process_value,
            pid_value,
            unit_value,
            hostname_value,
            executable_value,
            message_view,
        }
    }

    pub fn update(&self, entry: &LogEntry) {
        self.title_label.set_text("Log Entry Details");
        self.timestamp_value.set_text(
    &entry.timestamp
        .with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
); 
        self.severity_value.set_text(&entry.severity.to_string().to_uppercase());
        self.process_value.set_text(&entry.process);
        self.pid_value.set_text(
            &entry.pid.map(|p| p.to_string()).unwrap_or_else(|| "N/A".to_string()),
        );
        self.unit_value.set_text(entry.systemd_unit.as_deref().unwrap_or("N/A"));
        self.hostname_value.set_text(entry.hostname.as_deref().unwrap_or("N/A"));
        self.executable_value.set_text(entry.executable.as_deref().unwrap_or("N/A"));

        let buf = self.message_view.buffer();
        buf.set_text(&entry.message);
    }

    pub fn clear(&self) {
        self.title_label.set_text("Select a log entry");
        self.timestamp_value.set_text("");
        self.severity_value.set_text("");
        self.process_value.set_text("");
        self.pid_value.set_text("");
        self.unit_value.set_text("");
        self.hostname_value.set_text("");
        self.executable_value.set_text("");
        self.message_view.buffer().set_text("");
    }
}

fn make_field_row(parent: &gtk::Box, label_text: &str) -> gtk::Label {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 2);

    let label = gtk::Label::new(Some(label_text));
    label.add_css_class("log-detail-title");
    label.set_xalign(0.0);
    row.append(&label);

    let value = gtk::Label::new(None);
    value.add_css_class("log-detail-value");
    value.set_xalign(0.0);
    value.set_selectable(true);
    value.set_ellipsize(gtk::pango::EllipsizeMode::End);
    row.append(&value);

    parent.append(&row);
    value
}

