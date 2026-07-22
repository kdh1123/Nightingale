CREATE TABLE IF NOT EXISTS incidents (
  id INTEGER PRIMARY KEY,
  correlation_key TEXT NOT NULL,
  severity TEXT NOT NULL CHECK (severity IN ('info', 'low', 'medium', 'high', 'critical')),
  status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'investigating', 'resolved')),
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  event_count INTEGER NOT NULL DEFAULT 1,
  first_detected_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_detected_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  resolved_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_incidents_latest ON incidents(status, last_detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_incidents_correlation ON incidents(correlation_key, status, last_detected_at DESC);
CREATE TABLE IF NOT EXISTS incident_events (
  incident_id INTEGER NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
  security_event_id INTEGER NOT NULL REFERENCES security_events(id) ON DELETE CASCADE,
  PRIMARY KEY (incident_id, security_event_id)
);
INSERT OR IGNORE INTO schema_metadata (version) VALUES (3);
