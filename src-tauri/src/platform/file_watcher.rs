use crate::domain::file_monitoring::FileEventKind;
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind},
    EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{path::PathBuf, sync::mpsc::SyncSender};

pub struct WatchHandle {
    _watcher: RecommendedWatcher,
}
pub fn start(
    path: PathBuf,
    sender: SyncSender<(PathBuf, FileEventKind)>,
) -> Result<WatchHandle, String> {
    let mut watcher =
        notify::recommended_watcher(move |result: notify::Result<notify::Event>| match result {
            Ok(event) => {
                let kind = match event.kind {
                    EventKind::Create(
                        CreateKind::Any | CreateKind::File | CreateKind::Folder | CreateKind::Other,
                    ) => FileEventKind::Created,
                    EventKind::Remove(
                        RemoveKind::Any | RemoveKind::File | RemoveKind::Folder | RemoveKind::Other,
                    ) => FileEventKind::Deleted,
                    EventKind::Modify(ModifyKind::Name(_)) => FileEventKind::Renamed,
                    EventKind::Modify(ModifyKind::Metadata(_)) => FileEventKind::MetadataChanged,
                    EventKind::Modify(_) => FileEventKind::Modified,
                    _ => FileEventKind::Unknown,
                };
                for event_path in event.paths {
                    let _ = sender.try_send((event_path, kind));
                }
            }
            Err(_) => tracing::warn!("file watcher reported an operating system error"),
        })
        .map_err(|error| error.to_string())?;
    watcher
        .watch(&path, RecursiveMode::Recursive)
        .map_err(|error| error.to_string())?;
    Ok(WatchHandle { _watcher: watcher })
}
