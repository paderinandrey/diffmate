---
status: testing
phase: 04-review-and-publish
source: [04-VERIFICATION.md]
started: 2026-11-11T09:09:05Z
updated: 2026-11-11T09:09:05Z
---

## Current Test

number: 1
name: Accept or reject the judgment that the tui sync arm never calls create_review, and that a failed sync retries the same job while the publication stays submitted.
expected: |
  serve matches sync, calls execute_sync, and loops. A failed read requeues that job with next_attempt_at and does not POST. Tests y_runs_reconcile_after_a_due_sync and serve_runs_a_due_sync_then_a_newer_publish already passed (posts 0 on the y path, posts 1 only for the later publish).
awaiting: user response

## Tests

### 1. Accept or reject the judgment that the tui sync arm never calls create_review, and that a failed sync retries the same job while the publication stays submitted.
expected: serve matches sync, calls execute_sync, and loops. A failed read requeues that job with next_attempt_at and does not POST. Tests y_runs_reconcile_after_a_due_sync and serve_runs_a_due_sync_then_a_newer_publish already passed (posts 0 on the y path, posts 1 only for the later publish).
result: [pending]

### 2. Accept or reject the judgment that a missing publication marker does not move unknown to rejected.
expected: execute_reconcile on Ok(None) calls write::succeed and leaves the publication unknown. y_missing_marker_stays_unknown and reconcile_missing_marker_stays_unknown passed with state unknown, reconcile succeeded, and posts 0.
result: [pending]

### 3. Accept or reject the judgment that a later publish, reconcile, or sync job is not the latest review run on the queue or on diffmate status.
expected: The shared jobs subquery is max(id) where kind = 'review'. later_publish_and_sync_keep_the_review_run and queue_frame_keeps_the_review_run_after_sync passed with succeeded (findings: 1) after a later publish and sync. Publication, freshness, and newest-target order were unchanged.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
