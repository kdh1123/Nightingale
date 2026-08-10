//! Preferences, the notification inbox, and the local report/retention operations.

use crate::application::security_management;
use crate::domain::security_management::ApplicationSettings;
use crate::repository;

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
pub fn generate_security_report(
    app: tauri::AppHandle,
) -> Result<repository::SecurityReport, String> {
    repository::save_security_report(&repository::app_database_path(&app)?)
}

#[tauri::command]
pub fn cleanup_security_logs(app: tauri::AppHandle) -> Result<usize, String> {
    security_management::cleanup_logs(&repository::app_database_path(&app)?)
}
