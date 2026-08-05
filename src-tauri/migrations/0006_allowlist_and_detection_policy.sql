CREATE TABLE IF NOT EXISTS allowlist_entries (
  id INTEGER PRIMARY KEY,
  entry_type TEXT NOT NULL CHECK (entry_type IN ('path', 'extension')),
  value TEXT NOT NULL,
  expires_at TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(entry_type, value)
);
CREATE INDEX IF NOT EXISTS idx_allowlist_active ON allowlist_entries(entry_type, expires_at);

CREATE TABLE IF NOT EXISTS allowlist_audit (
  id INTEGER PRIMARY KEY,
  entry_id INTEGER,
  action TEXT NOT NULL CHECK (action IN ('added', 'removed')),
  entry_type TEXT NOT NULL,
  value TEXT NOT NULL,
  occurred_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_allowlist_audit_recent ON allowlist_audit(occurred_at DESC);

INSERT OR IGNORE INTO security_policies (version, policy_json) VALUES
  (1, '{"version":1,"sensitivity":"medium","features":{"massFileChanges":true,"massExtensionChanges":true,"suspiciousFileActivity":true,"integrityChanges":true,"phishingUrlAnalysis":true,"systemAnomalyDetection":true},"monitoredPaths":[],"excludedPaths":[]}');

INSERT OR IGNORE INTO schema_metadata (version) VALUES (6);
