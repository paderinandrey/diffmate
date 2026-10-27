CREATE TABLE targets (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  owner TEXT NOT NULL,
  repo TEXT NOT NULL,
  number INTEGER NOT NULL CHECK (number > 0),
  tracking TEXT NOT NULL DEFAULT 'untracked' CHECK (tracking IN ('untracked', 'tracked')),
  created_at INTEGER NOT NULL,
  UNIQUE (owner, repo, number)
);
CREATE TABLE jobs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  kind TEXT NOT NULL CHECK (kind IN ('review', 'publish', 'reconcile', 'sync')),
  target_id INTEGER NOT NULL REFERENCES targets (id),
  state TEXT NOT NULL CHECK (state IN
    ('queued', 'running', 'succeeded', 'failed', 'needs_confirmation', 'cancelled')),
  enqueued_by_pid INTEGER NOT NULL,
  enqueued_by_started_at INTEGER NOT NULL,
  failure_reason TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  target_id INTEGER NOT NULL REFERENCES targets (id),
  job_id INTEGER NOT NULL REFERENCES jobs (id),
  state TEXT NOT NULL CHECK (state IN ('queued', 'running', 'succeeded', 'failed')),
  result TEXT CHECK (result IN ('findings', 'no_findings')),
  failure_kind TEXT CHECK (failure_kind IN
    ('cli_error', 'agent_error', 'timeout', 'invalid_output', 'interrupted')),
  interrupted_cause TEXT CHECK (interrupted_cause IN ('shutdown', 'user_cancel', 'crash')),
  failure_detail TEXT,
  head_sha TEXT NOT NULL,
  merge_base_sha TEXT NOT NULL,
  base_tip_sha TEXT NOT NULL,
  base_ref TEXT NOT NULL,
  agent_pid INTEGER,
  agent_pgid INTEGER,
  total_cost_usd REAL,
  duration_ms INTEGER,
  warnings TEXT NOT NULL DEFAULT '[]',
  created_at INTEGER NOT NULL,
  finished_at INTEGER,
  CHECK ((state = 'succeeded') = (result IS NOT NULL)),
  CHECK ((state = 'failed') = (failure_kind IS NOT NULL)),
  CHECK (interrupted_cause IS NULL OR failure_kind = 'interrupted')
);
CREATE TABLE drafts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id INTEGER NOT NULL UNIQUE REFERENCES runs (id),
  original_output TEXT NOT NULL,
  limitation TEXT,
  future_improvements TEXT NOT NULL DEFAULT '[]',
  created_at INTEGER NOT NULL
);
CREATE TABLE findings (
  id TEXT PRIMARY KEY,
  run_id INTEGER NOT NULL REFERENCES runs (id),
  draft_id INTEGER NOT NULL REFERENCES drafts (id),
  ordinal INTEGER NOT NULL,
  severity TEXT NOT NULL CHECK (severity IN ('high', 'medium', 'low')),
  path TEXT,
  line INTEGER CHECK (line >= 1),
  side TEXT CHECK (side IN ('LEFT', 'RIGHT')),
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  previous_finding_id TEXT REFERENCES findings (id),
  decision TEXT NOT NULL DEFAULT 'undecided'
    CHECK (decision IN ('undecided', 'kept', 'edited', 'dropped')),
  UNIQUE (run_id, ordinal),
  CHECK ((line IS NULL) = (side IS NULL))
);
CREATE INDEX jobs_target ON jobs (target_id);
CREATE INDEX runs_target ON runs (target_id);
