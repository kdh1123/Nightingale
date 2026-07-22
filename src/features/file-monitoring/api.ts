import { invoke } from "@tauri-apps/api/core";
export interface MonitoredPath {
  id: number;
  path: string;
  enabled: boolean;
  monitoringStatus: string;
  baselineStatus: string;
  lastScanAt: string | null;
  lastEventAt: string | null;
  lastError: string | null;
}
export interface FileEvent {
  id: number;
  eventKind: string;
  filePath: string;
  severity: string;
  occurredAt: string;
}
export const listMonitoredPaths = () => invoke<MonitoredPath[]>("list_monitored_paths");
export const addMonitoredPath = (path: string) => invoke<number>("add_monitored_path", { path });
export const removeMonitoredPath = (id: number) => invoke<void>("remove_monitored_path", { id });
export const startBaselineScan = (id: number) => invoke<number>("start_baseline_scan", { id });
export const pauseFileMonitoring = (id: number) => invoke<void>("pause_file_monitoring", { id });
export const resumeFileMonitoring = (id: number) => invoke<void>("resume_file_monitoring", { id });
export const listFileEvents = () => invoke<FileEvent[]>("list_file_events");
