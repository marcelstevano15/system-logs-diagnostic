use gtk::prelude::*;
use adw::prelude::*;
use std::process::Command;
use gtk::{glib, gio};
use adw::{Application, ApplicationWindow, HeaderBar, ToolbarView, StatusPage};
use std::rc::Rc;
use std::cell::RefCell;
use chrono::Local;
use serde_json::Value;

const APP_ID: &str = "com.marcel.system-logs-diagnostic";

#[derive(Clone)]
struct LogEntry {
    content: String,
    severity: i32,
    process: String,
}

struct DiagnosticResult {
    title: String,
    description: String,
    app_class: &'static str,
    icon: &'static str,
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

fn get_adaptive_color(severity_key: &str, is_dark: bool) -> &str {
    match (severity_key, is_dark) {
        ("panic", true) => "#FF7B7B",
        ("panic", false) => "#A10000",
        ("error", true) => "#F66151",
        ("error", false) => "#C01C28",
        ("warning", true) => "#FFBE6F",
        ("warning", false) => "#E66100",
        ("normal", true) => "#8FF0A4",
        ("normal", false) => "#26A269",
        _ => "#000000",
    }
}

fn diagnose_system(
    raw_input: &str,
    idx: i32,
) -> (String, String, &'static str, &'static str) {
    let current_time = Local::now().format("%H:%M:%S").to_string();
    if idx == 2 {
        return (
            "POWER ARCHITECTURE AUDIT".to_string(),
            format!("Executive summary of system power cycles at {}.", current_time),
            "success",
            "emblem-ok-symbolic",
        );
    }

    let mut panic_count = 0;
    let mut error_count = 0;
    let mut warning_count = 0;
    for line in raw_input.lines() {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if let Some(prio_str) = v["PRIORITY"].as_str() {
                let prio = prio_str.parse::<i32>().unwrap_or(6);
                match prio {
                    0..=2 => panic_count += 1,
                    3 => error_count += 1,
                    4 => warning_count += 1,
                    _ => (),
                }
            }
        }
    }

    if panic_count > 0 || (idx == 3 && error_count > 0) {
        (
            format!("Critical Operational Failure Detected: {} Events", panic_count + error_count),
            format!("Critical system events detected at {}. Logs are available for inspection.", current_time),
            "destructive",
            "software-update-urgent-symbolic",
        )
    } else if error_count > 0 {
          (
            format!("Services Failures: {} Events", error_count),
            format!("One or more services failed. Core system operating normally, reported at {}.", current_time),
            "destructive",
            "dialog-error-symbolic",
        )
    } else if warning_count > 0 {
        (
            format!("System Operational: {} Minor Services Events Recorded", warning_count),
            format!("System operational: {} non-critical events recorded at {}.", warning_count, current_time),
            "warning",
            "dialog-warning-symbolic",
        )
    } else {
        (
            "System Integrity: Verified".to_string(),
            format!("No anomalies detected. All parameters nominal at {}.", current_time),
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
        .title("System Logs Diagnostic")
        .default_width(1050)
        .default_height(800)
        .build();
    let style_manager = adw::StyleManager::default();
    let current_logs = Rc::new(RefCell::new(Vec::<LogEntry>::new()));
    
    let active_index = Rc::new(RefCell::new(0i32));
    
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
    let menu = ["Current Session Logs", "Last Shutdown Logs", "Power Audit", "System Errors", "Kernel Logs"];
    for item in menu {
        list_box.append(&adw::ActionRow::builder().title(item).activatable(true).build());
    }

    sidebar_toolbar.set_content(Some(&list_box));
    let content_header = HeaderBar::new();
    let search_bar = gtk::SearchEntry::builder().placeholder_text("Search logs...").hexpand(true).build();
    content_header.set_title_widget(Some(&search_bar));

    let refresh_btn = gtk::Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text("Refresh Current Logs")
        .build();
    content_header.pack_start(&refresh_btn);

    let sort_menu_model = gio::Menu::new();
    sort_menu_model.append(Some("Sort by Process"), Some("win.sort_process"));
    sort_menu_model.append(Some("Sort by Time (Newest)"), Some("win.sort_time_newest"));
    sort_menu_model.append(Some("Sort by Time (Oldest)"), Some("win.sort_time_oldest"));
    sort_menu_model.append(Some("Sort by Severity (Low)"), Some("win.sort_sev_low"));
    sort_menu_model.append(Some("Sort by Severity (High)"), Some("win.sort_sev_high"));
    let sort_btn = gtk::MenuButton::builder().icon_name("view-sort-ascending-symbolic").menu_model(&sort_menu_model).build();
    content_header.pack_start(&sort_btn);

    let main_menu_model = gio::Menu::new();
    main_menu_model.append(Some("About System Logs Diagnostic"), Some("win.about"));
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
        let sm = style_manager.clone();
        move |logs: &[LogEntry]| {
            buf.set_text("");
            let is_dark = sm.is_dark();
            for e in logs {
                let severity_key = match e.severity {
                    3 => "panic",
                    2 => "error",
                    1 => "warning",
                    _ => "normal",
                };
                let color = get_adaptive_color(severity_key, is_dark);
                let tag_id = format!("{}_{}", severity_key, if is_dark { "d" } else { "l" });
                let mut iter = buf.end_iter();
                let tag = buf.tag_table().lookup(&tag_id).unwrap_or_else(|| {
                    let t = gtk::TextTag::builder()
                        .name(&tag_id)
                        .foreground(color)
                        .weight(if severity_key == "panic" { 800 } else { 400 })
                        .build();
                    buf.tag_table().add(&t);
                    t
                });
                buf.insert_with_tags(&mut iter, &format!("[{}] {}\n", e.process, e.content), &[&tag]);
            }
        }
    };
    let render_logs_rc = Rc::new(render_logs);

    let (tx, rx) = async_channel::unbounded::<(i32, Vec<LogEntry>, Option<(DiagnosticResult, Vec<String>)>)>();

    let update_view = {
        let tx_clone = tx.clone();
        move |idx: i32| {
            let tx = tx_clone.clone();
            if idx == 2 {
                std::thread::spawn(move || {
                    let raw_power = Command::new("sh")
                        .arg("-c")
                        .arg("last -x | grep -E 'reboot|shutdown'")
                        .output();
                    if let Ok(out) = raw_power {
                        let raw_content = String::from_utf8_lossy(&out.stdout).to_string();
                        let lines: Vec<String> = raw_content.lines().map(|s| s.to_string()).collect();
                        let mut unclean_count = 0;
                        for i in 1..lines.len() {
                            let current = &lines[i];
                            if current.starts_with("reboot") {
                                if let Some(prev) = lines.get(i + 1) {
                                    if !prev.starts_with("shutdown") {
                                        unclean_count += 1;
                                    }
                                }
                            }
                        }
                        let diag = if unclean_count > 0 {
                            DiagnosticResult {
                                title: format!("Unclean Shutdown Detected ({})", unclean_count),
                                description: format!("Power Audit Detected {} Unclean Shutdown Event.", unclean_count),
                                app_class: "destructive",
                                icon: "dialog-error-symbolic",
                            }
                        } else {
                            DiagnosticResult {
                                title: "SYSTEM OPERATIONAL INTEGRITY: VERIFIED".to_string(),
                                description: "All historical power cycles conform to standard shutdown protocols.".to_string(),
                                app_class: "success",
                                icon: "emblem-ok-symbolic",
                            }
                        };
                        let _ = tx.send_blocking((idx, Vec::new(), Some((diag, lines))));
                    }
                });
                return;
            }

            std::thread::spawn(move || {
                let cmd = match idx {
                    0 => "journalctl -b -n 500",
                    1 => "journalctl -b -1 -n 500",
                    3 => "journalctl -b -p 0..3 -n 500",
                    _ => "journalctl -k -b -n 500",
                };
                let raw_json = run_bash_json(cmd);
                let logs = parse_logs_smart(&raw_json);
                let _ = tx.send_blocking((idx, logs, None));
            });
        }
    };

    let u_rc = Rc::new(update_view);

    glib::MainContext::default().spawn_local({
        let status = status_page.clone();
        let logs_store = current_logs.clone();
        let render = render_logs_rc.clone();
        let buf = buffer.clone();
        async move {
            while let Ok((idx, logs, diag_opt)) = rx.recv().await {
                status.remove_css_class("error");
                status.remove_css_class("warning");
                status.remove_css_class("success");

                if idx == 2 {
                    if let Some((diag, lines)) = diag_opt {
                        buf.set_text("");
                        let tag_panic = buf.tag_table().lookup("panic_d").unwrap_or_else(|| {
                            let t = gtk::TextTag::builder().name("panic_d").foreground("#FF7B7B").weight(800).build();
                            buf.tag_table().add(&t);
                            t
                        });
                        for i in 1..lines.len() {
                            let current = &lines[i];
                            let mut is_unclean = false;
                            if current.starts_with("reboot") {
                                if let Some(prev) = lines.get(i + 1) {
                                    if !prev.starts_with("shutdown") {
                                        is_unclean = true;
                                    }
                                }
                            }
                            let mut iter = buf.end_iter();
                            if is_unclean || current.contains("crash") {
                                buf.insert_with_tags(&mut iter, &format!("{}\n", current), &[&tag_panic]);
                            } else {
                                buf.insert(&mut iter, &format!("{}\n", current));
                            }
                        }
                        status.set_title(&diag.title);
                        status.set_description(Some(&diag.description));
                        status.set_icon_name(Some(diag.icon));
                        match diag.app_class {
                            "destructive" => status.add_css_class("error"),
                            _ => status.add_css_class("success"),
                        }
                        logs_store.borrow_mut().clear();
                    }
                    continue;
                }

                let cmd = match idx {
                    0 => "journalctl -b -n 500",
                    1 => "journalctl -b -1 -n 500",
                    3 => "journalctl -b -p 0..3 -n 500",
                    _ => "journalctl -k -b -n 500",
                };
                let raw_json_cmd = run_bash_json(cmd);

                let (t, d, app_class, icon) = diagnose_system(&raw_json_cmd, idx);
                status.set_title(&t);
                status.set_description(Some(&d));
                status.set_icon_name(Some(icon));
                match app_class {
                    "destructive" => status.add_css_class("error"),
                    "warning" => status.add_css_class("warning"),
                    "success" => status.add_css_class("success"),
                    _ => (),
                }

                *logs_store.borrow_mut() = logs.clone();
                render(&logs);
            }
        }
    });

    let u = u_rc.clone();
    let active_idx_clone = active_index.clone();
    list_box.connect_row_activated(move |_, row| {
        let idx = row.index();
        *active_idx_clone.borrow_mut() = idx;
        u(idx);
    });

    let u_refresh = u_rc.clone();
    let active_idx_refresh = active_index.clone();
    refresh_btn.connect_clicked(move |_| {
        let current_idx = *active_idx_refresh.borrow();
        u_refresh(current_idx);
    });

    style_manager.connect_dark_notify({
        let logs_store = current_logs.clone();
        let render = render_logs_rc.clone();
        move |_| {
            let logs = logs_store.borrow();
            if !logs.is_empty() {
                render(&logs);
            }
        }
    });

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
            let about = adw::AboutDialog::builder()
                .application_name("System Logs Diagnostic")
                .application_icon("com.marcel.system-logs-diagnostic")
                .version("1.5.0")
                .developer_name("Marcel Stevano")
                .license_type(gtk::License::Gpl30)
                .website("https://github.com/marcelstevano15/system-logs-diagnostic")
                .issue_url("https://github.com/marcelstevano15/system-logs-diagnostic/issues")
                .build();
            about.present(Some(&window));
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
                "new" => {}
                "old" => logs.reverse(),
                "low" => logs.sort_by(|a, b| a.severity.cmp(&b.severity)),
                "high" => logs.sort_by(|a, b| b.severity.cmp(&a.severity)),
                _ => (),
            }
            render(&logs);
        });
        window.add_action(&action);
    }

    u_rc(0);
    window.present();
}

