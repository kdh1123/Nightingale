pub mod application;
pub mod domain;
pub mod platform;
mod repository;
mod tauri_api;

use repository::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    tauri::Builder::default()
        .setup(|app| {
            // Phase 0 verifies the database and migrations at launch. Repositories are
            // constructed per use case in later phases; a raw connection is not shared.
            let _database = Database::open_for_app(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![tauri_api::get_app_status])
        .run(tauri::generate_context!())
        .expect("Nightingale failed to run");
}
