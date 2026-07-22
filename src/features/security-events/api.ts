import { invoke } from "@tauri-apps/api/core";

export interface SecurityEvent {
  id: number;
  eventType: string;
  severity: string;
  title: string;
  description: string;
  occurredAt: string;
  reviewed: boolean;
}
export interface Incident {
  id: number;
  severity: string;
  status: "open" | "investigating" | "resolved";
  title: string;
  description: string;
  eventCount: number;
  firstDetectedAt: string;
  lastDetectedAt: string;
}
export interface SecurityScore {
  score: number;
  openIncidentCount: number;
  criticalIncidentCount: number;
}
export interface FileEvent { id: number; eventKind: string; filePath: string; severity: string; occurredAt: string; }

export const listSecurityEvents = () => invoke<SecurityEvent[]>("list_security_events");
export const markSecurityEventReviewed = (id: number) =>
  invoke<void>("mark_security_event_reviewed", { id });
export const listIncidents = (severity?: string, status?: string) =>
  invoke<Incident[]>("list_incidents", { severity: severity || null, status: status || null });
export const updateIncidentStatus = (id: number, status: Incident["status"]) =>
  invoke<void>("update_incident_status", { id, status });
export const getSecurityScore = () => invoke<SecurityScore>("get_security_score");
export const listFileEventsFiltered = (filters: { query?: string; severity?: string; from?: string; to?: string; sortDesc?: boolean }) => invoke<FileEvent[]>("list_file_events_filtered", { query: filters.query || null, severity: filters.severity || null, from: filters.from || null, to: filters.to || null, sortDesc: filters.sortDesc ?? true });
