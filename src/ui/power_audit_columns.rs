use crate::diagnostics::power_audit::{PowerCycleEntry, PowerEventKind};
use gtk::prelude::*;

pub fn create_power_event_column() -> gtk::ColumnViewColumn {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();

        let badge_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
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
        let entry = get_power_entry(item);

        let icon = badge_box
            .first_child()
            .unwrap()
            .downcast::<gtk::Image>()
            .unwrap();
        let label = icon
            .next_sibling()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap();

        for cls in &["power-reboot", "power-shutdown", "power-unclean"] {
            label.remove_css_class(cls);
        }

        match entry.kind {
            PowerEventKind::Reboot => {
                icon.set_icon_name(Some("system-reboot-symbolic"));
                if entry.is_unclean {
                    label.set_text("REBOOT (UNCLEAN)");
                    label.add_css_class("power-unclean");
                } else {
                    label.set_text("REBOOT");
                    label.add_css_class("power-reboot");
                }
            }
            PowerEventKind::Shutdown => {
                icon.set_icon_name(Some("system-shutdown-symbolic"));
                label.set_text("SHUTDOWN");
                label.add_css_class("power-shutdown");
            }
        }
    });

    factory.connect_unbind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        if let Some(child) = item.child() {
            if let Ok(badge_box) = child.downcast::<gtk::Box>() {
                if let Some(icon_widget) = badge_box.first_child() {
                    if let Ok(img) = icon_widget.downcast::<gtk::Image>() {
                        img.clear();
                        if let Some(lbl_widget) = img.next_sibling() {
                            if let Ok(lbl) = lbl_widget.downcast::<gtk::Label>() {
                                lbl.set_text("");
                                for cls in &["power-reboot", "power-shutdown", "power-unclean"] {
                                    lbl.remove_css_class(cls);
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let col = gtk::ColumnViewColumn::new(Some("Event"), Some(factory));
    col.set_fixed_width(180);
    col.set_resizable(true);
    col
}

pub fn create_power_user_column() -> gtk::ColumnViewColumn {
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
        let entry = get_power_entry(item);
        label.set_text(&entry.user);
    });

    factory.connect_unbind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        if let Some(child) = item.child() {
            if let Ok(label) = child.downcast::<gtk::Label>() {
                label.set_text("");
            }
        }
    });

    let col = gtk::ColumnViewColumn::new(Some("Type"), Some(factory));
    col.set_resizable(true);
    col
}

pub fn create_power_terminal_column() -> gtk::ColumnViewColumn {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = gtk::Label::new(None);
        label.set_xalign(0.0);
        label.set_selectable(true);
        label.add_css_class("monospace");
        label.add_css_class("caption");
        item.set_child(Some(&label));
    });

    factory.connect_bind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = item.child().unwrap().downcast::<gtk::Label>().unwrap();
        let entry = get_power_entry(item);
        label.set_text(&entry.terminal);
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
    col.set_fixed_width(110);
    col.set_resizable(true);
    col
}

pub fn create_power_timestamp_column() -> gtk::ColumnViewColumn {
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
        let entry = get_power_entry(item);
        let ts = entry.timestamp.format("%Y-%m-%d %H:%M").to_string();
        label.set_text(&ts);
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

pub fn create_power_status_column() -> gtk::ColumnViewColumn {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();

        let status_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        status_box.set_valign(gtk::Align::Center);

        let icon = gtk::Image::new();
        icon.set_icon_size(gtk::IconSize::Normal);

        let label = gtk::Label::new(None);
        label.set_xalign(0.0);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);

        status_box.append(&icon);
        status_box.append(&label);
        item.set_child(Some(&status_box));
    });

    factory.connect_bind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let status_box = item.child().unwrap().downcast::<gtk::Box>().unwrap();
        let entry = get_power_entry(item);

        let icon = status_box
            .first_child()
            .unwrap()
            .downcast::<gtk::Image>()
            .unwrap();
        let label = icon
            .next_sibling()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap();

        for cls in &["power-unclean", "power-clean"] {
            label.remove_css_class(cls);
        }

        if entry.is_unclean {
            icon.set_icon_name(Some("dialog-error-symbolic"));
            label.set_text("Unclean — no preceding shutdown");
            label.add_css_class("power-unclean");
        } else {
            icon.set_icon_name(Some("emblem-ok-symbolic"));
            label.set_text("Clean");
            label.add_css_class("power-clean");
        }
    });

    factory.connect_unbind(|_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        if let Some(child) = item.child() {
            if let Ok(status_box) = child.downcast::<gtk::Box>() {
                if let Some(icon_widget) = status_box.first_child() {
                    if let Ok(img) = icon_widget.downcast::<gtk::Image>() {
                        img.clear();
                        if let Some(lbl_widget) = img.next_sibling() {
                            if let Ok(lbl) = lbl_widget.downcast::<gtk::Label>() {
                                lbl.set_text("");
                                for cls in &["power-unclean", "power-clean"] {
                                    lbl.remove_css_class(cls);
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let col = gtk::ColumnViewColumn::new(Some("Status"), Some(factory));
    col.set_expand(true);
    col.set_resizable(true);
    col
}

fn get_power_entry(item: &gtk::ListItem) -> PowerCycleEntry {
    let boxed = item.item().unwrap();
    let obj = boxed.downcast::<glib::BoxedAnyObject>().unwrap();
let entry = obj.borrow::<PowerCycleEntry>().clone();
entry
}
