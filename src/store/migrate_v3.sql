ALTER TABLE targets ADD COLUMN request_seen INTEGER NOT NULL DEFAULT 0
  CHECK (request_seen IN (0, 1));
ALTER TABLE targets ADD COLUMN queued_head TEXT;
