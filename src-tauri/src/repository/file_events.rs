//! Raw file activity: the watcher write path, the baseline writer, and the reads that
//! feed the detector and the activity timeline.

use super::allowlist::allowlist_matches;
use super::models::{FileEvent, FileEventAnalysisContext};
use super::open_connection;
use super::policy::detection_policy_from_connection;
use crate::domain::file_monitoring::{sha256_file, FileEventKind};
use rusqlite::{params, Connection};

/// A short-lived repository-owned connection used by one baseline scan.
/// Keeping it open avoids opening SQLite once per file while preserving the
/// application → repository boundary.
pub struct BaselineWriter {
    connection: Connection,
}

impl BaselineWriter {
    pub fn open(database_path: &std::path::Path) -> Result<Self, String> {
        Ok(Self {
            connection: open_connection(database_path).map_err(|error| error.to_string())?,
        })
    }

    pub fn upsert(
        &self,
        monitored_path_id: i64,
        file_path: &std::path::Path,
        file_size: u64,
        modified_at: i64,
        sha256: &str,
    ) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO file_integrity_baselines (monitored_path_id, file_path, file_size, modified_at, sha256) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(monitored_path_id, file_path) DO UPDATE SET file_size = excluded.file_size, modified_at = excluded.modified_at, sha256 = excluded.sha256",
                params![monitored_path_id, file_path.to_string_lossy(), file_size as i64, modified_at, sha256],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }
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
    connection.execute("INSERT INTO file_events (monitored_path_id, event_kind, file_path, severity) VALUES (?1, ?2, ?3, 'info')", params![monitored_path_id, event_kind, path.to_string_lossy()]).map_err(|error| error.to_string())?;
    let id = connection.last_insert_rowid();
    connection.execute(
        "UPDATE monitored_paths SET last_event_at = CURRENT_TIMESTAMP, last_error = NULL WHERE id = ?1",
        params![monitored_path_id],
    ).map_err(|error| error.to_string())?;
    Ok(id)
}

pub fn file_event_analysis_context(
    database_path: &std::path::Path,
    monitored_path_id: i64,
    path: &std::path::Path,
    kind: FileEventKind,
) -> Result<FileEventAnalysisContext, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let threat_detection_enabled = connection
        .query_row(
            "SELECT threat_detection_enabled FROM application_settings WHERE id = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| error.to_string())?
        != 0;
    let file_path = path.to_string_lossy();
    let allowlisted = allowlist_matches(&connection, &file_path)?;
    let policy = detection_policy_from_connection(&connection)?;
    let expected_hash = connection.query_row("SELECT sha256 FROM file_integrity_baselines WHERE monitored_path_id = ?1 AND file_path = ?2", params![monitored_path_id, file_path], |row| row.get::<_, String>(0)).ok();
    let differs_from_baseline = match kind {
        FileEventKind::Deleted => expected_hash.is_some(),
        FileEventKind::Created => expected_hash.is_none(),
        FileEventKind::Modified => expected_hash
            .is_some_and(|expected| sha256_file(path).is_ok_and(|actual| actual != expected)),
        _ => false,
    };
    let recent_changes = connection.query_row("SELECT count(*) FROM file_events WHERE monitored_path_id = ?1 AND occurred_at >= datetime('now', '-60 seconds')", params![monitored_path_id], |row| row.get(0)).map_err(|error| error.to_string())?;
    Ok(FileEventAnalysisContext {
        threat_detection_enabled,
        allowlisted,
        policy,
        differs_from_baseline,
        recent_changes,
    })
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

pub fn list_file_events_filtered(
    database_path: &std::path::Path,
    query: Option<&str>,
    severity: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
    sort_desc: bool,
) -> Result<Vec<FileEvent>, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let order = if sort_desc { "DESC" } else { "ASC" };
    let sql = format!("SELECT id, event_kind, file_path, severity, occurred_at FROM file_events WHERE (?1 IS NULL OR file_path LIKE '%' || ?1 || '%') AND (?2 IS NULL OR severity = ?2) AND (?3 IS NULL OR occurred_at >= ?3) AND (?4 IS NULL OR occurred_at <= ?4) ORDER BY occurred_at {order} LIMIT 500");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| error.to_string())?;
    let events = statement
        .query_map(params![query, severity, from, to], |row| {
            Ok(FileEvent {
                id: row.get(0)?,
                event_kind: row.get(1)?,
                file_path: row.get(2)?,
                severity: row.get(3)?,
                occurred_at: row.get(4)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(events)
}
