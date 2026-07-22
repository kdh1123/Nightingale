import { invoke } from "@tauri-apps/api/core";
export interface Usage {
  totalBytes: number;
  usedBytes: number;
  percent: number;
}
export interface SystemSnapshot {
  operatingSystem: string;
  operatingSystemVersion?: string;
  logicalCpuCount: number;
  cpuPercent: number;
  memory: Usage;
  disk: Usage;
  collectedAtUnix: number;
}
export interface ProcessSummary {
  pid: number;
  name: string;
  cpuPercent: number;
  memoryBytes: number;
}
export const getSystemSnapshot = () => invoke<SystemSnapshot>("get_system_snapshot");
export const listProcesses = (query?: string, sortBy?: string) =>
  invoke<ProcessSummary[]>("list_processes", { query: query || null, sortBy: sortBy || null });
