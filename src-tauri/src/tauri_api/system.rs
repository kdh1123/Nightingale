//! Read-only system telemetry commands. All of these share the one `System` handle held in
//! Tauri state, so each locks it for as short a time as possible.

use crate::application::monitoring::{
    collect_snapshot, list_processes as collect_processes, terminate_process as terminate,
};
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

#[tauri::command]
pub fn terminate_process(pid: u32, system: tauri::State<'_, Mutex<System>>) -> Result<(), String> {
    let mut system = system
        .lock()
        .map_err(|_| "monitoring state is unavailable".to_string())?;
    terminate(&mut system, pid)
}
