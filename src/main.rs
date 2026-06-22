mod app;
mod config;
mod core;
mod diagnostics;
mod errors;
mod export;
mod journal;
mod models;
mod runtime;
mod state;
mod ui;
mod utils;

use adw::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

const APP_ID: &str = "com.marcel.system-logs-diagnostic";

fn main() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_names(true)
        .init();

    tracing::info!("Starting System Logs Diagnostic v{}", env!("CARGO_PKG_VERSION"));

    let rt = runtime::global_runtime();
    let _enter_guard = rt.enter();

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gtk::gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(|app| {
        app::window::create_application_window(app);
    });

    let exit_code = app.run();
    tracing::info!("Application exited with code: {:?}", exit_code);
    std::process::exit(exit_code.into());
}

