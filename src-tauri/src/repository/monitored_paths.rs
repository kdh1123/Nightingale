//! The folders the user asked Nightingale to watch, plus their watcher and baseline state.

use super::models::MonitoredPath;
use super::open_connection;
use rusqlite::{params, ErrorCode};
use std::path::PathBuf;

pub fn add_monitored_path(
    database_path: &std::path::Path,
    path: &std::path::Path,
) -> Result<i64, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO monitored_paths (path, normalized_path) VALUES (?1, ?2)",
            params![path.to_string_lossy(), path.to_string_lossy()],
        )
        .map_err(|error| match error {
            rusqlite::Error::SqliteFailure(details, _)
                if details.code == ErrorCode::ConstraintViolation =>
            {
                "이미 감시 중인 폴더입니다.".to_string()
            }
            other => other.to_string(),
        })?;
    Ok(connection.last_insert_rowid())
}

pub fn list_monitored_paths(database_path: &std::path::Path) -> Result<Vec<MonitoredPath>, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare("SELECT id, path, enabled, monitoring_status, baseline_status, last_scan_at, last_event_at, last_error FROM monitored_paths ORDER BY created_at DESC")
        .map_err(|error| error.to_string())?;
    let result = statement
        .query_map([], |row| {
            Ok(MonitoredPath {
                id: row.get(0)?,
                path: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
                monitoring_status: row.get(3)?,
                baseline_status: row.get(4)?,
                last_scan_at: row.get(5)?,
                last_event_at: row.get(6)?,
                last_error: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<MonitoredPath>, _>>()
        .map_err(|error| error.to_string());
    result
}

pub fn remove_monitored_path(database_path: &std::path::Path, id: i64) -> Result<(), String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection
        .execute("DELETE FROM monitored_paths WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn set_monitoring_status(
    database_path: &std::path::Path,
    id: i64,
    status: &str,
) -> Result<(), String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection
        .execute(
            "UPDATE monitored_paths SET monitoring_status = ?1, enabled = ?2 WHERE id = ?3",
            params![status, status == "running", id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn set_monitoring_error(
    database_path: &std::path::Path,
    id: i64,
    error: &str,
) -> Result<(), String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection
        .execute(
            "UPDATE monitored_paths SET monitoring_status = 'failed', last_error = ?1 WHERE id = ?2",
            params![error, id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn set_baseline_status(
    database_path: &std::path::Path,
    id: i64,
    status: &str,
    error: Option<&str>,
) -> Result<(), String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection.execute(
        "UPDATE monitored_paths SET baseline_status = ?1, last_scan_at = CASE WHEN ?1 = 'complete' THEN CURRENT_TIMESTAMP ELSE last_scan_at END, last_error = ?2 WHERE id = ?3",
        params![status, error, id],
    ).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn enabled_monitored_paths(
    database_path: &std::path::Path,
) -> Result<Vec<MonitoredPath>, String> {
    Ok(list_monitored_paths(database_path)?
        .into_iter()
        .filter(|path| path.enabled)
        .collect())
}

pub fn monitored_path(database_path: &std::path::Path, id: i64) -> Result<PathBuf, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection
        .query_row(
            "SELECT path FROM monitored_paths WHERE id = ?1",
            params![id],
            |row| row.get::<_, String>(0),
        )
        .map(PathBuf::from)
        .map_err(|error| error.to_string())
}
