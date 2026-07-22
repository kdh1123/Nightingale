use crate::domain::file_monitoring::{sha256_file, FileEventKind};
use rusqlite::{params, Connection, ErrorCode};
use serde::Serialize;
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
        Ok(Self {
            _connection: connection,
        })
    }
}

fn open_connection(path: &std::path::Path) -> Result<Connection, rusqlite::Error> {
    let connection = Connection::open(path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
    Ok(connection)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitoredPath {
    pub id: i64,
    pub path: String,
    pub enabled: bool,
    pub monitoring_status: String,
    pub baseline_status: String,
    pub last_scan_at: Option<String>,
    pub last_event_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEvent {
    pub id: i64,
    pub event_kind: String,
    pub file_path: String,
    pub severity: String,
    pub occurred_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEvent {
    pub id: i64,
    pub event_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub occurred_at: String,
    pub reviewed: bool,
}

pub fn record_file_event(
    database_path: &std::path::Path,
    monitored_path_id: i64,
    path: &std::path::Path,
    kind: FileEventKind,
) -> Result<i64, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let event_kind = match kind {
        FileEventKind::Created => "created",
        FileEventKind::Modified => "modified",
        FileEventKind::Deleted => "deleted",
        FileEventKind::Renamed => "renamed",
        FileEventKind::MetadataChanged => "metadata_changed",
        FileEventKind::Unknown => "unknown",
    };
    let (event_type, severity, title, description) =
        integrity_event_details(&connection, monitored_path_id, path, kind);
    connection.execute("INSERT INTO file_events (monitored_path_id, event_kind, file_path, severity) VALUES (?1, ?2, ?3, ?4)", params![monitored_path_id, event_kind, path.to_string_lossy(), severity]).map_err(|error| error.to_string())?;
    let id = connection.last_insert_rowid();
    connection.execute("INSERT INTO security_events (file_event_id, event_type, severity, title, description) VALUES (?1, ?2, ?3, ?4, ?5)", params![id, event_type, severity, title, description]).map_err(|error| error.to_string())?;
    connection.execute(
        "UPDATE monitored_paths SET last_event_at = CURRENT_TIMESTAMP, last_error = NULL WHERE id = ?1",
        params![monitored_path_id],
    ).map_err(|error| error.to_string())?;
    Ok(id)
}

fn integrity_event_details(
    connection: &Connection,
    monitored_path_id: i64,
    path: &std::path::Path,
    kind: FileEventKind,
) -> (&'static str, &'static str, &'static str, &'static str) {
    let expected_hash = connection
        .query_row(
            "SELECT sha256 FROM file_integrity_baselines WHERE monitored_path_id = ?1 AND file_path = ?2",
            params![monitored_path_id, path.to_string_lossy()],
            |row| row.get::<_, String>(0),
        )
        .ok();
    let differs_from_baseline = match kind {
        FileEventKind::Deleted => expected_hash.is_some(),
        FileEventKind::Created => expected_hash.is_none(),
        FileEventKind::Modified => expected_hash
            .is_some_and(|expected| sha256_file(path).is_ok_and(|actual| actual != expected)),
        _ => false,
    };
    if differs_from_baseline {
        (
            "integrity_changed",
            "medium",
            "무결성 기준선과 다른 파일 활동",
            "선택한 감시 폴더에서 기준선과 다른 파일 생성·변경·삭제가 감지되었습니다.",
        )
    } else {
        (
            "file_activity",
            "informational",
            "파일 활동 감지",
            "선택한 감시 폴더에서 파일 활동이 감지되었습니다.",
        )
    }
}
pub fn app_database_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|path| path.join("nightingale.sqlite3"))
        .map_err(|error| error.to_string())
}
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
pub fn upsert_file_baseline(
    database_path: &std::path::Path,
    monitored_path_id: i64,
    file_path: &std::path::Path,
    file_size: u64,
    modified_at: i64,
    sha256: &str,
) -> Result<(), String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO file_integrity_baselines (monitored_path_id, file_path, file_size, modified_at, sha256) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(monitored_path_id, file_path) DO UPDATE SET file_size = excluded.file_size, modified_at = excluded.modified_at, sha256 = excluded.sha256",
            params![monitored_path_id, file_path.to_string_lossy(), file_size as i64, modified_at, sha256],
        )
        .map_err(|error| error.to_string())?;
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
pub fn list_file_events(database_path: &std::path::Path) -> Result<Vec<FileEvent>, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let mut statement = connection.prepare("SELECT id, event_kind, file_path, severity, occurred_at FROM file_events ORDER BY occurred_at DESC LIMIT 100").map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(FileEvent {
                id: row.get(0)?,
                event_kind: row.get(1)?,
                file_path: row.get(2)?,
                severity: row.get(3)?,
                occurred_at: row.get(4)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}
pub fn list_security_events(database_path: &std::path::Path) -> Result<Vec<SecurityEvent>, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let mut statement = connection.prepare("SELECT id, event_type, severity, title, description, occurred_at, reviewed FROM security_events ORDER BY occurred_at DESC LIMIT 100").map_err(|error| error.to_string())?;
    let result = statement
        .query_map([], |row| {
            Ok(SecurityEvent {
                id: row.get(0)?,
                event_type: row.get(1)?,
                severity: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                occurred_at: row.get(5)?,
                reviewed: row.get::<_, i64>(6)? != 0,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string());
    result
}
pub fn mark_security_event_reviewed(
    database_path: &std::path::Path,
    id: i64,
) -> Result<(), String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection
        .execute(
            "UPDATE security_events SET reviewed = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creates_minimum_schema_in_memory() {
        let connection = Connection::open_in_memory().expect("in-memory database");
        connection
            .execute_batch(include_str!("../../migrations/0001_initial.sql"))
            .expect("migration");
        let count: i64 = connection.query_row("SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'security_policies'", [], |row| row.get(0)).expect("schema query");
        assert_eq!(count, 1);
    }
    #[test]
    fn applies_file_monitoring_migration_only_once() {
        let path = std::env::temp_dir().join(format!(
            "nightingale-migration-{}.sqlite3",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        Database::open(path.clone()).expect("first open");
        Database::open(path.clone()).expect("second open");
        std::fs::remove_file(path).expect("cleanup");
    }
    #[test]
    fn raises_severity_when_a_baselined_file_changes() {
        let connection = Connection::open_in_memory().expect("in-memory database");
        connection
            .execute_batch(include_str!("../../migrations/0001_initial.sql"))
            .expect("base migration");
        connection
            .execute_batch(include_str!("../../migrations/0002_file_monitoring.sql"))
            .expect("monitoring migration");
        connection
            .execute("INSERT INTO monitored_paths (path, normalized_path) VALUES ('/tmp/watch', '/tmp/watch')", [])
            .expect("monitored path");
        let file =
            std::env::temp_dir().join(format!("nightingale-baseline-{}.txt", std::process::id()));
        std::fs::write(&file, b"before").expect("fixture");
        let hash = sha256_file(&file).expect("initial hash");
        connection.execute("INSERT INTO file_integrity_baselines (monitored_path_id, file_path, file_size, modified_at, sha256) VALUES (1, ?1, 6, 0, ?2)", params![file.to_string_lossy(), hash]).expect("baseline");
        std::fs::write(&file, b"after").expect("modified fixture");
        let (_, severity, _, _) =
            integrity_event_details(&connection, 1, &file, FileEventKind::Modified);
        assert_eq!(severity, "medium");
        std::fs::remove_file(file).expect("cleanup");
    }
}
