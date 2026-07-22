use crate::application::{app_status, AppStatus};

#[tauri::command]
pub fn get_app_status() -> AppStatus { app_status() }
