use crate::application::monitoring::{collect_snapshot, list_processes as collect_processes};
use crate::application::{app_status, baseline, security_management, AppStatus};
use crate::domain::monitoring::{ProcessSummary, SystemSnapshot};
use crate::domain::security_management::ApplicationSettings;
use crate::{
    application::file_monitoring::FileMonitoringService,
    domain::file_monitoring::validate_watch_path, repository,
};
use std::sync::Mutex;
use sysinfo::System;

#[tauri::command]
pub fn get_app_status() -> AppStatus {
    app_status()
}

#[tauri::command]
pub fn get_system_snapshot(
    system: tauri::State<'_, Mutex<System>>,
) -> Result<SystemSnapshot, String> {
    let mut system = system
        .lock()
        .map_err(|_| "monitoring state is unavailable".to_string())?;
    Ok(collect_snapshot(&mut system))
}

#[tauri::command]
pub fn list_processes(
    query: Option<String>,
    sort_by: Option<String>,
    system: tauri::State<'_, Mutex<System>>,
) -> Result<Vec<ProcessSummary>, String> {
    let mut system = system
        .lock()
        .map_err(|_| "monitoring state is unavailable".to_string())?;
    Ok(collect_processes(
        &mut system,
        query.as_deref(),
        sort_by.as_deref(),
    ))
}

#[tauri::command]
pub fn add_monitored_path(
    path: String,
    app: tauri::AppHandle,
    service: tauri::State<'_, FileMonitoringService>,
) -> Result<i64, String> {
    let path = validate_watch_path(std::path::Path::new(&path))?;
    let database = repository::app_database_path(&app)?;
    let id = repository::add_monitored_path(&database, &path)?;
    if !repository::application_settings(&database)?.monitoring_enabled {
        repository::set_monitoring_status(&database, id, "stopped")?;
        return Ok(id);
    }
    if let Err(error) = service.start(id, path, database.clone()) {
        let _ = repository::remove_monitored_path(&database, id);
        return Err(error);
    }
    repository::set_monitoring_status(&database, id, "running")?;
    Ok(id)
}
#[tauri::command]
pub fn list_monitored_paths(
    app: tauri::AppHandle,
) -> Result<Vec<repository::MonitoredPath>, String> {
    repository::list_monitored_paths(&repository::app_database_path(&app)?)
}
#[tauri::command]
pub fn remove_monitored_path(
    id: i64,
    app: tauri::AppHandle,
    service: tauri::State<'_, FileMonitoringService>,
) -> Result<(), String> {
    service.stop(id)?;
    repository::remove_monitored_path(&repository::app_database_path(&app)?, id)
}
#[tauri::command]
pub fn pause_file_monitoring(
    id: i64,
    app: tauri::AppHandle,
    service: tauri::State<'_, FileMonitoringService>,
) -> Result<(), String> {
    service.stop(id)?;
    repository::set_monitoring_status(&repository::app_database_path(&app)?, id, "paused")
}
#[tauri::command]
pub fn resume_file_monitoring(
    id: i64,
    app: tauri::AppHandle,
    service: tauri::State<'_, FileMonitoringService>,
) -> Result<(), String> {
    let database = repository::app_database_path(&app)?;
    if !repository::application_settings(&database)?.monitoring_enabled {
        return Err("설정에서 파일 감시가 비활성화되어 있습니다.".to_string());
    }
    if let Err(error) = service.start(
        id,
        repository::monitored_path(&database, id)?,
        database.clone(),
    ) {
        let _ = repository::set_monitoring_error(&database, id, &error);
        return Err(error);
    }
    repository::set_monitoring_status(&database, id, "running")
}
#[tauri::command]
pub fn start_baseline_scan(id: i64, app: tauri::AppHandle) -> Result<usize, String> {
    let database = repository::app_database_path(&app)?;
    baseline::scan(&database, id, &repository::monitored_path(&database, id)?)
}
#[tauri::command]
pub fn list_file_events(app: tauri::AppHandle) -> Result<Vec<repository::FileEvent>, String> {
    repository::list_file_events(&repository::app_database_path(&app)?)
}
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
pub fn get_security_score(app: tauri::AppHandle) -> Result<repository::SecurityScore, String> {
    repository::security_score(&repository::app_database_path(&app)?)
}
#[tauri::command]
pub fn get_application_settings(app: tauri::AppHandle) -> Result<ApplicationSettings, String> {
    repository::application_settings(&repository::app_database_path(&app)?)
}
#[tauri::command]
pub fn update_application_settings(
    settings: ApplicationSettings,
    app: tauri::AppHandle,
) -> Result<ApplicationSettings, String> {
    security_management::update_settings(&repository::app_database_path(&app)?, settings)
}
#[tauri::command]
pub fn list_notifications(app: tauri::AppHandle) -> Result<Vec<repository::Notification>, String> {
    repository::list_notifications(&repository::app_database_path(&app)?)
}
#[tauri::command]
pub fn mark_notification_read(id: i64, app: tauri::AppHandle) -> Result<(), String> {
    repository::mark_notification_read(&repository::app_database_path(&app)?, id)
}
#[tauri::command]
pub fn get_security_report(app: tauri::AppHandle) -> Result<repository::SecurityReport, String> {
    repository::security_report(&repository::app_database_path(&app)?)
}
#[tauri::command]
pub fn list_file_events_filtered(
    query: Option<String>,
    severity: Option<String>,
    from: Option<String>,
    to: Option<String>,
    sort_desc: Option<bool>,
    app: tauri::AppHandle,
) -> Result<Vec<repository::FileEvent>, String> {
    repository::list_file_events_filtered(
        &repository::app_database_path(&app)?,
        query.as_deref(),
        severity.as_deref(),
        from.as_deref(),
        to.as_deref(),
        sort_desc.unwrap_or(true),
    )
}
#[tauri::command]
pub fn cleanup_security_logs(app: tauri::AppHandle) -> Result<usize, String> {
    security_management::cleanup_logs(&repository::app_database_path(&app)?)
}
