use crate::domain::monitoring::{ProcessSummary, SystemSnapshot, Usage};
use sysinfo::{Disks, Pid, ProcessesToUpdate, Signal, System};

pub fn collect_snapshot(system: &mut System) -> SystemSnapshot {
    system.refresh_cpu_all();
    system.refresh_memory();
    let disks = Disks::new_with_refreshed_list();
    let (total_disk, available_disk) = disks.iter().fold((0, 0), |(total, available), disk| {
        (
            total + disk.total_space(),
            available + disk.available_space(),
        )
    });
    let cpu_percent = system.global_cpu_usage().clamp(0.0, 100.0);
    SystemSnapshot {
        operating_system: System::name().unwrap_or_else(|| "unknown".into()),
        operating_system_version: System::os_version(),
        kernel_version: System::kernel_version(),
        host_name: System::host_name(),
        cpu_model: system.cpus().first().map(|cpu| cpu.brand().to_string()),
        logical_cpu_count: system.cpus().len(),
        cpu_percent,
        memory: Usage::new(system.total_memory(), system.used_memory()),
        disk: Usage::new(total_disk, total_disk.saturating_sub(available_disk)),
        uptime_seconds: System::uptime(),
        app_version: env!("CARGO_PKG_VERSION"),
        collected_at_unix: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs()),
    }
}

pub fn list_processes(
    system: &mut System,
    query: Option<&str>,
    sort_by: Option<&str>,
) -> Vec<ProcessSummary> {
    system.refresh_processes(ProcessesToUpdate::All, true);
    let query = query.unwrap_or("").to_lowercase();
    let mut processes: Vec<_> = system
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            let name = process.name().to_string_lossy().to_string();
            (query.is_empty()
                || name.to_lowercase().contains(&query)
                || pid.to_string().contains(&query))
            .then(|| ProcessSummary {
                pid: pid.as_u32(),
                name,
                executable_path: process.exe().map(|path| path.to_string_lossy().to_string()),
                cpu_percent: process.cpu_usage().clamp(0.0, 100.0),
                memory_bytes: process.memory(),
                started_at_unix: process.start_time(),
                status: format!("{:?}", process.status()),
                parent_pid: process.parent().map(|parent| parent.as_u32()),
            })
        })
        .collect();
    match sort_by.unwrap_or("cpu") {
        "memory" => processes.sort_by_key(|item| std::cmp::Reverse(item.memory_bytes)),
        "name" => processes.sort_by(|left, right| left.name.cmp(&right.name)),
        _ => processes.sort_by(|left, right| right.cpu_percent.total_cmp(&left.cpu_percent)),
    };
    processes
}

/// Sends a graceful termination signal to a user-selected process.
/// The application never terminates itself and does not perform automatic process termination.
pub fn terminate_process(system: &mut System, pid: u32) -> Result<(), String> {
    if pid == std::process::id() {
        return Err("Nightingale 자체 프로세스는 종료할 수 없습니다.".to_string());
    }
    system.refresh_processes(ProcessesToUpdate::All, true);
    let process = system.process(Pid::from_u32(pid)).ok_or_else(|| {
        "해당 프로세스를 찾을 수 없습니다. 이미 종료되었을 수 있습니다.".to_string()
    })?;
    process
        .kill_with(Signal::Term)
        .ok_or_else(|| "이 운영체제에서는 프로세스 종료를 지원하지 않습니다.".to_string())
        .and_then(|terminated| {
            terminated.then_some(()).ok_or_else(|| {
                "프로세스 종료 요청이 거부되었습니다. 권한을 확인하세요.".to_string()
            })
        })
}
