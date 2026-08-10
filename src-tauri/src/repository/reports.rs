//! The exportable security report, assembled from the other repository modules.

use super::models::{SecurityReport, SeverityCounts};
use super::open_connection;
use super::security_events::{list_security_events, security_score};
use rusqlite::params;

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
