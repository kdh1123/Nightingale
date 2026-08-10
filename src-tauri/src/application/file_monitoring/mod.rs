use crate::{
    application::threat_detection,
    domain::file_monitoring::{is_duplicate, FileEventKind, NormalizedFileEvent},
    platform::file_watcher::{self, WatchHandle},
    repository,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{mpsc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
mod tests;

/// Bounds how many watcher events may queue before the OS watcher thread blocks.
const EVENT_QUEUE_CAPACITY: usize = 256;
/// How long a path stays in the deduplication map. Keeps the map small during long runs.
const DEDUPLICATION_WINDOW_MS: u128 = 1_000;

pub struct FileMonitoringService {
    watchers: Mutex<HashMap<i64, WatchHandle>>,
}
impl Default for FileMonitoringService {
    fn default() -> Self {
        Self {
            watchers: Mutex::new(HashMap::new()),
        }
    }
}
impl FileMonitoringService {
    pub fn start(&self, id: i64, path: PathBuf, database: PathBuf) -> Result<(), String> {
        let (sender, receiver) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let handle = file_watcher::start(path.clone(), sender)?;
        self.watchers
            .lock()
            .map_err(|_| "watcher state unavailable".to_string())?
            .insert(id, handle);
        thread::spawn(move || consume_events(id, path, database, receiver));
        Ok(())
    }
    pub fn stop(&self, id: i64) -> Result<(), String> {
        self.watchers
            .lock()
            .map_err(|_| "watcher state unavailable".to_string())?
            .remove(&id);
        Ok(())
    }
    pub fn is_running(&self, id: i64) -> Result<bool, String> {
        Ok(self
            .watchers
            .lock()
            .map_err(|_| "watcher state unavailable".to_string())?
            .contains_key(&id))
    }
}

/// Owns one watch for the life of its thread: deduplicates the OS event stream and hands
/// what survives to the recorder. Returns when the watcher is dropped and the channel closes.
fn consume_events(
    id: i64,
    monitored_path: PathBuf,
    database: PathBuf,
    receiver: mpsc::Receiver<(PathBuf, FileEventKind)>,
) {
    // Deduplicate per path rather than only the immediately preceding event.
    let mut recent = HashMap::<PathBuf, NormalizedFileEvent>::new();
    while let Ok((event_path, kind)) = receiver.recv() {
        let next = NormalizedFileEvent {
            monitored_path: monitored_path.clone(),
            path: event_path,
            kind,
            observed_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |time| time.as_millis()),
        };
        if recent
            .get(&next.path)
            .is_some_and(|old| is_duplicate(old, &next))
        {
            continue;
        }
        recent.retain(|_, event| {
            next.observed_at_ms.saturating_sub(event.observed_at_ms) <= DEDUPLICATION_WINDOW_MS
        });
        record_and_analyze(&database, id, &next);
        recent.insert(next.path.clone(), next);
    }
}

/// A failure here must not stop the watch, so it is surfaced on the monitored path instead
/// of propagating out of the thread.
fn record_and_analyze(database: &Path, id: i64, event: &NormalizedFileEvent) {
    let analysis =
        repository::record_file_event(database, id, &event.path, event.kind).and_then(|event_id| {
            threat_detection::analyze_file_event(database, event_id, id, &event.path, event.kind)
        });
    if analysis.is_err() {
        let _ =
            repository::set_monitoring_error(database, id, "파일 이벤트를 기록하지 못했습니다.");
        tracing::warn!(watch_id = id, "file monitoring event could not be recorded");
    }
}
