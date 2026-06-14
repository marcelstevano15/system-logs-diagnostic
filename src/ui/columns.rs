use gtk::prelude::*;
use crate::models::log_entry::{LogEntry, Severity};

pub fn create_text_column(title: &str, property: &str) -> gtk::ColumnViewColumn {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(move |_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = gtk::Label::new(None);
        label.set_xalign(0.0);
        label.set_selectable(true);
        item.set_child(Some(&label));
    });

    let property_name = property.to_string();

    factory.connect_bind(move |_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = item.child().unwrap().downcast::<gtk::Label>().unwrap();

        let boxed = item.item().unwrap();
        let obj = boxed.downcast::<glib::BoxedAnyObject>().unwrap();
        let log = obj.borrow::<LogEntry>();

        let text = match property_name.as_str() {
            "timestamp" => log.timestamp.to_rfc3339(),
            "process" => log.process.clone(),
            "message" => log.message.clone(),
            "priority" => match log.severity {
                Severity::Critical => "critical".to_string(),
                Severity::Error => "error".to_string(),
                Severity::Warning => "warning".to_string(),
                Severity::Info => "info".to_string(),
                Severity::Debug => "debug".to_string(),
            },
            _ => String::new(),
        };

        label.set_text(&text);
    });

    gtk::ColumnViewColumn::new(Some(title), Some(factory))
}

