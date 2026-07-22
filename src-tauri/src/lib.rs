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
            let monitoring = app.state::<FileMonitoringService>();
            for path in repository::enabled_monitored_paths(&database_path)
                .map_err(std::io::Error::other)?
            {
                if let Err(error) =
                    monitoring.start(path.id, path.path.into(), database_path.clone())
                {
                    let _ = repository::set_monitoring_error(&database_path, path.id, &error);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tauri_api::get_app_status,
            tauri_api::get_system_snapshot,
            tauri_api::list_processes,
            tauri_api::add_monitored_path,
            tauri_api::list_monitored_paths,
            tauri_api::remove_monitored_path,
            tauri_api::start_baseline_scan,
            tauri_api::list_file_events,
            tauri_api::pause_file_monitoring,
            tauri_api::resume_file_monitoring,
            tauri_api::list_security_events,
            tauri_api::mark_security_event_reviewed,
            tauri_api::list_incidents,
            tauri_api::update_incident_status,
            tauri_api::get_security_score
        ])
        .run(tauri::generate_context!())
        .expect("Nightingale failed to run");
}
