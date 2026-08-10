//! Watched folder lifecycle and the raw file activity log. These commands keep the watcher
//! service and the stored monitoring status in step with each other.

use crate::application::baseline;
use crate::application::file_monitoring::FileMonitoringService;
use crate::domain::file_monitoring::validate_watch_path;
use crate::repository;

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
