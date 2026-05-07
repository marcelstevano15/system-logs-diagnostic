use gtk::prelude::*;
use adw::prelude::*;
use std::process::Command;
use gtk::{glib, gio};
use adw::{Application, ApplicationWindow, HeaderBar, ToolbarView, StatusPage};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use chrono::Local;
use serde_json::Value;
use regex::Regex;

const APP_ID: &str = "io.github.marcel.system-logs-diagnostic";

#[derive(Clone)]
struct LogEntry {
    content: String,
    severity: i32,
    process: String,
}

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn run_bash_json(cmd: &str) -> String {
    let json_cmd = format!("{} -o json", cmd);

    let output = Command::new("sh")
        .arg("-c")
        .arg(&json_cmd)
        .output();

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => String::new(),
    }
}

fn diagnose_system(
    raw_input: &str,
    idx: i32,
) -> (String, String, &'static str, &'static str) {
    let current_time = Local::now().format("%H:%M:%S").to_string();

    if idx == 2 {
        let has_crash = raw_input.contains("crash") || raw_input.contains("gone");

        return if has_crash {
            (
                "POWER ANOMALY".to_string(),
                format!("Detected hardware-level shutdown at {}.", current_time),
                "destructive",
                "system-shutdown-symbolic",
            )
        } else {
            (
                "POWER SYSTEM STABLE".to_string(),
                format!("Clean shutdown sequences verified at {}.", current_time),
                "success",
                "system-shutdown-symbolic",
            )
        };
    }

    let mut error_count = 0;
    let mut critical_found = false;

    for line in raw_input.lines() {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if let Some(prio_str) = v["PRIORITY"].as_str() {
                let prio = prio_str.parse::<i32>().unwrap_or(6);
                if prio <= 2 { critical_found = true; }
                if prio <= 3 { error_count += 1; }
            }
        }
    }

    if critical_found {
        (
            "CRITICAL FAILURE".to_string(),
            format!("Fatal exceptions detected at {}.", current_time),
            "destructive",
            "software-update-urgent-symbolic",
        )
    } else if error_count > 0 {
        (
            format!("{} ERRORS DETECTED", error_count),
            format!("System reported service failures at {}.", current_time),
            "warning",
            "dialog-warning-symbolic",
        )
    } else {
        (
            "SYSTEM INTEGRITY VERIFIED".to_string(),
            format!("No anomalies found. Scan finished at {}.", current_time),
            "success",
            "emblem-ok-symbolic",
        )
    }
}

fn parse_logs_smart(raw_json: &str) -> Vec<LogEntry> {
    raw_json
        .lines()
        .filter_map(|line| {
            let v: Value = serde_json::from_str(line).ok()?;
            let message = v["MESSAGE"].as_str()?.to_string();
            let prio = v["PRIORITY"].as_str().unwrap_or("6").parse::<i32>().unwrap_or(6);
            let comm = v["_COMM"].as_str().unwrap_or("kernel").to_string();

            let severity = match prio {
                0..=2 => 3,
                3 => 2,
                4 => 1,
                _ => 0,
            };

            Some(LogEntry {
                content: message,
                severity,
                process: comm,
            })
        })
        .collect()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("System Diagnostic Pro")
        .default_width(1050)
        .default_height(800)
        .build();

    let current_logs = Rc::new(RefCell::new(Vec::<LogEntry>::new()));
    let buffer = gtk::TextBuffer::new(None);
    let split_view = adw::NavigationSplitView::new();

    let sidebar_header = HeaderBar::new();
    let sidebar_toolbar = ToolbarView::new();
    sidebar_toolbar.add_top_bar(&sidebar_header);

    let list_box = gtk::ListBox::new();
    list_box.add_css_class("navigation-sidebar");
    list_box.set_margin_start(12);
    list_box.set_margin_end(12);
    list_box.set_margin_top(12);
    list_box.set_margin_bottom(12);

    let menu = ["Active Session", "Last Shutdown", "Power Audit", "Critical Errors", "Kernel Logs"];

    for item in menu {
        list_box.append(&adw::ActionRow::builder().title(item).activatable(true).build());
    }

    sidebar_toolbar.set_content(Some(&list_box));

    let content_header = HeaderBar::new();
    let search_bar = gtk::SearchEntry::builder().placeholder_text("Search logs...").hexpand(true).build();
    content_header.set_title_widget(Some(&search_bar));

    let sort_menu_model = gio::Menu::new();
    sort_menu_model.append(Some("Sort by Process"), Some("win.sort_process"));
    sort_menu_model.append(Some("Sort by Time (Newest)"), Some("win.sort_time_newest"));
    sort_menu_model.append(Some("Sort by Time (Oldest)"), Some("win.sort_time_oldest"));
    sort_menu_model.append(Some("Sort by Severity (Low)"), Some("win.sort_sev_low"));
    sort_menu_model.append(Some("Sort by Severity (High)"), Some("win.sort_sev_high"));

    let sort_btn = gtk::MenuButton::builder().icon_name("view-sort-ascending-symbolic").menu_model(&sort_menu_model).build();
    content_header.pack_start(&sort_btn);

    let main_menu_model = gio::Menu::new();
    main_menu_model.append(Some("About"), Some("win.about"));
    let main_menu = gtk::MenuButton::builder().primary(true).icon_name("open-menu-symbolic").menu_model(&main_menu_model).build();
    content_header.pack_end(&main_menu);

    let content_toolbar = ToolbarView::new();
    content_toolbar.add_top_bar(&content_header);

    let status_page = StatusPage::builder().title("System Health").icon_name("system-search-symbolic").build();
    let log_view = gtk::TextView::builder().editable(false).monospace(true).buffer(&buffer).wrap_mode(gtk::WrapMode::WordChar).build();
    log_view.add_css_class("card");

    let scrolled = gtk::ScrolledWindow::builder().vexpand(true).child(&log_view).margin_start(20)
        .margin_end(20).margin_bottom(20).build();

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content_box.append(&status_page);
    content_box.append(&scrolled);
    content_toolbar.set_content(Some(&content_box));

    split_view.set_sidebar(Some(&adw::NavigationPage::new(&sidebar_toolbar, "Menu")));
    split_view.set_content(Some(&adw::NavigationPage::new(&content_toolbar, "Content")));
    window.set_content(Some(&split_view));

    let render_logs = {
        let buf = buffer.clone();
        move |logs: &[LogEntry]| {
            buf.set_text("");
            for e in logs {
                let tag_name = match e.severity {
                    3 => "panic",
                    2 => "err",
                    1 => "warn",
                    _ => "normal",
                };

                let mut iter = buf.end_iter();
                let tag = buf.tag_table().lookup(tag_name).unwrap_or_else(|| {
                    let t = match tag_name {
                        "panic" => gtk::TextTag::builder().name("panic").foreground("#e01b24").weight(800).build(),
                        "err" => gtk::TextTag::builder().name("err").foreground("#ff5d5a").build(),
                        "warn" => gtk::TextTag::builder().name("warn").foreground("#f5c211").build(),
                        _ => gtk::TextTag::builder().name("normal").build(),
                    };
                    buf.tag_table().add(&t);
                    t
                });
                buf.insert_with_tags(&mut iter, &format!("[{}] {}\n", e.process, e.content), &[&tag]);
            }
        }
    };

    let render_logs_rc = Rc::new(render_logs);

    let update_view = {
        let status = status_page.clone();
        let logs_store = current_logs.clone();
        let render = render_logs_rc.clone();
        let buf = buffer.clone();

        move |idx: i32| {
            if idx == 2 {
                let raw_power = Command::new("sh")
                    .arg("-c")
                    .arg("last -x | head -n 500")
                    .output();
                
                if let Ok(out) = raw_power {
                    let raw_content = String::from_utf8_lossy(&out.stdout).to_string();
                    let (t, d, _, icon) = diagnose_system(&raw_content, idx);
                    
                    let re = Regex::new(r"^(?P<event>reboot|shutdown)\s+\S+\s+(?P<kernel>\S+)\s+(?P<date>.+)$").unwrap();
                    let mut filtered_output = String::new();

                    for line in raw_content.lines() {
                        if let Some(caps) = re.captures(line) {
                            filtered_output.push_str(&format!(
                                "{:<10} {:<15} {}\n",
                                &caps["event"],
                                &caps["kernel"],
                                &caps["date"]
                            ));
                        }
                    }

                    status.set_title(&t);
                    status.set_description(Some(&d));
                    status.set_icon_name(Some(icon));
                    status.remove_css_class("error");
                    buf.set_text(&filtered_output);
                    logs_store.borrow_mut().clear();
                }
                return;
            }

            let cmd = match idx {
                0 => "journalctl -b -n 500",
                1 => "journalctl -b -1 -n 500",
                3 => "journalctl -b -p 0..3 -n 500",
                _ => "journalctl -k -b -n 500",
            };

            let raw_json = run_bash_json(cmd);
            let (t, d, app_class, icon) = diagnose_system(&raw_json, idx);
            status.set_title(&t);
            status.set_description(Some(&d));
            status.set_icon_name(Some(icon));

            if app_class == "destructive" { status.add_css_class("error"); } else { status.remove_css_class("error"); }

            let logs = parse_logs_smart(&raw_json);
            *logs_store.borrow_mut() = logs.clone();
            render(&logs);
        }
    };

    let u_rc = Rc::new(update_view);
    let u = u_rc.clone();
    list_box.connect_row_activated(move |_, row| { u(row.index()); });

    let u_logs = current_logs.clone();
    let render_search = render_logs_rc.clone();
    search_bar.connect_search_changed(move |entry| {
        let query = entry.text().to_lowercase();
        let logs = u_logs.borrow();
        let filtered: Vec<LogEntry> = logs.iter().filter(|x| x.content.to_lowercase().contains(&query)).cloned().collect();
        render_search(&filtered);
    });

    let about_action = gio::SimpleAction::new("about", None);
    let window_weak = window.downgrade();
    
    
    about_action.connect_activate(move |_, _| {
        if let Some(window) = window_weak.upgrade() {
            let about = adw::AboutWindow::builder()
                .application_name("System Diagnostic")
                .version("0.5.0-beta-2")
                .developer_name("Marcel Stevano")
                .license_type(gtk::License::Gpl30)
                .website("https://github.com/marcelstevano15/linux-native-desktop-apps")
                .issue_url("https://github.com/marcelstevano15/linux-native-desktop-apps/issues")
                .transient_for(&window)
                .build();

            
            about.set_application_icon("help-about-symbolic");
            about.present();
        }
    });
    window.add_action(&about_action);

    let sort_actions = [
        ("sort_process", "proc"), 
        ("sort_time_newest", "new"), 
        ("sort_time_oldest", "old"), 
        ("sort_sev_low", "low"), 
        ("sort_sev_high", "high")
    ];

    for (action_name, mode) in sort_actions {
        let action = gio::SimpleAction::new(action_name, None);
        let logs_store = current_logs.clone();
        let render = render_logs_rc.clone();
        action.connect_activate(move |_, _| {
            let mut logs = logs_store.borrow().clone();
            match mode {
                "proc" => logs.sort_by(|a, b| a.process.to_lowercase().cmp(&b.process.to_lowercase())),
                "new" => { /* Newest adalah urutan asli journalctl */ },
                "old" => logs.reverse(),
                "low" => logs.sort_by(|a, b| a.severity.cmp(&b.severity)),
                "high" => logs.sort_by(|a, b| b.severity.cmp(&a.severity)),
                _ => (),
            }
            render(&logs);
        });
        window.add_action(&action);
    }
    window.present();
}

