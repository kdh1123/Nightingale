use crate::application::monitoring::{collect_snapshot, list_processes as collect_processes};
use crate::application::{app_status, AppStatus};
use crate::domain::monitoring::{ProcessSummary, SystemSnapshot};
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
