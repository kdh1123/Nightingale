CREATE INDEX IF NOT EXISTS idx_file_events_filter ON file_events(severity, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_file_events_path_time ON file_events(monitored_path_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_security_events_severity_time ON security_events(severity, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_incidents_severity_status_time ON incidents(severity, status, last_detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_notifications_retention ON notifications(created_at);
INSERT OR IGNORE INTO schema_metadata (version) VALUES (5);
