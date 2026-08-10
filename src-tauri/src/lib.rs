pub mod application;
pub mod domain;
pub mod platform;
mod repository;
mod tauri_api;

use application::file_monitoring::FileMonitoringService;
use repository::Database;
use std::sync::Mutex;
use sysinfo::System;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    tauri::Builder::default()
        .manage(Mutex::new(System::new_all()))
        .manage(FileMonitoringService::default())
        .setup(|app| {
            // Phase 0 verifies the database and migrations at launch. Repositories are
            // constructed per use case in later phases; a raw connection is not shared.
            let _database = Database::open_for_app(app.handle())?;
            let database_path =
                repository::app_database_path(app.handle()).map_err(std::io::Error::other)?;
            // Apply the persisted retention policy on every application launch.
            let _ = application::security_management::cleanup_logs(&database_path);
            let monitoring = app.state::<FileMonitoringService>();
            if repository::application_settings(&database_path)
                .map_err(std::io::Error::other)?
                .monitoring_enabled
            {
                for path in repository::enabled_monitored_paths(&database_path)
                    .map_err(std::io::Error::other)?
                {
                    if let Err(error) =
                        monitoring.start(path.id, path.path.into(), database_path.clone())
                    {
                        let _ = repository::set_monitoring_error(&database_path, path.id, &error);
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tauri_api::system::get_app_status,
            tauri_api::system::get_system_snapshot,
            tauri_api::system::list_processes,
            tauri_api::system::terminate_process,
            tauri_api::file_monitoring::add_monitored_path,
            tauri_api::file_monitoring::list_monitored_paths,
            tauri_api::file_monitoring::remove_monitored_path,
            tauri_api::file_monitoring::start_baseline_scan,
            tauri_api::file_monitoring::list_file_events,
            tauri_api::file_monitoring::pause_file_monitoring,
            tauri_api::file_monitoring::resume_file_monitoring,
            tauri_api::file_monitoring::list_file_events_filtered,
            tauri_api::security_events::list_security_events,
            tauri_api::security_events::mark_security_event_reviewed,
            tauri_api::security_events::list_incidents,
            tauri_api::security_events::update_incident_status,
            tauri_api::security_events::get_incident_timeline,
            tauri_api::security_events::get_security_score,
            tauri_api::detection::get_detection_policy,
            tauri_api::detection::update_detection_policy,
            tauri_api::detection::list_allowlist_entries,
            tauri_api::detection::add_allowlist_entry,
            tauri_api::detection::remove_allowlist_entry,
            tauri_api::detection::list_allowlist_audit,
            tauri_api::settings::get_application_settings,
            tauri_api::settings::update_application_settings,
            tauri_api::settings::list_notifications,
            tauri_api::settings::mark_notification_read,
            tauri_api::settings::get_security_report,
            tauri_api::settings::generate_security_report,
            tauri_api::settings::cleanup_security_logs
        ])
        .run(tauri::generate_context!())
        .expect("Nightingale failed to run");
}
