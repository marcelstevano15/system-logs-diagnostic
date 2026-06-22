use crate::app::window::{populate_log_store, populate_power_store};
use crate::core::pipeline::apply_filter;
use crate::core::sort::apply_sort;
use crate::diagnostics::{
    analyze, analyze_power_cycles, fetch_power_cycles,
    power_audit_diagnostic, DiagnosticPolicy,
};
use crate::journal::{fetch_boot_logs, start_live_stream};
use crate::models::log_entry::LogEntry;
use crate::state::AppState;
use crate::ui::detail_panel::DetailPanel;
use crate::ui::navigation::SidebarSection;
use crate::ui::stats_bar::StatsBar;
use adw::prelude::*;
use chrono::Utc;
use gtk::gio;
use gtk::glib;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

const DEBOUNCE_MS: u64 = 300;
const SEARCH_RESULT_LIMIT: usize = 50_000;
const LIVE_STREAM_SNAPSHOT_SIZE: usize = 5_000;

fn critical_focus_description(result: &crate::diagnostics::DiagnosticResult) -> String {
    format!(
        "Critical: {} | Errors: {} | Total: {}",
        result.critical_count, result.error_count, result.total_count
    )
}

pub fn setup_lifecycle_events(
    window: &adw::ApplicationWindow,
    sidebar: &gtk::ListBox,
    search: &gtk::SearchEntry,
    refresh: &gtk::Button,
    status: &adw::StatusPage,
    stats_bar: &Rc<StatsBar>,
    log_store: &gio::ListStore,
    power_store: &gio::ListStore,
    log_column_view: &gtk::ColumnView,
    view_stack: &gtk::Stack,
    detail_panel: Rc<DetailPanel>,
    state: AppState,
) {
    let policy = Arc::new(DiagnosticPolicy::default());

    setup_initial_load(log_store, status, stats_bar, &state, &policy);
    setup_sidebar_events(
        sidebar,
        search,
        log_store,
        power_store,
        status,
        stats_bar,
        view_stack,
        &state,
        &policy,
    );
    setup_search_events(search, log_store, status, stats_bar, &state, &policy);
    setup_refresh_events(
        refresh,
        search,
        log_store,
        power_store,
        status,
        stats_bar,
        view_stack,
        &state,
        &policy,
    );
    setup_row_selection(log_column_view, log_store, detail_panel);
    setup_window_save(window, &state);

    let stream_handle = setup_live_stream(log_store, search, status, stats_bar, &state, &policy);
    state.store_stream_handle(stream_handle);
}

fn setup_initial_load(
    store: &gio::ListStore,
    status: &adw::StatusPage,
    stats_bar: &Rc<StatsBar>,
    state: &AppState,
    policy: &Arc<DiagnosticPolicy>,
) {
    let store_c = store.clone();
    let status_c = status.clone();
    let stats_c = stats_bar.clone();
    let state_c = state.clone();
    let policy_c = policy.clone();

    glib::spawn_future_local(async move {
        state_c.set_loading(true);
        status_c.set_description(Some("Loading system logs…"));

        let limit = state_c.config.lock().journal_boot_limit;

        let logs = gio::spawn_blocking(move || {
            fetch_boot_logs(limit).unwrap_or_else(|e| {
                error!("Failed to fetch boot logs: {:?}", e);
                Vec::new()
            })
        })
        .await
        .unwrap_or_default();

        state_c.cache.replace(logs.clone());

        let engine = state_c.search_engine.clone();
        let logs_idx = logs.clone();
        gio::spawn_blocking(move || {
            if let Err(e) = engine.index_logs(&logs_idx) {
                error!("Initial indexing failed: {:?}", e);
            }
        })
        .await
        .ok();

        let filter = state_c.filters.lock().clone();
        let section = filter.section;
        let sort_key = state_c.get_sort();
        let policy_bg = policy_c.clone();
        let now = Utc::now();

        let (filtered, result) = gio::spawn_blocking(move || {
            let mut filtered = apply_filter(&logs, &filter);
            apply_sort(&mut filtered, &sort_key);
            let result = analyze(&filtered, &policy_bg, now);
            (filtered, result)
        })
        .await
        .unwrap_or_else(|_| (Vec::new(), crate::diagnostics::DiagnosticResult::empty()));

        populate_log_store(&store_c, &filtered);

        status_c.set_title(&result.title);

        state_c.set_log_diagnostic_result(result.clone());
        if section.is_critical() {
            status_c.set_description(Some(&critical_focus_description(&result)));
            stats_c.update_critical_focus(&result);
        } else {
            status_c.set_description(Some(&result.description));
            stats_c.update(&result);
        }

        state_c.set_loading(false);
        info!("Initial load complete: {} entries", filtered.len());
    });
}

fn setup_sidebar_events(
    sidebar: &gtk::ListBox,
    search: &gtk::SearchEntry,
    log_store: &gio::ListStore,
    power_store: &gio::ListStore,
    status: &adw::StatusPage,
    stats_bar: &Rc<StatsBar>,
    view_stack: &gtk::Stack,
    state: &AppState,
    policy: &Arc<DiagnosticPolicy>,
) {
    let log_store_c = log_store.clone();
    let power_store_c = power_store.clone();
    let status_c = status.clone();
    let stats_c = stats_bar.clone();
    let state_c = state.clone();
    let policy_c = policy.clone();
    let search_c = search.clone();
    let view_stack_c = view_stack.clone();

    sidebar.connect_row_selected(move |_, row| {
        let Some(row) = row else { return };
        let section = SidebarSection::from_index(row.index());
        let search_text = search_c.text().to_string();

        let log_store_cc = log_store_c.clone();
        let power_store_cc = power_store_c.clone();
        let status_cc = status_c.clone();
        let stats_cc = stats_c.clone();
        let state_cc = state_c.clone();
        let policy_cc = policy_c.clone();
        let view_stack_cc = view_stack_c.clone();

        {
            let mut filter = state_cc.filters.lock();
            filter.section = section;
        }

        if section.is_power_audit() {
            view_stack_cc.set_visible_child_name("power");

            glib::spawn_future_local(async move {
                status_cc.set_title("Power Audit");
                status_cc.set_description(Some("Fetching power cycle history…"));

                let (entries, audit_result, power_result) = gio::spawn_blocking(move || {
                    let entries = fetch_power_cycles().unwrap_or_else(|e| {
                        error!("Failed to fetch power cycles: {:?}", e);
                        Vec::new()
                    });
                    let audit_result = analyze_power_cycles(&entries);
                    let power_result =
                        power_audit_diagnostic(&audit_result, &policy_cc, Utc::now());
                    (entries, audit_result, power_result)
                })
                .await
                .unwrap_or_else(|_| {
                    (
                        Vec::new(),
                        Default::default(),
                        crate::diagnostics::DiagnosticResult::empty(),
                    )
                });

                status_cc.set_title(&audit_result.title);
                status_cc.set_description(Some(&audit_result.description));

                state_cc.set_power_audit_result(power_result.clone());

                let clean = audit_result.total_count.saturating_sub(audit_result.unclean_count);
                stats_cc.update_power(clean, audit_result.unclean_count, audit_result.total_count);

                populate_power_store(&power_store_cc, &entries);

                info!(
                    "Power audit complete: {} entries, {} unclean",
                    audit_result.total_count, audit_result.unclean_count
                );
            });

            return;
        }

        view_stack_cc.set_visible_child_name("logs");

        glib::spawn_future_local(async move {
            let logs = state_cc.cache.all();
            let filter = state_cc.filters.lock().clone();
            let sort_key = state_cc.get_sort();
            let policy_bg = policy_cc.clone();
            let now = Utc::now();
            let engine = state_cc.search_engine.clone();
            let q = search_text.clone();

            let (final_logs, result) = gio::spawn_blocking(move || {
                let mut results = if q.trim().is_empty() {
                    apply_filter(&logs, &filter)
                } else {
                    let matched_ids = engine.search(&q, SEARCH_RESULT_LIMIT).unwrap_or_default();
                    let all_matched = filter_by_seq_ids(&matched_ids, &logs);
                    apply_filter(&all_matched, &filter)
                };
                apply_sort(&mut results, &sort_key);
                let result = analyze(&results, &policy_bg, now);
                (results, result)
            })
            .await
            .unwrap_or_else(|_| (Vec::new(), crate::diagnostics::DiagnosticResult::empty()));

            populate_log_store(&log_store_cc, &final_logs);
            status_cc.set_title(&result.title);

            state_cc.set_log_diagnostic_result(result.clone());
            if section.is_critical() {
                status_cc.set_description(Some(&critical_focus_description(&result)));
                stats_cc.update_critical_focus(&result);
            } else {
                status_cc.set_description(Some(&result.description));
                stats_cc.update(&result);
            }
        });
    });
}

fn setup_search_events(
    search: &gtk::SearchEntry,
    store: &gio::ListStore,
    status: &adw::StatusPage,
    stats_bar: &Rc<StatsBar>,
    state: &AppState,
    policy: &Arc<DiagnosticPolicy>,
) {
    let store_c = store.clone();
    let status_c = status.clone();
    let stats_c = stats_bar.clone();
    let state_c = state.clone();
    let policy_c = policy.clone();

    let debounce_id: Rc<std::cell::Cell<Option<glib::SourceId>>> =
        Rc::new(std::cell::Cell::new(None));
    let debounce_fired: Rc<std::cell::Cell<bool>> = Rc::new(std::cell::Cell::new(true));

    search.connect_search_changed(move |entry| {
        let current_section = state_c.filters.lock().section;
        if current_section.is_power_audit() {
            return;
        }

        let search_text = entry.text().to_string();

        let store_cc = store_c.clone();
        let status_cc = status_c.clone();
        let stats_cc = stats_c.clone();
        let state_cc = state_c.clone();
        let policy_cc = policy_c.clone();

        if let Some(id) = debounce_id.take() {
            if !debounce_fired.get() {
                id.remove();
            }
        }
        debounce_fired.set(false);

        let debounce_fired_cc = debounce_fired.clone();
        let search_text_clone = search_text.clone();
        let new_id = glib::timeout_add_local_once(
            Duration::from_millis(DEBOUNCE_MS),
            move || {
                debounce_fired_cc.set(true);

                {
                    let mut filter = state_cc.filters.lock();
                    filter.query = search_text_clone.clone();
                }

                glib::spawn_future_local(async move {
                    let logs = state_cc.cache.all();
                    let filter = state_cc.filters.lock().clone();
                    let sort_key = state_cc.get_sort();
                    let policy_bg = policy_cc.clone();
                    let now = Utc::now();
                    let engine = state_cc.search_engine.clone();
                    let q = search_text_clone.clone();

                    let (final_logs, result) = gio::spawn_blocking(move || {
                        let mut results = if q.trim().is_empty() {
                            apply_filter(&logs, &filter)
                        } else {
                            let matched_ids =
                                engine.search(&q, SEARCH_RESULT_LIMIT).unwrap_or_default();
                            let all_matched = filter_by_seq_ids(&matched_ids, &logs);
                            apply_filter(&all_matched, &filter)
                        };
                        apply_sort(&mut results, &sort_key);
                        let result = analyze(&results, &policy_bg, now);
                        (results, result)
                    })
                    .await
                    .unwrap_or_else(|_| {
                        (Vec::new(), crate::diagnostics::DiagnosticResult::empty())
                    });

                    populate_log_store(&store_cc, &final_logs);
                    status_cc.set_title(&result.title);

                    state_cc.set_log_diagnostic_result(result.clone());
                    if current_section.is_critical() {
                        status_cc.set_description(Some(&critical_focus_description(&result)));
                        stats_cc.update_critical_focus(&result);
                    } else {
                        status_cc.set_description(Some(&result.description));
                        stats_cc.update(&result);
                    }
                });
            },
        );

        debounce_id.set(Some(new_id));
    });
}

fn setup_refresh_events(
    refresh: &gtk::Button,
    search: &gtk::SearchEntry,
    log_store: &gio::ListStore,
    power_store: &gio::ListStore,
    status: &adw::StatusPage,
    stats_bar: &Rc<StatsBar>,
    view_stack: &gtk::Stack,
    state: &AppState,
    policy: &Arc<DiagnosticPolicy>,
) {
    let log_store_c = log_store.clone();
    let power_store_c = power_store.clone();
    let status_c = status.clone();
    let stats_c = stats_bar.clone();
    let state_c = state.clone();
    let policy_c = policy.clone();
    let search_c = search.clone();
    let view_stack_c = view_stack.clone();

    refresh.connect_clicked(move |btn| {
        btn.set_sensitive(false);

        let current_section = state_c.filters.lock().section;

        if current_section.is_power_audit() {
            let power_store_cc = power_store_c.clone();
            let status_cc = status_c.clone();
            let stats_cc = stats_c.clone();
            let state_cc = state_c.clone();
            let policy_cc = policy_c.clone();
            let btn_cc = btn.clone();
            let view_stack_cc = view_stack_c.clone();

            view_stack_cc.set_visible_child_name("power");

            glib::spawn_future_local(async move {
                status_cc.set_description(Some("Refreshing power cycle history…"));

                let (entries, audit_result, power_result) = gio::spawn_blocking(move || {
                    let entries = fetch_power_cycles().unwrap_or_else(|e| {
                        error!("Failed to refresh power cycles: {:?}", e);
                        Vec::new()
                    });
                    let audit_result = analyze_power_cycles(&entries);
                    let power_result =
                        power_audit_diagnostic(&audit_result, &policy_cc, Utc::now());
                    (entries, audit_result, power_result)
                })
                .await
                .unwrap_or_else(|_| {
                    (
                        Vec::new(),
                        Default::default(),
                        crate::diagnostics::DiagnosticResult::empty(),
                    )
                });

                status_cc.set_title(&audit_result.title);
                status_cc.set_description(Some(&audit_result.description));

                state_cc.set_power_audit_result(power_result.clone());

                let clean = audit_result.total_count.saturating_sub(audit_result.unclean_count);
                stats_cc.update_power(clean, audit_result.unclean_count, audit_result.total_count);

                populate_power_store(&power_store_cc, &entries);
                btn_cc.set_sensitive(true);
            });

            return;
        }

        let search_text = search_c.text().to_string();

        let log_store_cc = log_store_c.clone();
        let status_cc = status_c.clone();
        let stats_cc = stats_c.clone();
        let state_cc = state_c.clone();
        let policy_cc = policy_c.clone();
        let btn_cc = btn.clone();
        let view_stack_cc = view_stack_c.clone();

        view_stack_cc.set_visible_child_name("logs");

        glib::spawn_future_local(async move {
            status_cc.set_description(Some("Refreshing…"));
            let limit = state_cc.config.lock().journal_boot_limit;

            let logs = gio::spawn_blocking(move || {
                fetch_boot_logs(limit).unwrap_or_else(|e| {
                    error!("Refresh failed: {:?}", e);
                    Vec::new()
                })
            })
            .await
            .unwrap_or_default();

            state_cc.cache.replace(logs.clone());

            let engine = state_cc.search_engine.clone();
            let logs_idx = logs.clone();
            gio::spawn_blocking(move || {
                if let Err(e) = engine.index_logs(&logs_idx) {
                    error!("Refresh indexing failed: {:?}", e);
                }
            })
            .await
            .ok();

            let filter = state_cc.filters.lock().clone();
            let sort_key = state_cc.get_sort();
            let policy_bg = policy_cc.clone();
            let now = Utc::now();
            let engine2 = state_cc.search_engine.clone();
            let q = search_text.clone();

            let (final_logs, result) = gio::spawn_blocking(move || {
                let mut results = if q.trim().is_empty() {
                    apply_filter(&logs, &filter)
                } else {
                    let matched_ids =
                        engine2.search(&q, SEARCH_RESULT_LIMIT).unwrap_or_default();
                    let all_matched = filter_by_seq_ids(&matched_ids, &logs);
                    apply_filter(&all_matched, &filter)
                };
                apply_sort(&mut results, &sort_key);
                let result = analyze(&results, &policy_bg, now);
                (results, result)
            })
            .await
            .unwrap_or_else(|_| (Vec::new(), crate::diagnostics::DiagnosticResult::empty()));

            populate_log_store(&log_store_cc, &final_logs);
            status_cc.set_title(&result.title);

            state_cc.set_log_diagnostic_result(result.clone());
            if current_section.is_critical() {
                status_cc.set_description(Some(&critical_focus_description(&result)));
                stats_cc.update_critical_focus(&result);
            } else {
                status_cc.set_description(Some(&result.description));
                stats_cc.update(&result);
            }
            btn_cc.set_sensitive(true);
        });
    });
}

fn setup_live_stream(
    store: &gio::ListStore,
    search: &gtk::SearchEntry,
    status: &adw::StatusPage,
    stats_bar: &Rc<StatsBar>,
    state: &AppState,
    policy: &Arc<DiagnosticPolicy>,
) -> crate::journal::LiveStreamHandle {
    let (tx, rx) = async_channel::bounded::<Vec<LogEntry>>(64);
    let handle = start_live_stream(tx);

    let store_c = store.clone();
    let status_c = status.clone();
    let stats_c = stats_bar.clone();
    let state_c = state.clone();
    let policy_c = policy.clone();
    let search_c = search.clone();

    glib::spawn_future_local(async move {
        while let Ok(batch) = rx.recv().await {
            if state_c.is_live_paused() {
                continue;
            }

            if batch.is_empty() {
                continue;
            }

            let current_section = state_c.filters.lock().section;
            if current_section.is_power_audit() {
                state_c.cache.push_batch(batch);
                continue;
            }

            state_c.cache.push_batch(batch.clone());

            let engine = state_c.search_engine.clone();
            let batch_idx = batch.clone();
            gio::spawn_blocking(move || {
                if let Err(e) = engine.index_batch_no_commit(&batch_idx) {
                    error!("Live stream indexing failed: {:?}", e);
                }
            })
            .await
            .ok();

            let search_text = search_c.text().to_string();
            let filter = state_c.filters.lock().clone();
            let sort_key = state_c.get_sort();
            let policy_bg = policy_c.clone();
            let now = Utc::now();

            let no_query = search_text.trim().is_empty();
            let default_sort = sort_key == crate::core::sort::SortKey::TimeNewest;

            if no_query && default_sort && !current_section.is_critical() {
                let snapshot = state_c.cache.snapshot_recent(LIVE_STREAM_SNAPSHOT_SIZE);

                let (filtered, result) = gio::spawn_blocking(move || {
                    let filtered = apply_filter(&snapshot, &filter);
                    let result = analyze(&filtered, &policy_bg, now);
                    (filtered, result)
                })
                .await
                .unwrap_or_else(|_| (Vec::new(), crate::diagnostics::DiagnosticResult::empty()));

                let objects: Vec<glib::BoxedAnyObject> = filtered
                    .iter()
                    .map(|log| glib::BoxedAnyObject::new(log.clone()))
                    .collect();

                store_c.splice(0, store_c.n_items(), &objects);

                status_c.set_title(&result.title);
                state_c.set_log_diagnostic_result(result.clone());
                status_c.set_description(Some(&result.description));
                stats_c.update(&result);
            } else {
                let logs = state_c.cache.snapshot_recent(LIVE_STREAM_SNAPSHOT_SIZE);
                let engine2 = state_c.search_engine.clone();
                let q = search_text.clone();

                let (final_logs, result) = gio::spawn_blocking(move || {
                    let mut results = if q.trim().is_empty() {
                        apply_filter(&logs, &filter)
                    } else {
                        let matched_ids =
                            engine2.search(&q, SEARCH_RESULT_LIMIT).unwrap_or_default();
                        let all_matched = filter_by_seq_ids(&matched_ids, &logs);
                        apply_filter(&all_matched, &filter)
                    };
                    apply_sort(&mut results, &sort_key);
                    let result = analyze(&results, &policy_bg, now);
                    (results, result)
                })
                .await
                .unwrap_or_else(|_| (Vec::new(), crate::diagnostics::DiagnosticResult::empty()));

                populate_log_store(&store_c, &final_logs);
                status_c.set_title(&result.title);

                state_c.set_log_diagnostic_result(result.clone());
                if current_section.is_critical() {
                    status_c.set_description(Some(&critical_focus_description(&result)));
                    stats_c.update_critical_focus(&result);
                } else {
                    status_c.set_description(Some(&result.description));
                    stats_c.update(&result);
                }
            }
        }
    });

    handle
}

fn setup_row_selection(
    column_view: &gtk::ColumnView,
    store: &gio::ListStore,
    detail_panel: Rc<DetailPanel>,
) {
    let store_c = store.clone();

    match column_view.model() {
        Some(model) => match model.downcast::<gtk::SingleSelection>() {
            Ok(selection) => {
                selection.connect_selection_changed(move |sel, _, _| {
                    let pos = sel.selected();
                    if pos == gtk::INVALID_LIST_POSITION {
                        detail_panel.clear();
                        return;
                    }
                    if let Some(obj) = store_c.item(pos) {
                        if let Ok(boxed) = obj.downcast::<glib::BoxedAnyObject>() {
                            let log = boxed.borrow::<LogEntry>();
                            detail_panel.update(&*log);
                        }
                    }
                });
            }
            Err(_) => {
                tracing::error!(
                    "setup_row_selection: column view model is not a SingleSelection; \
                     row selection will not function. Check GTK version compatibility."
                );
            }
        },
        None => {
            tracing::error!(
                "setup_row_selection: column view has no model; \
                 row selection will not function."
            );
        }
    }
}

fn setup_window_save(window: &adw::ApplicationWindow, state: &AppState) {
    let state_c = state.clone();
    window.connect_close_request(move |w| {
        let (width, height) = w.default_size();
        let mut cfg = state_c.config.lock();
        cfg.window_width = width;
        cfg.window_height = height;
        cfg.window_maximized = w.is_maximized();
        cfg.save();
        glib::Propagation::Proceed
    });
}

fn filter_by_seq_ids(
    matched_ids: &std::collections::HashSet<u64>,
    cache: &[LogEntry],
) -> Vec<LogEntry> {
    cache
        .iter()
        .filter(|e| matched_ids.contains(&e.seq_id))
        .cloned()
        .collect()
}
