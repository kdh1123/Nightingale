//! Row shapes returned across the repository boundary. They are serialized straight to the
//! UI, so the field names here are part of the Tauri command contract.

use crate::domain::policy::SecurityPolicy;
use serde::Serialize;

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
