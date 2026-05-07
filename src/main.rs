use gtk::prelude::*;
use adw::prelude::*;
use std::process::Command;
use gtk::{glib, gdk};
use adw::{Application, ApplicationWindow, HeaderBar, ToolbarView, StatusPage};
use std::rc::Rc;
use regex::Regex;
use std::path::Path;

const APP_ID: &str = "io.github.marcel.system-logs-diagnostic";

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn run_bash(cmd: &str) -> String {
    let output = Command::new("sh").arg("-c").arg(cmd).output();
    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => String::new(),
    }
}

fn diagnose_system(logs: &str, is_power_tab: bool) -> (String, String, &'static str, &'static str) {
    if is_power_tab { 
        return ("POWER AUDIT".to_string(), "Analysing power state history...".to_string(), "success", "system-shutdown-symbolic"); 
    }

    let is_shutdown_context = logs.contains("Reached target Shutdown") || logs.contains("Stopped User Manager");
    let re_critical = Regex::new(r"(?i)(kernel panic|hard resetting|out of memory|hardware error|machine check)").unwrap();
    
    let error_count = logs.to_lowercase().matches("error").count();
    let crash_count = logs.to_lowercase().split("segfault").count() - 1;
    let has_crit = re_critical.is_match(logs);

    if has_crit {
        ("CRITICAL FAILURE".to_string(), "Severe hardware or kernel issues detected!".to_string(), "destructive", "dialog-error-symbolic")
    } else if crash_count > 0 && !is_shutdown_context {
        let title = if crash_count == 1 { 
            "1 PROCESS CRASHED".to_string() 
        } else { 
            format!("{} PROCESSES CRASHED", crash_count) 
        };
        (title, format!("Detected {} unexpected application termination(s).", crash_count), "warning", "application-x-executable-symbolic")
    } else if error_count > 8 {
        ("SYSTEM ALERT".to_string(), format!("High volume of errors ({}) detected.", error_count), "warning", "dialog-warning-symbolic")
    } else if error_count > 0 {
        ("MINOR ISSUES".to_string(), format!("Found {} minor error logs.", error_count), "warning", "info-symbolic")
    } else {
        ("SYSTEM HEALTHY".to_string(), "No issues found in the current logs.".to_string(), "success", "emblem-ok-symbolic")
    }
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("System Diagnostic")
        .default_width(1100)
        .default_height(800)
        .build();

    let about_action = gio::SimpleAction::new("about", None);
    let window_weak = window.downgrade();
    
    about_action.connect_activate(move |_, _| {
        if let Some(window) = window_weak.upgrade() {
            let about = adw::AboutWindow::builder()
                .application_name("System Diagnostic")
                .version("0.1.0-beta")
                .developer_name("Marcel Stevano")
                .license_type(gtk::License::Gpl30)
                .website("https://github.com/marcelstevano15/linux-native-desktop-apps")
                .issue_url("https://github.com/marcelstevano15/linux-native-desktop-apps/issues")
                .transient_for(&window)
                .build();

            if Path::new("icon.png").exists() {
                about.set_application_icon("help-about-symbolic"); 
            } else {
                about.set_application_icon("help-about-symbolic");
            }

            about.present();
        }
    });
    app.add_action(&about_action);

    let split_view = adw::NavigationSplitView::new();

    let sidebar_toolbar = ToolbarView::new();
    let sidebar_header = HeaderBar::new();
    sidebar_header.set_show_end_title_buttons(false); 
    sidebar_toolbar.add_top_bar(&sidebar_header);

    let sidebar_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true)
        .build();
    sidebar_box.add_css_class("sidebar");
    
    let list_box = gtk::ListBox::new();
    list_box.add_css_class("navigation-sidebar");
    list_box.set_margin_start(12);
    list_box.set_margin_end(12);
    list_box.set_margin_top(12);
    list_box.set_margin_bottom(12);

    let create_item = |name: &str| {
        let row = adw::ActionRow::builder().title(name).activatable(true).build();
        list_box.append(&row);
    };

    let menu_items = ["Active Session", "Last Shutdown", "Power Audit", "Critical Errors", "Kernel Logs"];
    for item in menu_items { create_item(item); }
    
    sidebar_box.append(&list_box);
    let spacer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    sidebar_box.append(&spacer);

    sidebar_toolbar.set_content(Some(&sidebar_box));
    let sidebar_nav = adw::NavigationPage::new(&sidebar_toolbar, "Menu");

    let content_toolbar = ToolbarView::new();
    let content_header = HeaderBar::new();
    let menu_button = gtk::MenuButton::builder().icon_name("open-menu-symbolic").build();
    let menu_model = gio::Menu::new();
    menu_model.append(Some("About Application"), Some("app.about"));
    menu_button.set_menu_model(Some(&menu_model));
    content_header.pack_end(&menu_button);
    content_toolbar.add_top_bar(&content_header);

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let status_page = StatusPage::builder()
        .title("System Diagnostic")
        .description("Select a category from the sidebar to begin analysis.")
        .icon_name("security-high-symbolic")
        .build();

    let scrolled = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .margin_start(24).margin_end(24).margin_top(24).margin_bottom(24)
        .build();

    let log_view = gtk::TextView::builder()
        .editable(false)
        .monospace(true)
        .wrap_mode(gtk::WrapMode::WordChar)
        .left_margin(12).right_margin(12).top_margin(12).bottom_margin(12)
        .build();
    log_view.add_css_class("card");
    
    scrolled.set_child(Some(&log_view));
    content_box.append(&status_page);
    content_box.append(&scrolled);

    content_toolbar.set_content(Some(&content_box));
    let content_nav = adw::NavigationPage::new(&content_toolbar, "Content");

    split_view.set_sidebar(Some(&sidebar_nav));
    split_view.set_content(Some(&content_nav));
    window.set_content(Some(&split_view));

    let status_rc = Rc::new(status_page);
    let buffer_rc = Rc::new(log_view.buffer());

    let update_view = move |idx: i32| {
        let buf = &*buffer_rc;
        buf.set_text("");
        let is_power = idx == 2;
        let cmd = match idx {
            0 => "journalctl -b -n 400",
            1 => "journalctl -b -1 -n 400",
            2 => "last -x | grep -E 'shutdown|reboot' | head -n 30",
            3 => "journalctl -b -p 3 -n 400",
            4 => "journalctl -k -b -n 400",
            _ => "journalctl -n 100",
        };
        
        let logs = run_bash(cmd);
        let (title, desc, _style, icon) = diagnose_system(&logs, is_power);
        
        status_rc.set_title(&title);
        status_rc.set_description(Some(&desc));
        status_rc.set_icon_name(Some(icon));

        for line in logs.lines() {
            let mut iter = buf.end_iter();
            let lower = line.to_lowercase();
            if lower.contains("err") || lower.contains("panic") || lower.contains("crit") || lower.contains("segfault") {
                let tag = buf.tag_table().lookup("err").unwrap_or_else(|| {
                    let t = gtk::TextTag::builder().name("err").foreground("#ff7b72").weight(700).build();
                    buf.tag_table().add(&t); t
                });
                buf.insert_with_tags(&mut iter, &format!("{}\n", line), &[&tag]);
            } else {
                buf.insert(&mut iter, &format!("{}\n", line));
            }
        }
    };

    let update_rc = Rc::new(update_view);
    let u = update_rc.clone();
    list_box.connect_row_activated(move |_, row| {
        u(row.index());
    });

    window.present();
}

