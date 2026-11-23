CREATE TABLE pending_ref_cleanup (
  target_id INTEGER PRIMARY KEY REFERENCES targets (id)
);

UPDATE jobs
SET state = CASE state WHEN 'queued' THEN 'cancelled' ELSE state END,
    next_attempt_at = NULL,
    failure_reason = 'target no longer tracked'
WHERE origin = 'automatic' AND kind = 'review' AND state IN ('queued', 'running')
  AND EXISTS (SELECT 1 FROM targets
              WHERE id = jobs.target_id AND tracking = 'untracked'
                AND (request_seen = 1 OR queued_head IS NOT NULL OR jobs.head_sha IS NOT NULL));
