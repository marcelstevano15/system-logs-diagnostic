use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn global_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("sld-worker")
            .enable_all()
            .build()
            .expect("Failed to build global Tokio runtime")
    })
}

pub fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    global_runtime().block_on(fut)
}

