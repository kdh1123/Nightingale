use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity { Low, Medium, High }

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionFeatures { pub mass_file_changes: bool, pub mass_extension_changes: bool, pub suspicious_file_activity: bool, pub integrity_changes: bool, pub phishing_url_analysis: bool, pub system_anomaly_detection: bool }

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicy { pub version: u32, pub sensitivity: Sensitivity, pub features: DetectionFeatures, pub monitored_paths: Vec<String>, pub excluded_paths: Vec<String> }

impl Default for SecurityPolicy {
    fn default() -> Self { Self { version: 1, sensitivity: Sensitivity::Medium, features: DetectionFeatures { mass_file_changes: true, mass_extension_changes: true, suspicious_file_activity: true, integrity_changes: true, phishing_url_analysis: true, system_anomaly_detection: true }, monitored_paths: Vec::new(), excluded_paths: Vec::new() } }
}

impl SecurityPolicy {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version == 0 { return Err("policy version must be positive"); }
        if self.monitored_paths.iter().any(|path| path.trim().is_empty()) { return Err("monitored paths cannot be empty"); }
        Ok(())
    }
}

#[cfg(test)]
mod tests { use super::*; #[test] fn defaults_are_safe_and_valid() { let policy = SecurityPolicy::default(); assert_eq!(policy.sensitivity, Sensitivity::Medium); assert!(policy.monitored_paths.is_empty()); assert!(policy.validate().is_ok()); } }
