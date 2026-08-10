/** Panels that mirror live OS state and must stay close to real time. */
export const LIVE_REFETCH_MS = 2000;
/** Panels where a slower cadence is enough and keeps the query load low. */
export const BACKGROUND_REFETCH_MS = 5000;

const INCIDENTS = ["incidents"] as const;
const PROCESSES = ["processes"] as const;

/**
 * Every React Query key in the app. Invalidation matches on key prefixes, so the derived
 * keys below intentionally extend their base key.
 */
export const queryKeys = {
  appStatus: ["app-status"],
  systemSnapshot: ["snapshot"],
  processes: (query: string, sortBy: string) => [...PROCESSES, query, sortBy],
  resourceAlertProcesses: [...PROCESSES, "resource-alerts"],
  monitoredPaths: ["monitored-paths"],
  fileEvents: ["file-events"],
  securityEvents: ["security-events"],
  incidents: INCIDENTS,
  filteredIncidents: (severity: string, status: string) => [...INCIDENTS, severity, status],
  incidentTimeline: (incidentId: number | undefined) => ["incident-timeline", incidentId],
  securityReport: ["security-report"],
  settings: ["settings"],
  detectionPolicy: ["detection-policy"],
  allowlist: ["allowlist"],
  allowlistAudit: ["allowlist-audit"],
  notifications: ["notifications"],
} as const;
