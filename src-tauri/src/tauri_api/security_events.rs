//! Detection output for the investigation workspace: security events, incidents, and score.

use crate::repository;

#[tauri::command]
pub fn list_security_events(
    app: tauri::AppHandle,
) -> Result<Vec<repository::SecurityEvent>, String> {
    repository::list_security_events(&repository::app_database_path(&app)?)
}

#[tauri::command]
pub fn mark_security_event_reviewed(id: i64, app: tauri::AppHandle) -> Result<(), String> {
    repository::mark_security_event_reviewed(&repository::app_database_path(&app)?, id)
}

#[tauri::command]
pub fn list_incidents(
    severity: Option<String>,
    status: Option<String>,
    app: tauri::AppHandle,
) -> Result<Vec<repository::Incident>, String> {
    repository::list_incidents(
        &repository::app_database_path(&app)?,
        severity.as_deref(),
        status.as_deref(),
    )
}

#[tauri::command]
pub fn update_incident_status(
    id: i64,
    status: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    repository::update_incident_status(&repository::app_database_path(&app)?, id, &status)
}

#[tauri::command]
pub fn get_incident_timeline(
    id: i64,
    app: tauri::AppHandle,
) -> Result<Vec<repository::IncidentTimelineEvent>, String> {
    repository::incident_timeline(&repository::app_database_path(&app)?, id)
}

#[tauri::command]
pub fn get_security_score(app: tauri::AppHandle) -> Result<repository::SecurityScore, String> {
    repository::security_score(&repository::app_database_path(&app)?)
}
