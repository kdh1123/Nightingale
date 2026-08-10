//! Application preferences, the local notification inbox, and the retention sweep that
//! enforces the configured log retention.

use super::models::Notification;
use super::open_connection;
use crate::domain::security_management::ApplicationSettings;
use rusqlite::params;

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

/// Returns the number of deleted notifications; events are swept with the same cutoff.
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
