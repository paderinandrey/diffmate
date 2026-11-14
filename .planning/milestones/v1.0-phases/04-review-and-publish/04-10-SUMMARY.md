---
phase: 04-review-and-publish
plan: 10
subsystem: status
tags: [rusqlite, sqlite, ratatui, status, queue]

requires:
  - phase: 04-review-and-publish
    provides: "shared status SQL, queue_rows newest-target-first, and the latest-run formatter from plans 04-01 and 04-03"
provides:
  - "Queue rows and diffmate status use the latest review job, not a later publish, reconcile, or sync job."
affects: [04-review-and-publish, status, tui]
actuals:
  tokens: 1524
  tasks: 2
  commits: 2
plan_head_before: 5ed8ea172e3c0580be2a93dcb8827222e408b806
plan_head_after: 1464ef68a13d1f07c86ac462d384e80ce9c17415
tech-stack:
  added: []
  patterns:
    - "The shared status jobs subquery is max(id) where kind is review. The runs join, publication subquery, and both ORDER BY clauses are unchanged."
key-files:
  created: []
  modified:
    - src/store/status.rs
    - src/status/tests/stored.rs
    - src/tui/tests/queue.rs
key-decisions:
  - "Status and the queue join the latest review job, so a later publish or sync does not replace the review run."
patterns-established:
  - "Retry text follows that same review job. A succeeded review does not gain a retry phrase from a later queued sync."
requirements-completed: [PUB-01]
coverage:
  - id: D1
    description: "After a succeeded publish and a queued sync, target_statuses and queue_rows still show succeeded (findings: 1), and status latest does not add a retry phrase."
    requirement: PUB-01
    verification:
      - kind: unit
        ref: "src/status/tests/stored.rs#later_publish_and_sync_keep_the_review_run"
        status: pass
      - kind: unit
        ref: "src/status/tests/stored.rs#a_queued_crash_retry_appends_its_persisted_budget_to_latest"
        status: pass
    human_judgment: false
  - id: D2
    description: "The drawn queue keeps the review run and submitted on o/r#1, keeps succeeded (findings: 1) and none on o/r#2 above it, and q inserts no job."
    requirement: PUB-01
    verification:
      - kind: unit
        ref: "src/tui/tests/queue.rs#queue_frame_keeps_the_review_run_after_sync"
        status: pass
      - kind: unit
        ref: "src/tui/tests/queue.rs#queue_shows_the_target_and_quit_writes_nothing"
        status: pass
      - kind: unit
        ref: "src/tui/tests/queue.rs#queue_is_newest_first_and_arrows_move_the_selection"
        status: pass
    human_judgment: false
duration: 15 min
completed: 2026-11-11
status: complete
---

# Phase 4 Plan 10: Latest Review Run Summary

**The queue and diffmate status keep the latest review run after a later publish or sync job, including succeeded (findings: N).**

## Performance

- **Duration:** 15 min
- **Started:** 2026-11-11T07:57:57Z
- **Completed:** 2026-11-11T08:13:24Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- The shared status jobs subquery is `max(id)` where `kind = 'review'`. A later publish or sync job with no run no longer replaces the review run.
- `later_publish_and_sync_keep_the_review_run` sees `succeeded (findings: 1)` from `target_statuses` and `queue_rows` after those inserts. Publication, draft head, and known head stay the same. The retry phrase is not added because the review job succeeded.
- The queue frame still draws that review run and `submitted` on `o/r#1`, with `o/r#2` above it, and `q` does not insert a job.

## Task Commits

Each task was committed atomically:

1. **Task 1: Latest run stays the review after publish and sync** - `a2356a1` (fix)
2. **Task 2: The drawn queue shows that review run** - `1464ef6` (test)

## Files Created/Modified

- `src/store/status.rs` - jobs subquery requires `kind = 'review'`. The runs join is unchanged.
- `src/status/tests/stored.rs` - `later_publish_and_sync_keep_the_review_run`
- `src/tui/tests/queue.rs` - `queue_frame_keeps_the_review_run_after_sync`

## Decisions Made

- Retry text follows the review job because `RetryView` reads the same `j` columns. A later queued sync does not append a retry phrase to a succeeded review.
- No second join was added for the newest publish or sync job. `latest()` in `src/store/status/latest.rs` was not changed. `user_version` was not bumped.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

`state.advance-plan` was not run. The position is still Plan 9 of 10 because 04-09 has no summary. Advancing would record plan 9 as finished. `state.update-progress`, `state.record-metric`, `state.add-decision`, and `state.record-session` did run.

## User Setup Required

None - no external service configuration required.

## Verification

- `mise run --force test -- -E 'test(later_publish_and_sync_keep_the_review_run) | test(stored::) | test(queue_frame_keeps_the_review_run_after_sync) | test(queue::)' --no-tests=fail` exited 0 (9 passed).
- `mise run lint` exited 0.
- Each task commit hook ran the full nextest suite: 492 passed, then 493 passed. Clippy with warnings denied passed in both hooks.
- Tracer verify was re-run before the queue test and passed.

## Next Phase Readiness

Plan 04-09 still has no summary. This plan does not start it. Schema `user_version` stays 5. No new crate.

## Self-Check: PASSED

- FOUND: `a2356a1`
- FOUND: `1464ef6`
- FOUND: `src/store/status.rs`
- FOUND: `src/status/tests/stored.rs`
- FOUND: `src/tui/tests/queue.rs`

## Known Stubs

None.

*Phase: 04-review-and-publish*
*Completed: 2026-11-11*
