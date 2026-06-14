mod app;
mod config;
mod core;
mod diagnostics;
mod errors;
mod export;
mod journal;
mod models;
mod state;
mod ui;
mod utils;

use adw::prelude::*;
use app::build_ui;
use tracing_subscriber::FmtSubscriber;

const APP_ID: &str = "com.marcel.system-logs-diagnostic";

fn main() -> glib::ExitCode {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _tokio_guard = tokio_runtime.enter();

    adw::init().unwrap();

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();
    app.connect_activate(build_ui);

    let exit_code = app.run();

    drop(_tokio_guard);
    tokio_runtime.shutdown_background();

    exit_code
}


