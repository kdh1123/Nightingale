use crate::application::monitoring::{collect_snapshot, list_processes as collect_processes};
use crate::application::{app_status, baseline, AppStatus};
use crate::domain::monitoring::{ProcessSummary, SystemSnapshot};
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
