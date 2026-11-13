---
phase: 04-review-and-publish
plan: 09
subsystem: tui
tags: [tokio, rusqlite, ratatui, sync, publish, reconcile]

requires:
  - phase: 04-review-and-publish
    provides: "execute_sync later() retry, y enqueue_reconcile, and the crash publication harness from plans 04-07 and 04-08"
provides:
  - "While tui holds the runner lock, serve runs a due sync through execute_sync and then later reconcile and publish jobs."
affects: [04-review-and-publish, tui, publish]
actuals:
  tokens: 1560
  tasks: 2
  commits: 2
plan_head_before: 4b8545152695f2fd920c657e6a62a2dcfe5304dc
plan_head_after: 9693fd66c944666811f83d17e92ff94bc15981c3
tech-stack:
  added: []
  patterns:
    - "The tui sync arm calls start_job and execute_sync, then loops. It does not return, and it does not call create_review."
key-files:
  created: []
  modified:
    - src/tui/runner.rs
    - src/publish/tests/crash/keys.rs
    - src/publish/tests/crash/gh.rs
key-decisions:
  - "A due sync is served with execute_sync and the loop continues; next_attempt_at is left to later()."
  - "A second POST means another create_review of the same publication. The continuation fixture uses another draft only so its publish job sorts after the sync."
  - "The crash remote's create_review check prefers a sending publication when more than one row exists."
patterns-established:
  - "Unknown service kinds still return from serve. Sync, publish, and reconcile do not."
requirements-completed: [PUB-15, PUB-03]
coverage:
  - id: D1
    description: "Pressing y with an older due sync retries that sync without a POST and still reconciles the unknown publication."
    requirement: PUB-03
    verification:
      - kind: unit
        ref: "src/publish/tests/crash/keys.rs#y_runs_reconcile_after_a_due_sync"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/abandon.rs#b_safe_choice_leaves_unknown"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/abandon.rs#b_abandon_sets_confirmed_by_me"
        status: pass
    human_judgment: false
  - id: D2
    description: "A failed sync served by tui stays submitted, leaves the inline thread unknown, and is not a create_review."
    requirement: PUB-15
    verification:
      - kind: unit
        ref: "src/publish/tests/crash/keys.rs#y_runs_reconcile_after_a_due_sync"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/keys.rs#serve_runs_a_due_sync_then_a_newer_publish"
        status: pass
    human_judgment: false
  - id: D3
    description: "After that sync, a newer publish still runs and posts once. The pre-seeded publication stays submitted."
    requirement: PUB-15
    verification:
      - kind: unit
        ref: "src/publish/tests/crash/keys.rs#serve_runs_a_due_sync_then_a_newer_publish"
        status: pass
    human_judgment: false
duration: 22 min
completed: 2026-11-11
status: complete
---

# Phase 4 Plan 9: Tui Sync Service Summary

**tui serve runs a due comment sync through execute_sync, then still reconciles and publishes later jobs without a second review POST.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-11-11T08:26:16Z
- **Completed:** 2026-11-11T08:48:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `serve` matches `sync` with `start_job` and `execute_sync`, then returns to `next_service`. The `_` arm still returns on an unknown kind.
- Pressing `y` with an older due sync leaves that sync queued with `next_attempt_at` set, posts nothing, keeps the unknown publication unknown, and succeeds the reconcile job.
- A newer publish queued after that sync still runs. `posts()` is 1, the pre-seeded publication stays submitted, and its finding stays `thread_status` unknown.

## Task Commits

Each task was committed atomically:

1. **Task 1: Serve a due sync, then y's reconcile** - `2ad0e0d` (fix)
2. **Task 2: A newer publish still runs after sync** - `9693fd6` (fix)

**Plan metadata:** pending docs commit

## Files Created/Modified

- `src/tui/runner.rs` - sync arm in the tui service loop
- `src/publish/tests/crash/keys.rs` - `y_runs_reconcile_after_a_due_sync` and `serve_runs_a_due_sync_then_a_newer_publish`
- `src/publish/tests/crash/gh.rs` - create_review state check when more than one publication row exists

## Decisions Made

- The sync arm does not call `create_review`, `execute_publish`, or `execute_reconcile`, and it does not clear `next_attempt_at`. A failed sync stays on the existing `later()` path.
- "Second" stays a second POST of the same publication. The publish-continuation fixture uses another draft only so its job id sorts after the sync job.
- The continuation test calls `crate::tui::serve`, the `cfg(test)` re-export. `runner` is private to the tui module.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Crash remote create_review assumed one publication row**
- **Found during:** Task 2 (A newer publish still runs after sync)
- **Issue:** `SELECT state FROM publications` returned the pre-seeded submitted row, so `create_review` panicked with `submitted != sending` before the newer publish could finish.
- **Fix:** The check now reads one state ordered so a `sending` row wins. A single non-sending publication still fails the assertion.
- **Files modified:** `src/publish/tests/crash/gh.rs`
- **Verification:** `serve_runs_a_due_sync_then_a_newer_publish` failed on the old query and passed after the change. The plan filter of four tests passed.
- **Committed in:** `9693fd6` (Task 2 commit)

**2. [Rule 3 - Blocking] Test cannot name the private runner module**
- **Found during:** Task 2 (A newer publish still runs after sync)
- **Issue:** The plan names `crate::tui::runner::serve`. `runner` is private, so that path does not compile.
- **Fix:** The test calls `crate::tui::serve`, the existing test re-export of the same function.
- **Files modified:** `src/publish/tests/crash/keys.rs`
- **Verification:** `serve_runs_a_due_sync_then_a_newer_publish` passed.
- **Committed in:** `9693fd6` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both were required for the continuation test to compile and observe one POST. No new crate, migration, or service behavior beyond the sync arm.

## Issues Encountered

The first Task 2 commit read `/tmp/msg.txt` after the pre-commit hook, and another process had replaced that file. The commit was amended before push to `9693fd6` with subject `fix(tui): run a newer publish after sync`. The tree did not change.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plans 04-01 through 04-10 are implemented. Phase 04 is ready for verification. Open, re-run, discard, and abandon were not changed.

## Self-Check: PASSED

- FOUND: src/tui/runner.rs
- FOUND: src/publish/tests/crash/keys.rs
- FOUND: src/publish/tests/crash/gh.rs
- FOUND: 2ad0e0d
- FOUND: 9693fd6

## TDD Gate Compliance

Not applicable. `workflow.tdd_mode` is false and this plan is `type: execute`, not `type: tdd`.

---
*Phase: 04-review-and-publish*
*Completed: 2026-11-11*
