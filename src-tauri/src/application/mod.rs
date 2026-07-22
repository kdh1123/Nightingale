use crate::platform::{capabilities, operating_system, PlatformCapability};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub app_version: &'static str,
    pub operating_system: &'static str,
    pub capabilities: Vec<PlatformCapability>,
}
pub fn app_status() -> AppStatus {
    AppStatus {
        app_version: env!("CARGO_PKG_VERSION"),
        operating_system: operating_system(),
        capabilities: capabilities(),
    }
}
pub mod baseline;
pub mod error;
pub mod file_monitoring;
pub mod monitoring;
pub mod security_management;
pub mod threat_detection;
