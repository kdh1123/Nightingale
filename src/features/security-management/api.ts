import { invoke } from "@tauri-apps/api/core";
import type { SecurityEvent, SecurityScore } from "../security-events/api";

export interface ApplicationSettings {
  monitoringEnabled: boolean;
  threatDetectionEnabled: boolean;
  autoBaselineRefresh: boolean;
  securityScoreEnabled: boolean;
  logRetentionDays: number;
  uiTheme: "system" | "light" | "dark";
}
export interface Notification {
  id: number;
  incidentId: number | null;
  severity: string;
  title: string;
  message: string;
  read: boolean;
  createdAt: string;
}
export interface SeverityCounts {
  info: number;
  low: number;
  medium: number;
  high: number;
  critical: number;
}
export interface SecurityReport {
  generatedAt: string;
  securityScore: SecurityScore;
  totalIncidents: number;
  severityCounts: SeverityCounts;
  monitoredFolderCount: number;
  fileEventCount: number;
  recentDetections: SecurityEvent[];
  recentRiskEvents: SecurityEvent[];
}
export type Sensitivity = "low" | "medium" | "high";
export interface DetectionPolicy {
  version: number;
  sensitivity: Sensitivity;
  features: {
    massFileChanges: boolean;
    massExtensionChanges: boolean;
    suspiciousFileActivity: boolean;
    integrityChanges: boolean;
    phishingUrlAnalysis: boolean;
    systemAnomalyDetection: boolean;
  };
  monitoredPaths: string[];
  excludedPaths: string[];
}
export interface AllowlistEntry {
  id: number;
  entryType: "path" | "extension";
  value: string;
  expiresAt: string | null;
  createdAt: string;
}
export interface AllowlistAuditEntry {
  id: number;
  entryId: number | null;
  action: "added" | "removed";
  entryType: "path" | "extension";
  value: string;
  occurredAt: string;
}

export const getApplicationSettings = () => invoke<ApplicationSettings>("get_application_settings");
export const updateApplicationSettings = (settings: ApplicationSettings) =>
  invoke<ApplicationSettings>("update_application_settings", { settings });
export const listNotifications = () => invoke<Notification[]>("list_notifications");
export const markNotificationRead = (id: number) => invoke<void>("mark_notification_read", { id });
export const getSecurityReport = () => invoke<SecurityReport>("get_security_report");
export const cleanupSecurityLogs = () => invoke<number>("cleanup_security_logs");
export const getDetectionPolicy = () => invoke<DetectionPolicy>("get_detection_policy");
export const updateDetectionPolicy = (policy: DetectionPolicy) =>
  invoke<DetectionPolicy>("update_detection_policy", { policy });
export const listAllowlistEntries = () => invoke<AllowlistEntry[]>("list_allowlist_entries");
export const addAllowlistEntry = (
  entryType: AllowlistEntry["entryType"],
  value: string,
  expiresInDays: number | null,
) => invoke<AllowlistEntry>("add_allowlist_entry", { entryType, value, expiresInDays });
export const removeAllowlistEntry = (id: number) => invoke<void>("remove_allowlist_entry", { id });
export const listAllowlistAudit = () => invoke<AllowlistAuditEntry[]>("list_allowlist_audit");
