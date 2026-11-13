ALTER TABLE drafts ADD COLUMN limitation_included INTEGER NOT NULL DEFAULT 0
  CHECK (limitation_included IN (0, 1));
ALTER TABLE findings ADD COLUMN github_comment_id TEXT;
ALTER TABLE findings ADD COLUMN github_thread_id TEXT;
ALTER TABLE findings ADD COLUMN thread_status TEXT NOT NULL DEFAULT 'unknown'
  CHECK (thread_status IN ('resolved', 'unresolved', 'unknown', 'none'));
UPDATE findings SET thread_status = 'none' WHERE line IS NULL;

CREATE TABLE publications (
  id TEXT PRIMARY KEY CHECK (length(id) = 32),
  target_id INTEGER NOT NULL REFERENCES targets (id),
  draft_id INTEGER NOT NULL REFERENCES drafts (id),
  run_id INTEGER NOT NULL REFERENCES runs (id),
  state TEXT NOT NULL CHECK (state IN
    ('pending', 'sending', 'submitted', 'rejected', 'unknown', 'abandoned')),
  event TEXT NOT NULL CHECK (event IN ('APPROVE', 'REQUEST_CHANGES', 'COMMENT')),
  confirmed_sha TEXT NOT NULL,
  snapshot TEXT NOT NULL,
  reason TEXT,
  github_review_id TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

ALTER TABLE jobs ADD COLUMN publication_id TEXT;

CREATE INDEX publications_target ON publications (target_id, state);
CREATE INDEX publications_draft ON publications (draft_id, state);
CREATE UNIQUE INDEX jobs_publication ON jobs (publication_id)
  WHERE kind = 'publish' AND publication_id IS NOT NULL;
