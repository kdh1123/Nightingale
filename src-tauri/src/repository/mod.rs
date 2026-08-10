//! Local SQLite storage. Each submodule owns one group of tables; this module owns the
//! schema, connection setup, and the flat surface the rest of the crate calls through.

mod allowlist;
mod file_events;
mod models;
mod monitored_paths;
mod policy;
mod reports;
mod security_events;
mod settings;
#[cfg(test)]
mod tests;

pub use allowlist::{
    add_allowlist_entry, list_allowlist_audit, list_allowlist_entries, remove_allowlist_entry,
};
pub use file_events::{
    file_event_analysis_context, list_file_events, list_file_events_filtered, record_file_event,
    BaselineWriter,
};
pub use models::*;
pub use monitored_paths::{
    add_monitored_path, enabled_monitored_paths, list_monitored_paths, monitored_path,
    remove_monitored_path, set_baseline_status, set_monitoring_error, set_monitoring_status,
};
pub use policy::{detection_policy, save_detection_policy};
pub use reports::{save_security_report, security_report};
pub use security_events::{
    incident_timeline, list_incidents, list_security_events, mark_security_event_reviewed,
    persist_threat_assessment, security_score, update_incident_status,
};
pub use settings::{
    application_settings, cleanup_logs, list_notifications, mark_notification_read,
    save_application_settings,
};

use rusqlite::Connection;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub struct Database {
    _connection: Connection,
}
impl Database {
    pub fn open_for_app(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let directory = app.path().app_data_dir()?;
        std::fs::create_dir_all(&directory)?;
        Self::open(directory.join("nightingale.sqlite3"))
    }
    fn open(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let connection = open_connection(&path)?;
        connection.execute_batch(include_str!("../../migrations/0001_initial.sql"))?;
        let current_version: i64 = connection.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_metadata",
            [],
            |row| row.get(0),
        )?;
        if current_version < 2 {
            let already_upgraded = connection
                .prepare("PRAGMA table_info(monitored_paths)")?
                .query_map([], |row| row.get::<_, String>(1))?
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .any(|column| column == "normalized_path");
            if already_upgraded {
                connection.execute(
                    "INSERT OR IGNORE INTO schema_metadata (version) VALUES (2)",
                    [],
                )?;
            } else {
                connection
                    .execute_batch(include_str!("../../migrations/0002_file_monitoring.sql"))?;
            }
        }
        if current_version < 3 {
            connection.execute_batch(include_str!("../../migrations/0003_threat_detection.sql"))?;
        }
        if current_version < 4 {
            connection.execute_batch(include_str!(
                "../../migrations/0004_security_management.sql"
            ))?;
        }
        if current_version < 5 {
            connection.execute_batch(include_str!("../../migrations/0005_performance.sql"))?;
        }
        if current_version < 6 {
            connection.execute_batch(include_str!(
                "../../migrations/0006_allowlist_and_detection_policy.sql"
            ))?;
        }
        Ok(Self {
            _connection: connection,
        })
    }
}

/// Every repository call opens its own connection; the pragmas here are what make that
/// safe for a long-running app with a background watcher.
fn open_connection(path: &std::path::Path) -> Result<Connection, rusqlite::Error> {
    let connection = Connection::open(path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000; PRAGMA synchronous = NORMAL;")?;
    Ok(connection)
}

pub fn app_database_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|path| path.join("nightingale.sqlite3"))
        .map_err(|error| error.to_string())
}
