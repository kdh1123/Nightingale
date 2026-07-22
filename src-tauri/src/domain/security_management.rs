use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSettings {
    pub monitoring_enabled: bool,
    pub threat_detection_enabled: bool,
    pub auto_baseline_refresh: bool,
    pub security_score_enabled: bool,
    pub log_retention_days: i64,
    pub ui_theme: String,
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            monitoring_enabled: true,
            threat_detection_enabled: true,
            auto_baseline_refresh: false,
            security_score_enabled: true,
            log_retention_days: 90,
            ui_theme: "system".to_string(),
        }
    }
}

impl ApplicationSettings {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(1..=3650).contains(&self.log_retention_days) {
            return Err("로그 보관 기간은 1~3650일이어야 합니다.");
        }
        if !["system", "light", "dark"].contains(&self.ui_theme.as_str()) {
            return Err("지원하지 않는 UI 테마입니다.");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_settings_are_valid() {
        assert!(ApplicationSettings::default().validate().is_ok());
    }
}
