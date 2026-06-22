use crate::core::sort::{apply_sort, SortKey};
use crate::diagnostics::power_audit::PowerCycleEntry;
use crate::state::AppState;
use crate::ui::columns::{
    create_message_column, create_process_column, create_severity_column,
    create_timestamp_column, create_unit_column,
};
use crate::ui::detail_panel::DetailPanel;
use crate::ui::navigation::SidebarSection;
use crate::ui::power_audit_columns::{
    create_power_event_column, create_power_status_column, create_power_terminal_column,
    create_power_timestamp_column, create_power_user_column,
};
use crate::ui::stats_bar::StatsBar;
use crate::ui::stylesheet::APP_CSS;
use crate::app::events;
use adw::prelude::*;
use gtk::gio;
use gtk::glib;
use std::rc::Rc;

pub fn create_application_window(app: &adw::Application) {
    if let Err(e) = try_create_application_window(app) {
        tracing::error!("Fatal initialization error: {:?}", e);

        let dialog = adw::MessageDialog::new(
            None::<&adw::ApplicationWindow>,
            Some("Initialization Failed"),
            Some(&format!(
                "System Logs Diagnostic could not start:\n\n{}\n\nPlease check that systemd is running and sufficient memory is available.",
                e
            )),
        );
        dialog.add_response("close", "Close");
        dialog.set_default_response(Some("close"));
        dialog.connect_response(None, |d, _| d.close());
        dialog.present();
    }
}

fn try_create_application_window(app: &adw::Application) -> crate::errors::AppResult<()> {
    load_stylesheet();

    let state = AppState::new()?;
    let cfg = state.config.lock().clone();

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("System Logs Diagnostic")
        .default_width(cfg.window_width)
        .default_height(cfg.window_height)
        .build();

    window.set_size_request(crate::config::MIN_WINDOW_WIDTH, crate::config::MIN_WINDOW_HEIGHT);

    if cfg.window_maximized {
        window.maximize();
    }

    let toast_overlay = adw::ToastOverlay::new();

    let split = adw::NavigationSplitView::new();
    split.set_sidebar_width_fraction(0.22);
    split.set_min_sidebar_width(200.0);
    split.set_max_sidebar_width(320.0);

    let breakpoint = adw::Breakpoint::new(adw::BreakpointCondition::new_length(
        adw::BreakpointConditionLengthType::MaxWidth,
        860.0,
        adw::LengthUnit::Sp,
    ));
    breakpoint.add_setter(&split, "collapsed", Some(&true.to_value()));
    window.add_breakpoint(breakpoint);

    let sidebar_page = build_sidebar();
    let (
        content_page,
        log_store,
        power_store,
        search,
        refresh,
        live_button,
        status,
        stats_bar,
        detail_panel,
        log_column_view,
        power_column_view,
        view_stack,
    ) = build_content_page();

    split.set_sidebar(Some(&sidebar_page.0));
    split.set_content(Some(&content_page));
    toast_overlay.set_child(Some(&split));
    window.set_content(Some(&toast_overlay));

    let detail_panel = Rc::new(detail_panel);
    let stats_bar = Rc::new(stats_bar);

    let state_live = state.clone();
    live_button.connect_toggled(move |btn| {
        let paused = state_live.toggle_live_pause();
        if paused {
            btn.set_icon_name("media-playback-start-symbolic");
            btn.set_tooltip_text(Some("Resume Live Stream"));
        } else {
            btn.set_icon_name("media-playback-pause-symbolic");
            btn.set_tooltip_text(Some("Pause Live Stream"));
        }
    });

    register_actions(
        &window,
        &toast_overlay,
        &log_store,
        &search,
        &refresh,
        &status,
        &stats_bar,
        state.clone(),
    );

    window.present();

    events::setup_lifecycle_events(
        &window,
        &sidebar_page.1,
        &search,
        &refresh,
        &status,
        &stats_bar,
        &log_store,
        &power_store,
        &log_column_view,
        &view_stack,
        detail_panel,
        state,
    );

    Ok(())
}

fn build_sidebar() -> (adw::NavigationPage, gtk::ListBox) {
    let toolbar_view = adw::ToolbarView::new();

    let header = adw::HeaderBar::new();
    header.set_show_start_title_buttons(true);
    header.set_show_end_title_buttons(false);
    header.set_decoration_layout(Some(":"));

    let sidebar_title = adw::WindowTitle::new("Sections", "");
    header.set_title_widget(Some(&sidebar_title));
    toolbar_view.add_top_bar(&header);

    let sidebar = gtk::ListBox::new();
    sidebar.add_css_class("navigation-sidebar");
    sidebar.set_vexpand(true);

    for section in SidebarSection::all() {
        let row = adw::ActionRow::builder()
            .title(section.to_string())
            .activatable(true)
            .build();

        let icon = gtk::Image::from_icon_name(section.icon_name());
        icon.set_icon_size(gtk::IconSize::Normal);
        row.add_prefix(&icon);

        sidebar.append(&row);
    }

    let scroll = gtk::ScrolledWindow::builder()
        .child(&sidebar)
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();

    toolbar_view.set_content(Some(&scroll));

    let page = adw::NavigationPage::new(&toolbar_view, "Sections");
    (page, sidebar)
}

#[allow(clippy::type_complexity)]
fn build_content_page() -> (
    adw::NavigationPage,
    gio::ListStore,
    gio::ListStore,
    gtk::SearchEntry,
    gtk::Button,
    gtk::ToggleButton,
    adw::StatusPage,
    StatsBar,
    DetailPanel,
    gtk::ColumnView,
    gtk::ColumnView,
    gtk::Stack,
) {
    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();

    let search = gtk::SearchEntry::builder()
        .placeholder_text("Search logs")
        .hexpand(true)
        .build();
    header.set_title_widget(Some(&search));

    let refresh = gtk::Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text("Refresh Logs")
        .build();
    header.pack_start(&refresh);

    let live_button = gtk::ToggleButton::builder()
        .icon_name("media-playback-pause-symbolic")
        .tooltip_text("Pause Live Stream")
        .build();
    header.pack_start(&live_button);

    let sort_menu = build_sort_menu();
    let sort_button = gtk::MenuButton::builder()
        .icon_name("view-sort-ascending-symbolic")
        .tooltip_text("Sort Logs")
        .menu_model(&sort_menu)
        .build();
    header.pack_start(&sort_button);

    let export_menu = gio::Menu::new();
    let export_section = gio::Menu::new();
    export_section.append(Some("Export as JSON"), Some("win.export_json"));
    export_section.append(Some("Export as CSV"), Some("win.export_csv"));
    export_section.append(Some("Export as Archive (.tar.gz)"), Some("win.export_archive"));
    export_menu.append_section(Some("Export"), &export_section);

    let about_section = gio::Menu::new();
    about_section.append(Some("About"), Some("win.about"));
    export_menu.append_section(None, &about_section);

    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&export_menu)
        .build();
    header.pack_end(&menu_button);

    toolbar.add_top_bar(&header);

    let status = adw::StatusPage::builder()
        .title("System Health")
        .description("Loading system logs…")
        .icon_name("utilities-system-monitor-symbolic")
        .build();

    let stats_bar = StatsBar::new();
    toolbar.add_bottom_bar(&stats_bar.container);

    let log_store = gio::ListStore::new::<glib::BoxedAnyObject>();
    let log_selection = gtk::SingleSelection::new(Some(log_store.clone()));
    let log_column_view = gtk::ColumnView::new(Some(log_selection));
    log_column_view.set_vexpand(true);
    log_column_view.set_hexpand(true);
    log_column_view.set_show_column_separators(true);
    log_column_view.set_show_row_separators(true);
    log_column_view.add_css_class("data-table");

    log_column_view.append_column(&create_timestamp_column());
    log_column_view.append_column(&create_severity_column());
    log_column_view.append_column(&create_process_column());
    log_column_view.append_column(&create_unit_column());
    log_column_view.append_column(&create_message_column());

    let log_scroll = gtk::ScrolledWindow::builder()
        .child(&log_column_view)
        .vexpand(true)
        .hexpand(true)
        .build();

    let power_store = gio::ListStore::new::<glib::BoxedAnyObject>();
    let power_selection = gtk::SingleSelection::new(Some(power_store.clone()));
    let power_column_view = gtk::ColumnView::new(Some(power_selection));
    power_column_view.set_vexpand(true);
    power_column_view.set_hexpand(true);
    power_column_view.set_show_column_separators(true);
    power_column_view.set_show_row_separators(true);
    power_column_view.add_css_class("data-table");

    power_column_view.append_column(&create_power_event_column());
    power_column_view.append_column(&create_power_user_column());
    power_column_view.append_column(&create_power_terminal_column());
    power_column_view.append_column(&create_power_timestamp_column());
    power_column_view.append_column(&create_power_status_column());

    let power_scroll = gtk::ScrolledWindow::builder()
        .child(&power_column_view)
        .vexpand(true)
        .hexpand(true)
        .build();

    let view_stack = gtk::Stack::new();
    view_stack.set_transition_type(gtk::StackTransitionType::None);
    view_stack.add_named(&log_scroll, Some("logs"));
    view_stack.add_named(&power_scroll, Some("power"));
    view_stack.set_visible_child_name("logs");

    let detail_panel = DetailPanel::new();
    detail_panel.clear();

    let main_split = gtk::Paned::new(gtk::Orientation::Horizontal);
    main_split.set_wide_handle(true);

    let left_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    left_box.append(&status);
    left_box.append(&view_stack);
    left_box.set_hexpand(true);

    main_split.set_start_child(Some(&left_box));
    main_split.set_end_child(Some(&detail_panel.container));
    main_split.set_resize_start_child(true);
    main_split.set_resize_end_child(false);
    main_split.set_shrink_end_child(true);

    toolbar.set_content(Some(&main_split));

    let page = adw::NavigationPage::new(&toolbar, "Logs");

    (
        page,
        log_store,
        power_store,
        search,
        refresh,
        live_button,
        status,
        stats_bar,
        detail_panel,
        log_column_view,
        power_column_view,
        view_stack,
    )
}

fn build_sort_menu() -> gio::Menu {
    let menu = gio::Menu::new();

    let time_section = gio::Menu::new();
    time_section.append(Some("Time: Newest First"), Some("win.sort_time_newest"));
    time_section.append(Some("Time: Oldest First"), Some("win.sort_time_oldest"));
    menu.append_section(Some("Time"), &time_section);

    let sev_section = gio::Menu::new();
    sev_section.append(Some("Severity: High → Low"), Some("win.sort_sev_high_low"));
    sev_section.append(Some("Severity: Low → High"), Some("win.sort_sev_low_high"));
    menu.append_section(Some("Severity"), &sev_section);

    let process_section = gio::Menu::new();
    process_section.append(Some("Process: A → Z"), Some("win.sort_process_az"));
    process_section.append(Some("Process: Z → A"), Some("win.sort_process_za"));
    menu.append_section(Some("Process"), &process_section);

    let unit_section = gio::Menu::new();
    unit_section.append(Some("Unit: A → Z"), Some("win.sort_unit_az"));
    unit_section.append(Some("Unit: Z → A"), Some("win.sort_unit_za"));
    menu.append_section(Some("Unit"), &unit_section);

    let host_section = gio::Menu::new();
    host_section.append(Some("Hostname: A → Z"), Some("win.sort_hostname_az"));
    host_section.append(Some("Hostname: Z → A"), Some("win.sort_hostname_za"));
    menu.append_section(Some("Hostname"), &host_section);

    let pid_section = gio::Menu::new();
    pid_section.append(Some("PID: Ascending"), Some("win.sort_pid_asc"));
    pid_section.append(Some("PID: Descending"), Some("win.sort_pid_desc"));
    menu.append_section(Some("PID"), &pid_section);

    menu
}

fn register_sort_action(
    window: &adw::ApplicationWindow,
    store: &gio::ListStore,
    state: &AppState,
    key: SortKey,
) {
    let action_full = key.action_name();
    let action_name = action_full.trim_start_matches("win.");

    let action = gio::SimpleAction::new(action_name, None);
    let store_c = store.clone();
    let state_c = state.clone();

    action.connect_activate(move |_, _| {
        state_c.set_sort(key.clone());

        let store_cc = store_c.clone();
        let logs = state_c.cache.all();
        let filter = state_c.filters.lock().clone();
        let key_cc = key.clone();

        glib::spawn_future_local(async move {
            let filtered = gio::spawn_blocking(move || {
                let mut filtered = crate::core::pipeline::apply_filter(&logs, &filter);
                apply_sort(&mut filtered, &key_cc);
                filtered
            })
            .await
            .unwrap_or_default();

            populate_log_store(&store_cc, &filtered);
        });
    });

    window.add_action(&action);
}

fn register_actions(
    window: &adw::ApplicationWindow,
    toast_overlay: &adw::ToastOverlay,
    store: &gio::ListStore,
    _search: &gtk::SearchEntry,
    _refresh: &gtk::Button,
    _status: &adw::StatusPage,
    _stats_bar: &Rc<StatsBar>,
    state: AppState,
) {
    let about_action = gio::SimpleAction::new("about", None);
    let weak = window.downgrade();
    about_action.connect_activate(move |_, _| {
        if let Some(w) = weak.upgrade() {
            let dialog = adw::AboutDialog::builder()
                .application_name("System Logs Diagnostic")
                .application_icon("com.marcel.system-logs-diagnostic")
                .version(env!("CARGO_PKG_VERSION"))
                .copyright("© 2026 Marcel Stevano")
                .license_type(gtk::License::Gpl30)
                .website("https://github.com/marcelstevano15/system-logs-diagnostic")
                .issue_url("https://github.com/marcelstevano15/system-logs-diagnostic/issues")
                .support_url("https://github.com/marcelstevano15/system-logs-diagnostic/discussions")
                .developer_name("Marcel Stevano")
                .developers(vec!["Marcel Stevano <marcelstevano15@gmail.com>"])
                .designers(vec!["Marcel Stevano"])
                .build();
            dialog.present(Some(&w));
        }
    });
    window.add_action(&about_action);

    let export_json = gio::SimpleAction::new("export_json", None);
    let state_ej = state.clone();
    let weak_ej = window.downgrade();
    let overlay_ej = toast_overlay.clone();
    export_json.connect_activate(move |_, _| {
        if let Some(w) = weak_ej.upgrade() {
            let logs = state_ej.cache.all();
            show_export_dialog(&w, &overlay_ej, "json", logs);
        }
    });
    window.add_action(&export_json);

    let export_csv = gio::SimpleAction::new("export_csv", None);
    let state_ec = state.clone();
    let weak_ec = window.downgrade();
    let overlay_ec = toast_overlay.clone();
    export_csv.connect_activate(move |_, _| {
        if let Some(w) = weak_ec.upgrade() {
            let logs = state_ec.cache.all();
            show_export_dialog(&w, &overlay_ec, "csv", logs);
        }
    });
    window.add_action(&export_csv);

    let export_archive = gio::SimpleAction::new("export_archive", None);
    let state_ea = state.clone();
    let weak_ea = window.downgrade();
    let overlay_ea = toast_overlay.clone();
    export_archive.connect_activate(move |_, _| {
        if let Some(w) = weak_ea.upgrade() {
            let logs = state_ea.cache.all();
            show_export_dialog(&w, &overlay_ea, "archive", logs);
        }
    });
    window.add_action(&export_archive);

    for key in SortKey::all() {
        register_sort_action(window, store, &state, key.clone());
    }
}

fn show_export_dialog(
    window: &adw::ApplicationWindow,
    toast_overlay: &adw::ToastOverlay,
    format: &str,
    logs: Vec<crate::models::log_entry::LogEntry>,
) {
    use gtk::gio::prelude::FileExt;

    let fmt = format.to_string();
    let (suffix, title) = match format {
        "json" => ("logs.json", "Export as JSON"),
        "csv" => ("logs.csv", "Export as CSV"),
        _ => ("logs.tar.gz", "Export as Archive"),
    };

    let dialog = gtk::FileDialog::builder()
        .title(title)
        .initial_name(suffix)
        .build();

    let overlay_weak = toast_overlay.clone();
    let log_count = logs.len();

    dialog.save(Some(window), gio::Cancellable::NONE, move |result| {
        if let Ok(file) = result {
            let path = file.path().unwrap_or_default();
            let result = match fmt.as_str() {
                "json" => crate::export::json_export::export(&path, &logs),
                "csv" => crate::export::csv_export::export(&path, &logs),
                _ => crate::export::archive::export_archive(&path, &logs),
            };

            let msg = match result {
                Ok(()) => {
                    tracing::info!("Exported {} log entries to {:?}", log_count, path);
                    format!("Exported {} log entries", log_count)
                }
                Err(e) => {
                    tracing::error!("Export failed: {}", e);
                    format!("Export failed: {}", e)
                }
            };

            let toast = adw::Toast::new(&msg);
            overlay_weak.add_toast(toast);
        }
    });
}

pub fn populate_log_store(
    store: &gio::ListStore,
    logs: &[crate::models::log_entry::LogEntry],
) {
    let total = logs.len();
    let to_remove = store.n_items();

    if total == 0 {
        store.splice(0, to_remove, &[] as &[glib::BoxedAnyObject]);
        return;
    }

    let objects: Vec<glib::BoxedAnyObject> = logs
        .iter()
        .map(|log| glib::BoxedAnyObject::new(log.clone()))
        .collect();

    store.splice(0, to_remove, &objects);
}

pub fn populate_power_store(store: &gio::ListStore, entries: &[PowerCycleEntry]) {
    let total = entries.len();
    let to_remove = store.n_items();

    if total == 0 {
        store.splice(0, to_remove, &[] as &[glib::BoxedAnyObject]);
        return;
    }

    let objects: Vec<glib::BoxedAnyObject> = entries
        .iter()
        .map(|e| glib::BoxedAnyObject::new(e.clone()))
        .collect();

    store.splice(0, to_remove, &objects);
}

fn load_stylesheet() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(APP_CSS);

    match gdk4::Display::default() {
        Some(display) => {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        None => {
            tracing::warn!("No display available; stylesheet not loaded. UI will use default theme.");
        }
    }
}
