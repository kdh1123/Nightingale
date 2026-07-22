use crate::{
    domain::file_monitoring::{is_duplicate, NormalizedFileEvent},
    platform::file_watcher::{self, WatchHandle},
    repository,
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{mpsc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

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
        let (sender, receiver) = mpsc::sync_channel(256);
        let handle = file_watcher::start(path.clone(), sender)?;
        self.watchers
            .lock()
            .map_err(|_| "watcher state unavailable".to_string())?
            .insert(id, handle);
        thread::spawn(move || {
            let mut previous: Option<NormalizedFileEvent> = None;
            while let Ok((event_path, kind)) = receiver.recv() {
                let next = NormalizedFileEvent {
                    monitored_path: path.clone(),
                    path: event_path,
                    kind,
                    observed_at_ms: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_or(0, |time| time.as_millis()),
                };
                if previous
                    .as_ref()
                    .is_some_and(|old| is_duplicate(old, &next))
                {
                    continue;
                }
                if repository::record_file_event(&database, id, &next.path, next.kind).is_err() {
                    let _ = repository::set_monitoring_error(
                        &database,
                        id,
                        "파일 이벤트를 기록하지 못했습니다.",
                    );
                    tracing::warn!(watch_id = id, "file monitoring event could not be recorded");
                }
                previous = Some(next);
            }
        });
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{application::baseline, repository};
    use rusqlite::Connection;
    use std::{fs, time::Duration};

    fn wait_until(mut predicate: impl FnMut() -> bool) -> bool {
        for _ in 0..100 {
            if predicate() {
                return true;
            }
            thread::sleep(Duration::from_millis(100));
        }
        false
    }

    #[test]
    fn records_file_lifecycle_and_recovers_after_watcher_restart() {
        let root = std::env::temp_dir().join(format!(
            ".nightingale-integration-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let watch_path = root.join("watch");
        let database = root.join("nightingale.sqlite3");
        fs::create_dir_all(&watch_path).expect("test directory");
        let original = watch_path.join("baselined.txt");
        fs::write(&original, b"before").expect("baseline fixture");
        let connection = Connection::open(&database).expect("database");
        connection
            .execute_batch(include_str!("../../migrations/0001_initial.sql"))
            .expect("base migration");
        connection
            .execute_batch(include_str!("../../migrations/0002_file_monitoring.sql"))
            .expect("monitoring migration");
        drop(connection);

        let id = repository::add_monitored_path(&database, &watch_path).expect("monitored path");
        assert_eq!(
            baseline::scan(&database, id, &watch_path).expect("baseline scan"),
            1
        );
        assert_eq!(
            repository::list_monitored_paths(&database).expect("monitored paths")[0]
                .baseline_status,
            "complete"
        );

        let service = FileMonitoringService::default();
        service
            .start(id, watch_path.clone(), database.clone())
            .expect("start watcher");
        repository::set_monitoring_status(&database, id, "running").expect("running status");
        thread::sleep(Duration::from_millis(300));
        let created = watch_path.join("created.txt");
        fs::write(&created, b"created").expect("create file");
        assert!(wait_until(|| {
            repository::list_file_events(&database)
                .expect("file events")
                .iter()
                .any(|event| event.event_kind == "created")
        }));
        fs::write(&original, b"after").expect("modify file");
        assert!(wait_until(|| {
            repository::list_file_events(&database)
                .expect("file events")
                .iter()
                .any(|event| event.event_kind == "modified")
        }));
        fs::remove_file(&original).expect("delete file");
        assert!(wait_until(|| {
            let events = repository::list_file_events(&database).expect("file events");
            events.iter().any(|event| event.event_kind == "deleted")
        }));
        assert!(wait_until(|| {
            repository::list_security_events(&database)
                .expect("security events")
                .iter()
                .any(|event| event.event_type == "integrity_changed")
        }));

        let event = repository::list_security_events(&database)
            .expect("security events")
            .into_iter()
            .next()
            .expect("security event");
        repository::mark_security_event_reviewed(&database, event.id).expect("review event");
        assert!(repository::list_security_events(&database)
            .expect("security events")
            .iter()
            .any(|item| item.id == event.id && item.reviewed));

        service.stop(id).expect("pause watcher");
        repository::set_monitoring_status(&database, id, "paused").expect("paused status");
        thread::sleep(Duration::from_millis(700));
        let paused_count = repository::list_file_events(&database)
            .expect("paused events")
            .len();
        fs::write(&created, b"changed while paused").expect("paused write");
        thread::sleep(Duration::from_millis(700));
        assert_eq!(
            repository::list_file_events(&database)
                .expect("paused events")
                .len(),
            paused_count
        );

        service
            .start(id, watch_path.clone(), database.clone())
            .expect("resume watcher");
        repository::set_monitoring_status(&database, id, "running").expect("running status");
        fs::write(&created, b"changed after resume").expect("resume write");
        assert!(wait_until(|| {
            repository::list_file_events(&database)
                .expect("resumed events")
                .len()
                > paused_count
        }));
        service.stop(id).expect("stop watcher");

        let restored_path = repository::enabled_monitored_paths(&database)
            .expect("persisted enabled path")
            .into_iter()
            .next()
            .expect("enabled path");
        assert_eq!(restored_path.id, id);
        let restored_service = FileMonitoringService::default();
        restored_service
            .start(id, restored_path.path.into(), database.clone())
            .expect("restore watcher");
        fs::write(&created, b"changed after restart").expect("restart write");
        assert!(wait_until(|| {
            repository::list_file_events(&database)
                .expect("restored events")
                .len()
                > paused_count + 1
        }));
        restored_service.stop(id).expect("stop restored watcher");
        drop(restored_service);
        fs::remove_dir_all(&root).expect("cleanup");
        assert!(!root.exists());
    }
}
