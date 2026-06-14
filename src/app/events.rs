use crate::diagnostics::{analyze, DiagnosticPolicy};
use crate::journal::{fetch_boot_logs, start_live_stream};
use crate::models::log_entry::LogEntry;
use crate::ui::navigation::SidebarSection;
use crate::core::pipeline::apply_filter;
use crate::state::AppState;
use adw::prelude::*;
use gtk::prelude::*;

pub fn setup_lifecycle_events(
    window: &adw::ApplicationWindow,
    sidebar: &gtk::ListBox,
    search: &gtk::SearchEntry,
    refresh: &gtk::Button,
    status: &adw::StatusPage,
    store: &gtk::gio::ListStore,
    state: AppState,
) {
    let policy = DiagnosticPolicy::default();

    let state_init = state.clone();
    let store_init = store.clone();
    let status_init = status.clone();
    let policy_init = policy.clone();
    
    gtk::glib::spawn_future_local(async move {
        let initial_logs = tokio::task::spawn_blocking(|| {
            fetch_boot_logs().unwrap_or_default()
        }).await.unwrap_or_default();

        state_init.cache.replace(initial_logs.clone());
        
        let engine_clone = state_init.search_engine.clone();
        let logs_clone = initial_logs.clone();
        tokio::task::spawn_blocking(move || {
            let _ = engine_clone.index_logs(&logs_clone);
        }).await.ok();

        populate_store(&store_init, &initial_logs);

        let initial_result = analyze(&initial_logs, &policy_init, None);
        status_init.set_title(&initial_result.title);
        status_init.set_description(Some(&initial_result.description));
    });

    let state_sidebar = state.clone();
    let store_sidebar = store.clone();
    let status_sidebar = status.clone();
    let search_sidebar = search.clone();
    let policy_sidebar = policy.clone();

    sidebar.connect_row_selected(move |_, row| {
        let Some(row) = row else { return; };
        let section = section_from_index(row.index());
        
        let state_c = state_sidebar.clone();
        let store_c = store_sidebar.clone();
        let status_c = status_sidebar.clone();
        let policy_c = policy_sidebar.clone();
        let search_text = search_sidebar.text().to_string();

        gtk::glib::spawn_future_local(async move {
            let current_filter = {
                let mut filter_lock = state_c.filters.lock();
                filter_lock.section = section;
                filter_lock.clone()
            };
            
            let logs = state_c.cache.all();
            let filtered = apply_filter(&logs, &current_filter);
            
            let engine_clone = state_c.search_engine.clone();
            let filtered_clone = filtered.clone();
            tokio::task::spawn_blocking(move || {
                let _ = engine_clone.index_logs(&filtered_clone);
            }).await.ok();

            let final_logs = if search_text.trim().is_empty() {
                filtered
            } else {
                let engine_search = state_c.search_engine.clone();
                let q = search_text.clone();
                tokio::task::spawn_blocking(move || {
                    engine_search.search(&q, 5000).unwrap_or_default()
                }).await.unwrap_or_default()
            };

            populate_store(&store_c, &final_logs);
            let result = analyze(&final_logs, &policy_c, None);
            status_c.set_title(&result.title);
            status_c.set_description(Some(&result.description));
        });
    });

    let state_search = state.clone();
    let store_search = store.clone();
    let status_search = status.clone();
    let policy_search = policy.clone();

    search.connect_search_changed(move |entry| {
        let search_text = entry.text().to_string();
        
        let state_c = state_search.clone();
        let store_c = store_search.clone();
        let status_c = status_search.clone();
        let policy_c = policy_search.clone();

        gtk::glib::spawn_future_local(async move {
            let current_filter = {
                let mut filter_lock = state_c.filters.lock();
                filter_lock.query = search_text.clone();
                filter_lock.clone()
            };
            
            let logs = state_c.cache.all();
            let filtered = apply_filter(&logs, &current_filter);
            
            let engine_clone = state_c.search_engine.clone();
            let filtered_clone = filtered.clone();
            tokio::task::spawn_blocking(move || {
                let _ = engine_clone.index_logs(&filtered_clone);
            }).await.ok();

            let matched_logs = if search_text.trim().is_empty() {
                filtered
            } else {
                let engine_search = state_c.search_engine.clone();
                let q = search_text.clone();
                tokio::task::spawn_blocking(move || {
                    engine_search.search(&q, 5000).unwrap_or_default()
                }).await.unwrap_or_default()
            };

            populate_store(&store_c, &matched_logs);
            let result = analyze(&matched_logs, &policy_c, None);
            status_c.set_title(&result.title);
            status_c.set_description(Some(&result.description));
        });
    });

    let state_refresh = state.clone();
    let store_refresh = store.clone();
    let status_refresh = status.clone();
    let search_refresh = search.clone();
    let policy_refresh = policy.clone();

    refresh.connect_clicked(move |_| {
        let state_c = state_refresh.clone();
        let store_c = store_refresh.clone();
        let status_c = status_refresh.clone();
        let policy_c = policy_refresh.clone();
        let search_text = search_refresh.text().to_string();

        gtk::glib::spawn_future_local(async move {
            let logs = tokio::task::spawn_blocking(|| {
                fetch_boot_logs().unwrap_or_default()
            }).await.unwrap_or_default();

            state_c.cache.replace(logs.clone());
            
            let current_filter = state_c.filters.lock().clone();
            let filtered = apply_filter(&logs, &current_filter);
            
            let engine_clone = state_c.search_engine.clone();
            let filtered_clone = filtered.clone();
            tokio::task::spawn_blocking(move || {
                let _ = engine_clone.index_logs(&filtered_clone);
            }).await.ok();

            let matched_logs = if search_text.trim().is_empty() {
                filtered
            } else {
                let engine_search = state_c.search_engine.clone();
                let q = search_text.clone();
                tokio::task::spawn_blocking(move || {
                    engine_search.search(&q, 5000).unwrap_or_default()
                }).await.unwrap_or_default()
            };

            populate_store(&store_c, &matched_logs);
            let result = analyze(&matched_logs, &policy_c, None);
            status_c.set_title(&result.title);
            status_c.set_description(Some(&result.description));
        });
    });

    let (tx, rx) = async_channel::unbounded::<LogEntry>();
    start_live_stream(tx);

    let state_live = state.clone();
    let store_live = store.clone();
    let status_live = status.clone();
    let search_live = search.clone();
    let policy_live = policy.clone();

    gtk::glib::spawn_future_local(async move {
        while let Ok(entry) = rx.recv().await {
            state_live.cache.push(entry.clone());
            
            let engine_clone = state_live.search_engine.clone();
            let entry_clone = entry.clone();
            tokio::task::spawn_blocking(move || {
                let _ = engine_clone.index_single_log(&entry_clone);
            }).await.ok();
            
            let current_filter = state_live.filters.lock().clone();
            let logs = state_live.cache.all();
            let filtered = apply_filter(&logs, &current_filter);
            let search_text = search_live.text().to_string();
            
            let current_logs = if search_text.trim().is_empty() {
                filtered
            } else {
                let engine_search = state_live.search_engine.clone();
                let q = search_text.clone();
                tokio::task::spawn_blocking(move || {
                    engine_search.search(&q, 5000).unwrap_or_default()
                }).await.unwrap_or_default()
            };

            populate_store(&store_live, &current_logs);
            let result = analyze(&current_logs, &policy_live, None);
            status_live.set_title(&result.title);
            status_live.set_description(Some(&result.description));
        }
    });

    let about_action = gtk::gio::SimpleAction::new("about", None);
    let weak = window.downgrade();
    about_action.connect_activate(move |_, _| {
        if let Some(window) = weak.upgrade() {
            let dialog = adw::AboutDialog::builder()
                .application_name("System Logs Diagnostic")
                .application_icon("com.marcel.system-logs-diagnostic")
                .version("3.0.0-beta-1")
                .copyright("© 2026 Marcel Stevano")
                .license_type(gtk::License::Gpl30)
                .website("https://github.com/marcelstevano15/system-logs-diagnostic")
                .issue_url("https://github.com/marcelstevano15/system-logs-diagnostic/issues")
                .support_url("https://github.com/marcelstevano15/system-logs-diagnostic/discussions")
                .developer_name("Marcel Stevano")
                .developers(vec!["Marcel Stevano <marcelstevano15@gmail.com>"])
                .designers(vec!["Marcel Stevano"])
                .documenters(vec!["Marcel Stevano"])
                .build();
            dialog.present(Some(&window));
        }
    });
    window.add_action(&about_action);

    let sort_process = gtk::gio::SimpleAction::new("sort_process", None);
    let state_sort_process = state.clone();
    let store_sort_process = store.clone();
    sort_process.connect_activate(move |_, _| {
        let current_filter = state_sort_process.filters.lock().clone();
        let logs = state_sort_process.cache.all();
        let filtered = apply_filter(&logs, &current_filter);
        let mut sorted = filtered;
        sorted.sort_by(|a, b| a.process.cmp(&b.process));
        populate_store(&store_sort_process, &sorted);
    });
    window.add_action(&sort_process);

    let sort_sev_high = gtk::gio::SimpleAction::new("sort_sev_high", None);
    let state_sort_sev = state.clone();
    let store_sort_sev = store.clone();
    sort_sev_high.connect_activate(move |_, _| {
        let current_filter = state_sort_sev.filters.lock().clone();
        let logs = state_sort_sev.cache.all();
        let filtered = apply_filter(&logs, &current_filter);
        let mut sorted = filtered;
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        populate_store(&store_sort_sev, &sorted);
    });
    window.add_action(&sort_sev_high);
}

fn section_from_index(index: i32) -> SidebarSection {
    match index {
        0 => SidebarSection::LiveLogs,
        1 => SidebarSection::BootLogs,
        2 => SidebarSection::Kernel,
        3 => SidebarSection::Security,
        4 => SidebarSection::Services,
        5 => SidebarSection::Storage,
        6 => SidebarSection::Networking,
        _ => SidebarSection::LiveLogs,
    }
}

fn populate_store(store: &gtk::gio::ListStore, logs: &[LogEntry]) {
    store.remove_all();
    for log in logs {
        store.append(&gtk::glib::BoxedAnyObject::new(log.clone()));
    }
}

