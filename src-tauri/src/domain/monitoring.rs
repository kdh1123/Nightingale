use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub percent: f32,
}

impl Usage {
    pub fn new(total_bytes: u64, used_bytes: u64) -> Self {
        let percent = if total_bytes == 0 {
            0.0
        } else {
            (used_bytes as f64 / total_bytes as f64 * 100.0).clamp(0.0, 100.0) as f32
        };
        Self {
            total_bytes,
            used_bytes,
            percent,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSummary {
    pub pid: u32,
    pub name: String,
    pub executable_path: Option<String>,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub started_at_unix: u64,
    pub status: String,
    pub parent_pid: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSnapshot {
    pub operating_system: String,
    pub operating_system_version: Option<String>,
    pub kernel_version: Option<String>,
    pub host_name: Option<String>,
    pub cpu_model: Option<String>,
    pub logical_cpu_count: usize,
    pub cpu_percent: f32,
    pub memory: Usage,
    pub disk: Usage,
    pub uptime_seconds: u64,
    pub app_version: &'static str,
    pub collected_at_unix: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitoredPath {
    pub id: i64,
    pub path: String,
    pub active: bool,
    pub baseline_status: String,
    pub last_scanned_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileEventKind {
    Created,
    Modified,
    Deleted,
    Renamed,
    MetadataChanged,
    Unknown,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEvent {
    pub id: i64,
    pub monitored_path_id: i64,
    pub kind: FileEventKind,
    pub path: String,
    pub previous_path: Option<String>,
    pub severity: String,
    pub detected_at: String,
    pub reviewed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn usage_is_bounded() {
        assert_eq!(Usage::new(0, 4).percent, 0.0);
        assert_eq!(Usage::new(10, 20).percent, 100.0);
    }
}
