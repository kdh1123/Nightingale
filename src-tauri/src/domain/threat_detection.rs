use crate::domain::{file_monitoring::FileEventKind, policy::SecurityPolicy};
use serde::{Deserialize, Serialize};

pub const CORRELATION_WINDOW_SECONDS: i64 = 300;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub const fn score_penalty(self) -> i64 {
        match self {
            Self::Info => 0,
            Self::Low => 3,
            Self::Medium => 10,
            Self::High => 25,
            Self::Critical => 45,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileEventContext<'a> {
    pub kind: FileEventKind,
    pub path: &'a str,
    pub differs_from_baseline: bool,
    pub recent_changes: i64,
    pub policy: &'a SecurityPolicy,
}

#[derive(Clone, Debug)]
pub struct ThreatAssessment {
    pub event_type: &'static str,
    pub severity: Severity,
    pub title: &'static str,
    pub description: &'static str,
    pub correlation_key: String,
}

/// Centralized, deterministic threat policy. New signal sources can add their
/// own assessors without changing persistence or Tauri command code.
pub struct ThreatDetectionService;

impl ThreatDetectionService {
    pub const MASS_CHANGE_THRESHOLD: i64 = 20;

    pub fn assess_file_event(context: FileEventContext<'_>) -> ThreatAssessment {
        let suspicious_extension =
            ["exe", "bat", "cmd", "ps1", "js", "vbs", "scr"]
                .iter()
                .any(|extension| {
                    context
                        .path
                        .rsplit('.')
                        .next()
                        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
                });
        let (event_type, severity, title, description) =
            if context.policy.features.mass_file_changes
                && context.recent_changes >= mass_change_threshold(context.policy)
            {
                (
                    "mass_file_change",
                    Severity::High,
                    "대량 파일 변경 감지",
                    "짧은 시간에 다수의 파일 변경이 감지되었습니다.",
                )
            } else if context.policy.features.integrity_changes
                && context.differs_from_baseline
                && suspicious_extension
            {
                (
                    "suspicious_integrity_change",
                    Severity::High,
                    "의심스러운 파일 무결성 변경",
                    "기준선 파일의 실행 가능 또는 스크립트 파일 변경이 감지되었습니다.",
                )
            } else if context.policy.features.integrity_changes && context.differs_from_baseline {
                (
                    "integrity_changed",
                    Severity::Medium,
                    "무결성 기준선과 다른 파일 활동",
                    "선택한 감시 폴더에서 기준선과 다른 파일 생성·변경·삭제가 감지되었습니다.",
                )
            } else if context.policy.features.suspicious_file_activity
                && suspicious_extension
                && matches!(
                    context.kind,
                    FileEventKind::Created | FileEventKind::Modified
                )
            {
                (
                    "suspicious_file_activity",
                    Severity::Low,
                    "의심스러운 파일 활동",
                    "감시 폴더에서 실행 가능 또는 스크립트 파일 활동이 감지되었습니다.",
                )
            } else {
                (
                    "file_activity",
                    Severity::Info,
                    "파일 활동 감지",
                    "선택한 감시 폴더에서 파일 활동이 감지되었습니다.",
                )
            };
        ThreatAssessment {
            event_type,
            severity,
            title,
            description,
            correlation_key: format!(
                "{event_type}:{}",
                context.path.rsplit('/').nth(1).unwrap_or(context.path)
            ),
        }
    }
}

fn mass_change_threshold(policy: &SecurityPolicy) -> i64 {
    match policy.sensitivity {
        crate::domain::policy::Sensitivity::Low => 35,
        crate::domain::policy::Sensitivity::Medium => ThreatDetectionService::MASS_CHANGE_THRESHOLD,
        crate::domain::policy::Sensitivity::High => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_escalates_mass_changes_and_suspicious_baseline_changes() {
        let mass = ThreatDetectionService::assess_file_event(FileEventContext {
            kind: FileEventKind::Modified,
            path: "/watch/a.txt",
            differs_from_baseline: false,
            recent_changes: 20,
            policy: &SecurityPolicy::default(),
        });
        assert_eq!(mass.severity, Severity::High);
        let suspicious = ThreatDetectionService::assess_file_event(FileEventContext {
            kind: FileEventKind::Modified,
            path: "/watch/update.ps1",
            differs_from_baseline: true,
            recent_changes: 1,
            policy: &SecurityPolicy::default(),
        });
        assert_eq!(suspicious.severity, Severity::High);
    }
}
