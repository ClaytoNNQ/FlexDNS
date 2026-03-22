use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind, Event, Error};
use tokio::sync::watch;
use std::path::PathBuf;
use anyhow::Result;

pub fn start_config_watcher(path: PathBuf, tx: watch::Sender<()>) -> Result<RecommendedWatcher> {
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res: Result<Event, Error>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        let _ = tx.send(());
                    }
                    _ => {}
                }
            }
        },
        notify::Config::default(),
    )?;

    watcher.watch(&path, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}