//! Trusted paths and extensions. Every change is written to an audit table in the same
//! transaction so suppressed detections always have a traceable reason.

use super::models::{AllowlistAuditEntry, AllowlistEntry};
use super::open_connection;
use rusqlite::{params, Connection, OptionalExtension, Row};

const ENTRY_COLUMNS: &str = "id, entry_type, value, expires_at, created_at";

fn read_entry(row: &Row<'_>) -> rusqlite::Result<AllowlistEntry> {
    Ok(AllowlistEntry {
        id: row.get(0)?,
        entry_type: row.get(1)?,
        value: row.get(2)?,
        expires_at: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn find_entry(connection: &Connection, id: i64) -> Result<Option<AllowlistEntry>, String> {
    connection
        .query_row(
            &format!("SELECT {ENTRY_COLUMNS} FROM allowlist_entries WHERE id = ?1"),
            params![id],
            read_entry,
        )
        .optional()
        .map_err(|error| error.to_string())
}

pub(super) fn allowlist_matches(connection: &Connection, file_path: &str) -> Result<bool, String> {
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

pub fn list_allowlist_entries(
    database_path: &std::path::Path,
) -> Result<Vec<AllowlistEntry>, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare(&format!(
            "SELECT {ENTRY_COLUMNS} FROM allowlist_entries ORDER BY created_at DESC, id DESC"
        ))
        .map_err(|error| error.to_string())?;
    let entries = statement
        .query_map([], read_entry)
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
    find_entry(&connection, id)?.ok_or_else(|| "신뢰 항목을 찾을 수 없습니다.".to_string())
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
