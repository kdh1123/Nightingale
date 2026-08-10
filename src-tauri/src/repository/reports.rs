//! The exportable security report, assembled from the other repository modules.

use super::models::{SecurityReport, SeverityCounts};
use super::open_connection;
use super::security_events::{
    list_security_events_from_connection, security_score_from_connection,
};
use rusqlite::{params, Connection};

pub fn security_report(database_path: &std::path::Path) -> Result<SecurityReport, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    security_report_from_connection(&connection)
}

/// Building a report needs a dozen queries, so it takes a connection instead of a path and
/// every caller reuses the one it already has.
fn security_report_from_connection(connection: &Connection) -> Result<SecurityReport, String> {
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
    let recent_detections = list_security_events_from_connection(connection)?;
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
        security_score: security_score_from_connection(connection)?,
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
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let report = security_report_from_connection(&connection)?;
    connection
        .execute(
            "INSERT INTO report_history (report_json) VALUES (?1)",
            params![serde_json::to_string(&report).map_err(|error| error.to_string())?],
        )
        .map_err(|error| error.to_string())?;
    Ok(report)
}
