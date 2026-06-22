use gtk::prelude::*;
use crate::models::log_entry::{LogEntry, Severity};
use std::sync::Arc;

pub fn create_timestamp_column() -> gtk::ColumnViewColumn {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = gtk::Label::new(None);
        label.set_xalign(0.0);
        label.set_selectable(true);
        label.add_css_class("monospace");
        item.set_child(Some(&label));
    });

    factory.connect_bind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = item.child().unwrap().downcast::<gtk::Label>().unwrap();
        with_log_entry(item, |log| {
            let ts = log.timestamp.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string();
            label.set_text(&ts);
        });
    });

    factory.connect_unbind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        if let Some(child) = item.child() {
            if let Ok(label) = child.downcast::<gtk::Label>() {
                label.set_text("");
            }
        }
    });

    let col = gtk::ColumnViewColumn::new(Some("Timestamp"), Some(factory));
    col.set_fixed_width(175);
    col.set_resizable(true);
    col
}

pub fn create_severity_column() -> gtk::ColumnViewColumn {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let badge_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        badge_box.set_valign(gtk::Align::Center);

        let icon = gtk::Image::new();
        icon.set_icon_size(gtk::IconSize::Normal);

        let label = gtk::Label::new(None);
        label.set_xalign(0.0);
        label.add_css_class("caption");

        badge_box.append(&icon);
        badge_box.append(&label);
        item.set_child(Some(&badge_box));
    });

    factory.connect_bind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let badge_box = item.child().unwrap().downcast::<gtk::Box>().unwrap();

        with_log_entry(item, |log| {
            let icon = badge_box.first_child().unwrap().downcast::<gtk::Image>().unwrap();
            let label = icon.next_sibling().unwrap().downcast::<gtk::Label>().unwrap();

            icon.set_icon_name(Some(log.severity.icon_name()));

            for css in &["severity-critical", "severity-error", "severity-warning", "severity-info", "severity-debug"] {
                label.remove_css_class(css);
            }

            let sev_text = log.severity.to_string().to_uppercase();
            label.set_text(&sev_text);
            label.add_css_class(log.severity.css_class());
        });
    });

    factory.connect_unbind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        if let Some(child) = item.child() {
            if let Ok(badge_box) = child.downcast::<gtk::Box>() {
                if let Some(icon) = badge_box.first_child() {
                    if let Ok(img) = icon.downcast::<gtk::Image>() {
                        img.clear();
                        if let Some(lbl_widget) = img.next_sibling() {
                            if let Ok(lbl) = lbl_widget.downcast::<gtk::Label>() {
                                lbl.set_text("");
                            }
                        }
                    }
                }
            }
        }
    });

    let col = gtk::ColumnViewColumn::new(Some("Severity"), Some(factory));
    col.set_fixed_width(110);
    col.set_resizable(true);
    col
}

pub fn create_process_column() -> gtk::ColumnViewColumn {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = gtk::Label::new(None);
        label.set_xalign(0.0);
        label.set_selectable(true);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        item.set_child(Some(&label));
    });

    factory.connect_bind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = item.child().unwrap().downcast::<gtk::Label>().unwrap();
        with_log_entry(item, |log| {
            if let Some(pid) = log.pid {
                let text = format!("{} [{}]", log.process, pid);
                label.set_text(&text);
            } else {
                label.set_text(&log.process);
            }
        });
    });

    factory.connect_unbind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        if let Some(child) = item.child() {
            if let Ok(label) = child.downcast::<gtk::Label>() {
                label.set_text("");
            }
        }
    });

    let col = gtk::ColumnViewColumn::new(Some("Process"), Some(factory));
    col.set_fixed_width(160);
    col.set_resizable(true);
    col
}

pub fn create_unit_column() -> gtk::ColumnViewColumn {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = gtk::Label::new(None);
        label.set_xalign(0.0);
        label.set_selectable(true);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.add_css_class("caption");
        item.set_child(Some(&label));
    });

    factory.connect_bind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = item.child().unwrap().downcast::<gtk::Label>().unwrap();
        with_log_entry(item, |log| {
            label.set_text(log.systemd_unit.as_deref().unwrap_or(""));
        });
    });

    factory.connect_unbind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        if let Some(child) = item.child() {
            if let Ok(label) = child.downcast::<gtk::Label>() {
                label.set_text("");
            }
        }
    });

    let col = gtk::ColumnViewColumn::new(Some("Unit"), Some(factory));
    col.set_fixed_width(180);
    col.set_resizable(true);
    col
}

pub fn create_message_column() -> gtk::ColumnViewColumn {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = gtk::Label::new(None);
        label.set_xalign(0.0);
        label.set_selectable(true);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.set_wrap(false);
        item.set_child(Some(&label));
    });

    factory.connect_bind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = item.child().unwrap().downcast::<gtk::Label>().unwrap();

        with_log_entry(item, |log| {
            for css in &["row-critical", "row-error", "row-warning"] {
                label.remove_css_class(css);
            }

            match log.severity {
                Severity::Critical => label.add_css_class("row-critical"),
                Severity::Error => label.add_css_class("row-error"),
                Severity::Warning => label.add_css_class("row-warning"),
                _ => {}
            }

            label.set_text(&log.message);
        });
    });

    factory.connect_unbind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        if let Some(child) = item.child() {
            if let Ok(label) = child.downcast::<gtk::Label>() {
                label.set_text("");
                for css in &["row-critical", "row-error", "row-warning"] {
                    label.remove_css_class(css);
                }
            }
        }
    });

    let col = gtk::ColumnViewColumn::new(Some("Message"), Some(factory));
    col.set_expand(true);
    col.set_resizable(true);
    col
}

fn with_log_entry<F: FnOnce(&LogEntry)>(item: &gtk::ListItem, f: F) {
    if let Some(boxed) = item.item() {
        if let Ok(obj) = boxed.downcast::<glib::BoxedAnyObject>() {
            let log = obj.borrow::<LogEntry>();
            f(&*log);
        }
    }
}
