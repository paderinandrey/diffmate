ALTER TABLE jobs ADD COLUMN origin TEXT NOT NULL DEFAULT 'manual'
  CHECK (origin IN ('manual', 'automatic'));
ALTER TABLE jobs ADD COLUMN force INTEGER NOT NULL DEFAULT 0 CHECK (force IN (0, 1));
ALTER TABLE jobs ADD COLUMN stop_reason TEXT
  CHECK (stop_reason IN ('timeout', 'shutdown', 'user_cancel'));
ALTER TABLE jobs ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0 CHECK (retry_count >= 0);
ALTER TABLE jobs ADD COLUMN max_retries INTEGER CHECK (max_retries >= 0);
ALTER TABLE jobs ADD COLUMN next_attempt_at INTEGER;
ALTER TABLE jobs ADD COLUMN result_run_id INTEGER REFERENCES runs (id);
ALTER TABLE targets ADD COLUMN head_sha TEXT;
ALTER TABLE targets ADD COLUMN head_checked_at INTEGER;
CREATE INDEX jobs_state ON jobs (state, next_attempt_at);
