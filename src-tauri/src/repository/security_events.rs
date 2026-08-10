//! Detector output: individual security events, the incidents they correlate into, and the
//! score derived from the still-open incidents.

use super::models::{Incident, IncidentTimelineEvent, SecurityEvent, SecurityScore};
use super::open_connection;
use crate::domain::threat_detection::{Severity, ThreatAssessment, CORRELATION_WINDOW_SECONDS};
use rusqlite::{params, Connection};

/// Writes one assessment and folds it into an open incident when a matching correlation key
/// was seen inside the correlation window; otherwise a new incident is opened.
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

pub fn list_security_events(database_path: &std::path::Path) -> Result<Vec<SecurityEvent>, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    list_security_events_from_connection(&connection)
}

/// Shared with the report builder so one report does not open several connections.
pub(super) fn list_security_events_from_connection(
    connection: &Connection,
) -> Result<Vec<SecurityEvent>, String> {
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
    security_score_from_connection(&connection)
}

pub(super) fn security_score_from_connection(
    connection: &Connection,
) -> Result<SecurityScore, String> {
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
