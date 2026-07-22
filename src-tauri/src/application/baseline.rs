use crate::{domain::file_monitoring::sha256_file, repository};
use std::path::Path;

pub fn scan(database_path: &Path, monitored_path_id: i64, root: &Path) -> Result<usize, String> {
    repository::set_baseline_status(database_path, monitored_path_id, "running", None)?;
    let writer = repository::BaselineWriter::open(database_path)?;
    let mut count = 0;
    fn visit(
        writer: &repository::BaselineWriter,
        id: i64,
        path: &Path,
        count: &mut usize,
    ) -> Result<(), String> {
        for entry in
            std::fs::read_dir(path).map_err(|error| format!("{}: {error}", path.display()))?
        {
            let entry = entry.map_err(|error| format!("{}: {error}", path.display()))?;
            let file_type = entry
                .file_type()
                .map_err(|error| format!("{}: {error}", entry.path().display()))?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                visit(writer, id, &entry.path(), count)?;
            } else if file_type.is_file() {
                let metadata = entry
                    .metadata()
                    .map_err(|error| format!("{}: {error}", entry.path().display()))?;
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(0, |time| time.as_secs() as i64);
                let hash = sha256_file(&entry.path())
                    .map_err(|error| format!("{}: {error}", entry.path().display()))?;
                writer.upsert(id, &entry.path(), metadata.len(), modified, &hash)?;
                *count += 1;
            }
        }
        Ok(())
    }
    match visit(&writer, monitored_path_id, root, &mut count) {
        Ok(()) => {
            repository::set_baseline_status(database_path, monitored_path_id, "complete", None)?;
            Ok(count)
        }
        Err(error) => {
            let _ = repository::set_baseline_status(
                database_path,
                monitored_path_id,
                "failed",
                Some(&error),
            );
            Err(error)
        }
    }
}
