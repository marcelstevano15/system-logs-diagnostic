use crate::errors::AppResult;
use async_channel::Sender;
use ignore::WalkBuilder;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{debug, error, info, warn};

pub enum WatchEvent {
    LogFileChanged(PathBuf),
    LogFileCreated(PathBuf),
    LogFileRemoved(PathBuf),
}

pub struct LogDirectoryWatcher {
    watcher: Arc<Mutex<RecommendedWatcher>>,
    watched_paths: Arc<Mutex<Vec<PathBuf>>>,
}

impl LogDirectoryWatcher {
    pub fn new(sender: Sender<WatchEvent>) -> AppResult<Self> {
        let sender_clone = sender.clone();

        let watcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
            match result {
                Ok(event) => {
                    for path in &event.paths {
                        let watch_event = match event.kind {
                            EventKind::Modify(_) => {
                                debug!("Log file modified: {:?}", path);
                                Some(WatchEvent::LogFileChanged(path.clone()))
                            }
                            EventKind::Create(_) => {
                                info!("Log file created: {:?}", path);
                                Some(WatchEvent::LogFileCreated(path.clone()))
                            }
                            EventKind::Remove(_) => {
                                warn!("Log file removed: {:?}", path);
                                Some(WatchEvent::LogFileRemoved(path.clone()))
                            }
                            _ => None,
                        };

                        if let Some(ev) = watch_event {
                            if sender_clone.send_blocking(ev).is_err() {
                                error!("Watch event channel closed");
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Filesystem watch error: {:?}", e);
                }
            }
        })
        .map_err(|e| crate::errors::AppError::Watch(e.to_string()))?;

        Ok(Self {
            watcher: Arc::new(Mutex::new(watcher)),
            watched_paths: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn watch_directory(&self, path: &Path) -> AppResult<()> {
        let walker = WalkBuilder::new(path)
            .hidden(false)
            .ignore(false)
            .git_ignore(false)
            .build();

        let mut watcher = self.watcher.lock();
        let mut watched = self.watched_paths.lock();

        for entry in walker.flatten() {
            let p = entry.path().to_path_buf();
            if p.is_dir() {
                watcher
                    .watch(&p, RecursiveMode::NonRecursive)
                    .map_err(|e| crate::errors::AppError::Watch(e.to_string()))?;
                watched.push(p);
            }
        }

        info!("Watching {} directories under {:?}", watched.len(), path);
        Ok(())
    }

    pub fn unwatch_all(&self) {
        let mut watcher = self.watcher.lock();
        let mut watched = self.watched_paths.lock();
        for path in watched.drain(..) {
            let _ = watcher.unwatch(&path);
        }
    }
}

