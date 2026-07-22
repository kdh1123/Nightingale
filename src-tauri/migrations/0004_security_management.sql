CREATE TABLE IF NOT EXISTS application_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  monitoring_enabled INTEGER NOT NULL DEFAULT 1,
  threat_detection_enabled INTEGER NOT NULL DEFAULT 1,
  auto_baseline_refresh INTEGER NOT NULL DEFAULT 0,
  security_score_enabled INTEGER NOT NULL DEFAULT 1,
  log_retention_days INTEGER NOT NULL DEFAULT 90 CHECK (log_retention_days BETWEEN 1 AND 3650),
  ui_theme TEXT NOT NULL DEFAULT 'system' CHECK (ui_theme IN ('system', 'light', 'dark')),
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT OR IGNORE INTO application_settings (id) VALUES (1);
CREATE TABLE IF NOT EXISTS notifications (
  id INTEGER PRIMARY KEY,
  incident_id INTEGER REFERENCES incidents(id) ON DELETE SET NULL,
  severity TEXT NOT NULL CHECK (severity IN ('info', 'low', 'medium', 'high', 'critical')),
  title TEXT NOT NULL,
  message TEXT NOT NULL,
  read INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_notifications_recent ON notifications(read, created_at DESC);
CREATE TABLE IF NOT EXISTS report_history (
  id INTEGER PRIMARY KEY,
  report_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT OR IGNORE INTO schema_metadata (version) VALUES (4);
