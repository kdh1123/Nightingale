use crate::{domain::file_monitoring::sha256_file, repository};
use std::path::Path;

pub fn scan(database_path: &Path, monitored_path_id: i64, root: &Path) -> Result<usize, String> {
    crate::repository::set_baseline_status(database_path, monitored_path_id, "running", None)?;
    let mut count = 0;
    fn visit(database_path: &Path, id: i64, path: &Path, count: &mut usize) -> Result<(), String> {
        for entry in std::fs::read_dir(path).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                visit(database_path, id, &entry.path(), count)?;
            } else if file_type.is_file() {
                let metadata = entry.metadata().map_err(|error| error.to_string())?;
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(0, |time| time.as_secs() as i64);
                let hash = sha256_file(&entry.path()).map_err(|error| error.to_string())?;
                repository::upsert_file_baseline(
                    database_path,
                    id,
                    &entry.path(),
                    metadata.len(),
                    modified,
                    &hash,
                )?;
                *count += 1;
            }
        }
        Ok(())
    }
    match visit(database_path, monitored_path_id, root, &mut count) {
        Ok(()) => {
            crate::repository::set_baseline_status(
                database_path,
                monitored_path_id,
                "complete",
                None,
            )?;
            Ok(count)
        }
        Err(error) => {
            let _ = crate::repository::set_baseline_status(
                database_path,
                monitored_path_id,
                "failed",
                Some(&error),
            );
            Err(error)
        }
    }
}
