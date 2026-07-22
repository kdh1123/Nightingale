use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("permission denied")]
    PermissionDenied,
    #[error("invalid security policy")]
    InvalidPolicy,
    #[error("feature is not supported on this platform")]
    UnsupportedFeature,
    #[error("temporary service failure")]
    TemporarilyUnavailable,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFacingError {
    pub code: &'static str,
    pub message: &'static str,
    pub action: &'static str,
    pub retryable: bool,
}

impl From<AppError> for UserFacingError {
    fn from(error: AppError) -> Self {
        match error {
            AppError::PermissionDenied => Self {
                code: "permission_denied",
                message: "이 기능을 사용하려면 추가 권한이 필요합니다.",
                action: "권한을 확인한 후 다시 시도하세요.",
                retryable: true,
            },
            AppError::InvalidPolicy => Self {
                code: "invalid_policy",
                message: "보안 정책을 적용할 수 없습니다.",
                action: "기본 설정으로 복구하세요.",
                retryable: false,
            },
            AppError::UnsupportedFeature => Self {
                code: "unsupported_feature",
                message: "현재 운영체제에서 이 기능을 지원하지 않습니다.",
                action: "지원 상태를 확인하세요.",
                retryable: false,
            },
            AppError::TemporarilyUnavailable => Self {
                code: "temporarily_unavailable",
                message: "기능을 지금 사용할 수 없습니다.",
                action: "잠시 후 다시 시도하세요.",
                retryable: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_has_stable_user_code() {
        let error: UserFacingError = AppError::PermissionDenied.into();
        assert_eq!(error.code, "permission_denied");
        assert!(error.retryable);
    }
}
