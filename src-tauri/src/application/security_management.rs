use crate::{domain::security_management::ApplicationSettings, repository};
use std::path::Path;

pub fn update_settings(
    database_path: &Path,
    settings: ApplicationSettings,
) -> Result<ApplicationSettings, String> {
    settings.validate().map_err(str::to_string)?;
    repository::save_application_settings(database_path, &settings)?;
    Ok(settings)
}

pub fn cleanup_logs(database_path: &Path) -> Result<usize, String> {
    let settings = repository::application_settings(database_path)?;
    repository::cleanup_logs(database_path, settings.log_retention_days)
}
