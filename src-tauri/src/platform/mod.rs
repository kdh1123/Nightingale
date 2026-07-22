use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Supported,
    PartiallySupported,
    PermissionRequired,
    Unsupported,
    TemporarilyUnavailable,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCapability {
    pub key: &'static str,
    pub status: CapabilityStatus,
    pub detail: &'static str,
}

pub fn capabilities() -> Vec<PlatformCapability> {
    vec![
        PlatformCapability {
            key: "앱 상태",
            status: CapabilityStatus::Supported,
            detail: "현재 앱과 운영체제 정보를 안전하게 표시합니다.",
        },
        PlatformCapability {
            key: "시스템 모니터링",
            status: CapabilityStatus::TemporarilyUnavailable,
            detail: "Phase 1에서 추가됩니다.",
        },
        PlatformCapability {
            key: "파일 감시",
            status: CapabilityStatus::TemporarilyUnavailable,
            detail: "사용자가 선택한 폴더만 이후 단계에서 감시합니다.",
        },
        PlatformCapability {
            key: "운영체제 보안 저장소",
            status: CapabilityStatus::PartiallySupported,
            detail: "Windows Credential Manager와 macOS Keychain 연동은 설계 단계입니다.",
        },
        PlatformCapability {
            key: "상세 프로세스 정보",
            status: CapabilityStatus::PermissionRequired,
            detail: "일부 프로세스 정보는 추가 권한이 필요할 수 있습니다.",
        },
        PlatformCapability {
            key: "자동 대응",
            status: CapabilityStatus::Unsupported,
            detail: "Phase 0에서는 프로세스 종료, 파일 삭제, 네트워크 차단을 제공하지 않습니다.",
        },
    ]
}

pub fn operating_system() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else {
        "지원되지 않는 운영체제"
    }
}
