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

export const listSecurityEvents = () => invoke<SecurityEvent[]>("list_security_events");
export const markSecurityEventReviewed = (id: number) =>
  invoke<void>("mark_security_event_reviewed", { id });
