import { invoke } from "@tauri-apps/api/core";
import type { SecurityEvent, SecurityScore } from "../security-events/api";

export interface ApplicationSettings { monitoringEnabled: boolean; threatDetectionEnabled: boolean; autoBaselineRefresh: boolean; securityScoreEnabled: boolean; logRetentionDays: number; uiTheme: "system" | "light" | "dark"; }
export interface Notification { id: number; incidentId: number | null; severity: string; title: string; message: string; read: boolean; createdAt: string; }
export interface SeverityCounts { info: number; low: number; medium: number; high: number; critical: number; }
export interface SecurityReport { generatedAt: string; securityScore: SecurityScore; totalIncidents: number; severityCounts: SeverityCounts; monitoredFolderCount: number; fileEventCount: number; recentDetections: SecurityEvent[]; recentRiskEvents: SecurityEvent[]; }

export const getApplicationSettings = () => invoke<ApplicationSettings>("get_application_settings");
export const updateApplicationSettings = (settings: ApplicationSettings) => invoke<ApplicationSettings>("update_application_settings", { settings });
export const listNotifications = () => invoke<Notification[]>("list_notifications");
export const markNotificationRead = (id: number) => invoke<void>("mark_notification_read", { id });
export const getSecurityReport = () => invoke<SecurityReport>("get_security_report");
export const cleanupSecurityLogs = () => invoke<number>("cleanup_security_logs");
