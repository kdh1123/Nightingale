use crate::domain::{
    file_monitoring::{sha256_file, FileEventKind},
    policy::SecurityPolicy,
    security_management::ApplicationSettings,
    threat_detection::{Severity, ThreatAssessment, CORRELATION_WINDOW_SECONDS},
};
use rusqlite::{params, Connection, ErrorCode, OptionalExtension};
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

fn open_connection(path: &std::path::Path) -> Result<Connection, rusqlite::Error> {
    let connection = Connection::open(path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000; PRAGMA synchronous = NORMAL;")?;
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

#[derive(Clone, Debug, Serialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Incident {
    pub id: i64,
    pub severity: String,
    pub status: String,
    pub title: String,
    pub description: String,
    pub event_count: i64,
    pub first_detected_at: String,
    pub last_detected_at: String,
}

/// An explainable signal that contributed to an incident. File data is joined
/// here so the UI can show the evidence without guessing from timestamps.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncidentTimelineEvent {
    pub security_event_id: i64,
    pub event_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub occurred_at: String,
    pub reviewed: bool,
    pub file_event_id: Option<i64>,
    pub file_event_kind: Option<String>,
    pub file_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowlistEntry {
    pub id: i64,
    pub entry_type: String,
    pub value: String,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowlistAuditEntry {
    pub id: i64,
    pub entry_id: Option<i64>,
    pub action: String,
    pub entry_type: String,
    pub value: String,
    pub occurred_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityScore {
    pub score: i64,
    pub open_incident_count: i64,
    pub critical_incident_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: i64,
    pub incident_id: Option<i64>,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub read: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityReport {
    pub generated_at: String,
    pub security_score: SecurityScore,
    pub total_incidents: i64,
    pub severity_counts: SeverityCounts,
    pub monitored_folder_count: i64,
    pub file_event_count: i64,
    pub recent_detections: Vec<SecurityEvent>,
    pub recent_risk_events: Vec<SecurityEvent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeverityCounts {
    pub info: i64,
    pub low: i64,
    pub medium: i64,
    pub high: i64,
    pub critical: i64,
}

/// Values needed by the detector, loaded through one read connection per event.
pub struct FileEventAnalysisContext {
    pub threat_detection_enabled: bool,
    pub allowlisted: bool,
    pub policy: SecurityPolicy,
    pub differs_from_baseline: bool,
    pub recent_changes: i64,
}

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

fn detection_policy_from_connection(connection: &Connection) -> Result<SecurityPolicy, String> {
    let raw = connection
        .query_row(
            "SELECT policy_json FROM security_policies ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    raw.map(|json| serde_json::from_str(&json).map_err(|error| error.to_string()))
        .transpose()
        .map(|policy| policy.unwrap_or_default())
}

fn allowlist_matches(connection: &Connection, file_path: &str) -> Result<bool, String> {
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM allowlist_entries WHERE (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP) AND ((entry_type = 'path' AND (?1 = value OR ?1 LIKE value || '/%')) OR (entry_type = 'extension' AND value = ?2)))",
            params![file_path, extension],
            |row| row.get::<_, i64>(0),
        )
        .map(|result| result != 0)
        .map_err(|error| error.to_string())
}

pub fn detection_policy(database_path: &std::path::Path) -> Result<SecurityPolicy, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    detection_policy_from_connection(&connection)
}

pub fn save_detection_policy(
    database_path: &std::path::Path,
    mut policy: SecurityPolicy,
) -> Result<SecurityPolicy, String> {
    policy.validate().map_err(str::to_string)?;
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let next_version = connection
        .query_row(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM security_policies",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    policy.version = next_version;
    connection
        .execute(
            "INSERT INTO security_policies (version, policy_json) VALUES (?1, ?2)",
            params![
                policy.version,
                serde_json::to_string(&policy).map_err(|error| error.to_string())?
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(policy)
}

pub fn list_allowlist_entries(
    database_path: &std::path::Path,
) -> Result<Vec<AllowlistEntry>, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare("SELECT id, entry_type, value, expires_at, created_at FROM allowlist_entries ORDER BY created_at DESC, id DESC")
        .map_err(|error| error.to_string())?;
    let entries = statement
        .query_map([], |row| {
            Ok(AllowlistEntry {
                id: row.get(0)?,
                entry_type: row.get(1)?,
                value: row.get(2)?,
                expires_at: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(entries)
}

pub fn add_allowlist_entry(
    database_path: &std::path::Path,
    entry_type: &str,
    value: &str,
    expires_in_days: Option<i64>,
) -> Result<AllowlistEntry, String> {
    if !["path", "extension"].contains(&entry_type) || value.trim().is_empty() {
        return Err("신뢰 항목 형식이 올바르지 않습니다.".to_string());
    }
    if expires_in_days.is_some_and(|days| !(1..=3650).contains(&days)) {
        return Err("만료 기간은 1~3650일이어야 합니다.".to_string());
    }
    let value = if entry_type == "extension" {
        value.trim().trim_start_matches('.').to_ascii_lowercase()
    } else {
        value.trim().trim_end_matches('/').to_string()
    };
    if value.is_empty() {
        return Err("신뢰 항목 값이 비어 있습니다.".to_string());
    }
    let mut connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    transaction.execute("INSERT INTO allowlist_entries (entry_type, value, expires_at) VALUES (?1, ?2, CASE WHEN ?3 IS NULL THEN NULL ELSE datetime('now', '+' || ?3 || ' days') END)", params![entry_type, value, expires_in_days]).map_err(|error| error.to_string())?;
    let id = transaction.last_insert_rowid();
    transaction.execute("INSERT INTO allowlist_audit (entry_id, action, entry_type, value) VALUES (?1, 'added', ?2, ?3)", params![id, entry_type, value]).map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    let entries = list_allowlist_entries(database_path)?;
    entries
        .into_iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| "신뢰 항목을 찾을 수 없습니다.".to_string())
}

pub fn remove_allowlist_entry(database_path: &std::path::Path, id: i64) -> Result<(), String> {
    let mut connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let (entry_type, value): (String, String) = transaction
        .query_row(
            "SELECT entry_type, value FROM allowlist_entries WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute("DELETE FROM allowlist_entries WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    transaction.execute("INSERT INTO allowlist_audit (entry_id, action, entry_type, value) VALUES (?1, 'removed', ?2, ?3)", params![id, entry_type, value]).map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())
}

pub fn list_allowlist_audit(
    database_path: &std::path::Path,
) -> Result<Vec<AllowlistAuditEntry>, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let mut statement = connection.prepare("SELECT id, entry_id, action, entry_type, value, occurred_at FROM allowlist_audit ORDER BY occurred_at DESC, id DESC LIMIT 50").map_err(|error| error.to_string())?;
    let entries = statement
        .query_map([], |row| {
            Ok(AllowlistAuditEntry {
                id: row.get(0)?,
                entry_id: row.get(1)?,
                action: row.get(2)?,
                entry_type: row.get(3)?,
                value: row.get(4)?,
                occurred_at: row.get(5)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(entries)
}

pub fn persist_threat_assessment(
    database_path: &std::path::Path,
    file_event_id: i64,
    assessment: &ThreatAssessment,
) -> Result<(), String> {
    let mut connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "UPDATE file_events SET severity = ?1 WHERE id = ?2",
            params![assessment.severity.as_str(), file_event_id],
        )
        .map_err(|error| error.to_string())?;
    transaction.execute("INSERT INTO security_events (file_event_id, event_type, severity, title, description) VALUES (?1, ?2, ?3, ?4, ?5)", params![file_event_id, assessment.event_type, assessment.severity.as_str(), assessment.title, assessment.description]).map_err(|error| error.to_string())?;
    let security_event_id = transaction.last_insert_rowid();
    let incident_id = transaction.query_row(
        "SELECT id FROM incidents WHERE correlation_key = ?1 AND status != 'resolved' AND last_detected_at >= datetime('now', ?2) ORDER BY last_detected_at DESC LIMIT 1",
        params![assessment.correlation_key, format!("-{} seconds", CORRELATION_WINDOW_SECONDS)], |row| row.get::<_, i64>(0)
    ).ok();
    let incident_id = if let Some(id) = incident_id {
        transaction.execute("UPDATE incidents SET severity = CASE WHEN severity IN ('info','low','medium') AND ?1 IN ('high','critical') THEN ?1 ELSE severity END, event_count = event_count + 1, last_detected_at = CURRENT_TIMESTAMP WHERE id = ?2", params![assessment.severity.as_str(), id]).map_err(|error| error.to_string())?;
        id
    } else {
        transaction.execute("INSERT INTO incidents (correlation_key, severity, title, description) VALUES (?1, ?2, ?3, ?4)", params![assessment.correlation_key, assessment.severity.as_str(), assessment.title, assessment.description]).map_err(|error| error.to_string())?;
        transaction.last_insert_rowid()
    };
    transaction
        .execute(
            "INSERT INTO incident_events (incident_id, security_event_id) VALUES (?1, ?2)",
            params![incident_id, security_event_id],
        )
        .map_err(|error| error.to_string())?;
    if assessment.severity != Severity::Info {
        transaction.execute("INSERT INTO notifications (incident_id, severity, title, message) VALUES (?1, ?2, ?3, ?4)", params![incident_id, assessment.severity.as_str(), assessment.title, assessment.description]).map_err(|error| error.to_string())?;
    }
    transaction.commit().map_err(|error| error.to_string())
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

pub fn list_incidents(
    database_path: &std::path::Path,
    severity: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<Incident>, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let mut statement = connection.prepare("SELECT id, severity, status, title, description, event_count, first_detected_at, last_detected_at FROM incidents WHERE (?1 IS NULL OR severity = ?1) AND (?2 IS NULL OR status = ?2) ORDER BY last_detected_at DESC LIMIT 100").map_err(|error| error.to_string())?;
    let incidents = statement
        .query_map(params![severity, status], |row| {
            Ok(Incident {
                id: row.get(0)?,
                severity: row.get(1)?,
                status: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                event_count: row.get(5)?,
                first_detected_at: row.get(6)?,
                last_detected_at: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(incidents)
}

pub fn update_incident_status(
    database_path: &std::path::Path,
    id: i64,
    status: &str,
) -> Result<(), String> {
    if !["open", "investigating", "resolved"].contains(&status) {
        return Err("지원하지 않는 Incident 상태입니다.".to_string());
    }
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection.execute("UPDATE incidents SET status = ?1, resolved_at = CASE WHEN ?1 = 'resolved' THEN CURRENT_TIMESTAMP ELSE NULL END WHERE id = ?2", params![status, id]).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn incident_timeline(
    database_path: &std::path::Path,
    incident_id: i64,
) -> Result<Vec<IncidentTimelineEvent>, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT se.id, se.event_type, se.severity, se.title, se.description, se.occurred_at, se.reviewed, fe.id, fe.event_kind, fe.file_path FROM incident_events ie JOIN security_events se ON se.id = ie.security_event_id LEFT JOIN file_events fe ON fe.id = se.file_event_id WHERE ie.incident_id = ?1 ORDER BY se.occurred_at ASC, se.id ASC",
        )
        .map_err(|error| error.to_string())?;
    let events = statement
        .query_map(params![incident_id], |row| {
            Ok(IncidentTimelineEvent {
                security_event_id: row.get(0)?,
                event_type: row.get(1)?,
                severity: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                occurred_at: row.get(5)?,
                reviewed: row.get::<_, i64>(6)? != 0,
                file_event_id: row.get(7)?,
                file_event_kind: row.get(8)?,
                file_path: row.get(9)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(events)
}

pub fn security_score(database_path: &std::path::Path) -> Result<SecurityScore, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let mut statement = connection.prepare("SELECT severity FROM incidents WHERE status != 'resolved' AND last_detected_at >= datetime('now', '-7 days')").map_err(|error| error.to_string())?;
    let severities = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let penalty: i64 = severities
        .iter()
        .map(|value| match value.as_str() {
            "low" => Severity::Low.score_penalty(),
            "medium" => Severity::Medium.score_penalty(),
            "high" => Severity::High.score_penalty(),
            "critical" => Severity::Critical.score_penalty(),
            _ => Severity::Info.score_penalty(),
        })
        .sum();
    Ok(SecurityScore {
        score: (100 - penalty).max(0),
        open_incident_count: severities.len() as i64,
        critical_incident_count: severities
            .iter()
            .filter(|value| value.as_str() == "critical")
            .count() as i64,
    })
}

pub fn application_settings(
    database_path: &std::path::Path,
) -> Result<ApplicationSettings, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection.query_row("SELECT monitoring_enabled, threat_detection_enabled, auto_baseline_refresh, security_score_enabled, log_retention_days, ui_theme FROM application_settings WHERE id = 1", [], |row| Ok(ApplicationSettings { monitoring_enabled: row.get::<_, i64>(0)? != 0, threat_detection_enabled: row.get::<_, i64>(1)? != 0, auto_baseline_refresh: row.get::<_, i64>(2)? != 0, security_score_enabled: row.get::<_, i64>(3)? != 0, log_retention_days: row.get(4)?, ui_theme: row.get(5)? })).map_err(|error| error.to_string())
}

pub fn save_application_settings(
    database_path: &std::path::Path,
    settings: &ApplicationSettings,
) -> Result<(), String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection.execute("INSERT INTO application_settings (id, monitoring_enabled, threat_detection_enabled, auto_baseline_refresh, security_score_enabled, log_retention_days, ui_theme, updated_at) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP) ON CONFLICT(id) DO UPDATE SET monitoring_enabled=excluded.monitoring_enabled, threat_detection_enabled=excluded.threat_detection_enabled, auto_baseline_refresh=excluded.auto_baseline_refresh, security_score_enabled=excluded.security_score_enabled, log_retention_days=excluded.log_retention_days, ui_theme=excluded.ui_theme, updated_at=CURRENT_TIMESTAMP", params![settings.monitoring_enabled, settings.threat_detection_enabled, settings.auto_baseline_refresh, settings.security_score_enabled, settings.log_retention_days, settings.ui_theme]).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_notifications(database_path: &std::path::Path) -> Result<Vec<Notification>, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let mut statement = connection.prepare("SELECT id, incident_id, severity, title, message, read, created_at FROM notifications ORDER BY created_at DESC LIMIT 100").map_err(|error| error.to_string())?;
    let notifications = statement
        .query_map([], |row| {
            Ok(Notification {
                id: row.get(0)?,
                incident_id: row.get(1)?,
                severity: row.get(2)?,
                title: row.get(3)?,
                message: row.get(4)?,
                read: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(notifications)
}

pub fn mark_notification_read(database_path: &std::path::Path, id: i64) -> Result<(), String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection
        .execute(
            "UPDATE notifications SET read = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
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

pub fn cleanup_logs(database_path: &std::path::Path, retention_days: i64) -> Result<usize, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let age = format!("-{} days", retention_days);
    let deleted = connection
        .execute(
            "DELETE FROM notifications WHERE created_at < datetime('now', ?1)",
            params![age],
        )
        .map_err(|error| error.to_string())?;
    connection
        .execute(
            "DELETE FROM security_events WHERE occurred_at < datetime('now', ?1)",
            params![age],
        )
        .map_err(|error| error.to_string())?;
    connection
        .execute(
            "DELETE FROM file_events WHERE occurred_at < datetime('now', ?1)",
            params![age],
        )
        .map_err(|error| error.to_string())?;
    Ok(deleted)
}

pub fn security_report(database_path: &std::path::Path) -> Result<SecurityReport, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let count = |sql: &str| {
        connection
            .query_row(sql, [], |row| row.get::<_, i64>(0))
            .map_err(|error| error.to_string())
    };
    let severity_counts = SeverityCounts {
        info: count("SELECT count(*) FROM security_events WHERE severity = 'info'")?,
        low: count("SELECT count(*) FROM security_events WHERE severity = 'low'")?,
        medium: count("SELECT count(*) FROM security_events WHERE severity = 'medium'")?,
        high: count("SELECT count(*) FROM security_events WHERE severity = 'high'")?,
        critical: count("SELECT count(*) FROM security_events WHERE severity = 'critical'")?,
    };
    let recent_detections = list_security_events(database_path)?;
    let recent_risk_events = recent_detections
        .iter()
        .filter(|event| ["medium", "high", "critical"].contains(&event.severity.as_str()))
        .take(10)
        .cloned()
        .collect();
    let generated_at = connection
        .query_row("SELECT CURRENT_TIMESTAMP", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    let report = SecurityReport {
        generated_at,
        security_score: security_score(database_path)?,
        total_incidents: count("SELECT count(*) FROM incidents")?,
        severity_counts,
        monitored_folder_count: count("SELECT count(*) FROM monitored_paths WHERE enabled = 1")?,
        file_event_count: count("SELECT count(*) FROM file_events")?,
        recent_detections,
        recent_risk_events,
    };
    Ok(report)
}

pub fn save_security_report(database_path: &std::path::Path) -> Result<SecurityReport, String> {
    let report = security_report(database_path)?;
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO report_history (report_json) VALUES (?1)",
            params![serde_json::to_string(&report).map_err(|error| error.to_string())?],
        )
        .map_err(|error| error.to_string())?;
    Ok(report)
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
    fn correlates_events_and_calculates_security_score() {
        let path = std::env::temp_dir().join(format!(
            "nightingale-threat-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .elapsed()
                .map_or(0, |duration| duration.as_nanos())
        ));
        Database::open(path.clone()).expect("database");
        let monitored = add_monitored_path(&path, std::path::Path::new("/tmp/nightingale-threat"))
            .expect("monitored path");
        let assessment = ThreatAssessment {
            event_type: "mass_file_change",
            severity: Severity::High,
            title: "대량 파일 변경 감지",
            description: "test",
            correlation_key: "mass_file_change:/tmp".to_string(),
        };
        for _ in 0..2 {
            let event = record_file_event(
                &path,
                monitored,
                std::path::Path::new("/tmp/nightingale-threat/a.txt"),
                FileEventKind::Modified,
            )
            .expect("file event");
            persist_threat_assessment(&path, event, &assessment).expect("assessment");
        }
        let incidents = list_incidents(&path, Some("high"), Some("open")).expect("incidents");
        assert_eq!(incidents.len(), 1);
        assert_eq!(incidents[0].event_count, 2);
        let timeline = incident_timeline(&path, incidents[0].id).expect("timeline");
        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline[0].event_type, "mass_file_change");
        assert_eq!(
            timeline[0].file_path.as_deref(),
            Some("/tmp/nightingale-threat/a.txt")
        );
        assert_eq!(security_score(&path).expect("score").score, 75);
        std::fs::remove_file(path).expect("cleanup");
    }

    #[test]
    fn allowlist_is_audited_and_suppresses_matching_file_analysis() {
        let path = std::env::temp_dir().join(format!(
            "nightingale-allowlist-{}-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos(),
            "test"
        ));
        Database::open(path.clone()).expect("database");
        let monitored = add_monitored_path(&path, std::path::Path::new("/tmp/nightingale-allow"))
            .expect("monitored path");
        let entry =
            add_allowlist_entry(&path, "extension", ".log", Some(30)).expect("allowlist entry");
        assert_eq!(entry.value, "log");
        let context = file_event_analysis_context(
            &path,
            monitored,
            std::path::Path::new("/tmp/nightingale-allow/build.log"),
            FileEventKind::Created,
        )
        .expect("analysis context");
        assert!(context.allowlisted);
        remove_allowlist_entry(&path, entry.id).expect("remove entry");
        assert_eq!(list_allowlist_audit(&path).expect("audit").len(), 2);
        let mut policy = detection_policy(&path).expect("default policy");
        policy.sensitivity = crate::domain::policy::Sensitivity::High;
        let saved = save_detection_policy(&path, policy).expect("saved policy");
        assert_eq!(saved.sensitivity, crate::domain::policy::Sensitivity::High);
        std::fs::remove_file(path).expect("cleanup");
    }
}
