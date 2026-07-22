use crate::{
    domain::{
        file_monitoring::FileEventKind,
        threat_detection::{FileEventContext, ThreatDetectionService},
    },
    repository,
};
use std::path::Path;

pub fn analyze_file_event(
    database_path: &Path,
    file_event_id: i64,
    monitored_path_id: i64,
    path: &Path,
    kind: FileEventKind,
) -> Result<(), String> {
    let context =
        repository::file_event_analysis_context(database_path, monitored_path_id, path, kind)?;
    if !context.threat_detection_enabled {
        return Ok(());
    }
    let assessment = ThreatDetectionService::assess_file_event(FileEventContext {
        kind,
        path: &path.to_string_lossy(),
        differs_from_baseline: context.differs_from_baseline,
        recent_changes: context.recent_changes,
    });
    repository::persist_threat_assessment(database_path, file_event_id, &assessment)
}
