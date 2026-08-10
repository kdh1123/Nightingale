//! What the detector looks for (policy) and what it is told to trust (allowlist).

use crate::domain::policy::SecurityPolicy;
use crate::repository;

#[tauri::command]
pub fn get_detection_policy(app: tauri::AppHandle) -> Result<SecurityPolicy, String> {
    repository::detection_policy(&repository::app_database_path(&app)?)
}

#[tauri::command]
pub fn update_detection_policy(
    policy: SecurityPolicy,
    app: tauri::AppHandle,
) -> Result<SecurityPolicy, String> {
    repository::save_detection_policy(&repository::app_database_path(&app)?, policy)
}

#[tauri::command]
pub fn list_allowlist_entries(
    app: tauri::AppHandle,
) -> Result<Vec<repository::AllowlistEntry>, String> {
    repository::list_allowlist_entries(&repository::app_database_path(&app)?)
}

#[tauri::command]
pub fn add_allowlist_entry(
    entry_type: String,
    value: String,
    expires_in_days: Option<i64>,
    app: tauri::AppHandle,
) -> Result<repository::AllowlistEntry, String> {
    repository::add_allowlist_entry(
        &repository::app_database_path(&app)?,
        &entry_type,
        &value,
        expires_in_days,
    )
}

#[tauri::command]
pub fn remove_allowlist_entry(id: i64, app: tauri::AppHandle) -> Result<(), String> {
    repository::remove_allowlist_entry(&repository::app_database_path(&app)?, id)
}

#[tauri::command]
pub fn list_allowlist_audit(
    app: tauri::AppHandle,
) -> Result<Vec<repository::AllowlistAuditEntry>, String> {
    repository::list_allowlist_audit(&repository::app_database_path(&app)?)
}
