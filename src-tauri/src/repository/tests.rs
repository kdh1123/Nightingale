use super::*;
use crate::domain::file_monitoring::FileEventKind;
use crate::domain::threat_detection::{Severity, ThreatAssessment};
use rusqlite::Connection;

#[test]
fn creates_minimum_schema_in_memory() {
    let connection = Connection::open_in_memory().expect("in-memory database");
    connection
        .execute_batch(include_str!("../../migrations/0001_initial.sql"))
        .expect("migration");
    let count: i64 = connection.query_row("SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'security_policies'", [], |row| row.get(0)).expect("schema query");
    assert_eq!(count, 1);
}

#[test]
fn applies_file_monitoring_migration_only_once() {
    let path = std::env::temp_dir().join(format!(
        "nightingale-migration-{}.sqlite3",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    Database::open(path.clone()).expect("first open");
    Database::open(path.clone()).expect("second open");
    std::fs::remove_file(path).expect("cleanup");
}

#[test]
fn correlates_events_and_calculates_security_score() {
    let path = std::env::temp_dir().join(format!(
        "nightingale-threat-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .elapsed()
            .map_or(0, |duration| duration.as_nanos())
    ));
    Database::open(path.clone()).expect("database");
    let monitored = add_monitored_path(&path, std::path::Path::new("/tmp/nightingale-threat"))
        .expect("monitored path");
    let assessment = ThreatAssessment {
        event_type: "mass_file_change",
        severity: Severity::High,
        title: "대량 파일 변경 감지",
        description: "test",
        correlation_key: "mass_file_change:/tmp".to_string(),
    };
    for _ in 0..2 {
        let event = record_file_event(
            &path,
            monitored,
            std::path::Path::new("/tmp/nightingale-threat/a.txt"),
            FileEventKind::Modified,
        )
        .expect("file event");
        persist_threat_assessment(&path, event, &assessment).expect("assessment");
    }
    let incidents = list_incidents(&path, Some("high"), Some("open")).expect("incidents");
    assert_eq!(incidents.len(), 1);
    assert_eq!(incidents[0].event_count, 2);
    let timeline = incident_timeline(&path, incidents[0].id).expect("timeline");
    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].event_type, "mass_file_change");
    assert_eq!(
        timeline[0].file_path.as_deref(),
        Some("/tmp/nightingale-threat/a.txt")
    );
    assert_eq!(security_score(&path).expect("score").score, 75);
    std::fs::remove_file(path).expect("cleanup");
}

#[test]
fn allowlist_is_audited_and_suppresses_matching_file_analysis() {
    let path = std::env::temp_dir().join(format!(
        "nightingale-allowlist-{}-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        "test"
    ));
    Database::open(path.clone()).expect("database");
    let monitored = add_monitored_path(&path, std::path::Path::new("/tmp/nightingale-allow"))
        .expect("monitored path");
    let entry = add_allowlist_entry(&path, "extension", ".log", Some(30)).expect("allowlist entry");
    assert_eq!(entry.value, "log");
    let context = file_event_analysis_context(
        &path,
        monitored,
        std::path::Path::new("/tmp/nightingale-allow/build.log"),
        FileEventKind::Created,
    )
    .expect("analysis context");
    assert!(context.allowlisted);
    remove_allowlist_entry(&path, entry.id).expect("remove entry");
    assert_eq!(list_allowlist_audit(&path).expect("audit").len(), 2);
    let mut policy = detection_policy(&path).expect("default policy");
    policy.sensitivity = crate::domain::policy::Sensitivity::High;
    let saved = save_detection_policy(&path, policy).expect("saved policy");
    assert_eq!(saved.sensitivity, crate::domain::policy::Sensitivity::High);
    std::fs::remove_file(path).expect("cleanup");
}
