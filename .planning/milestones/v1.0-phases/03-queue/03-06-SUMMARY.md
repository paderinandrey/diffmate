---
phase: 03-queue
plan: 06
subsystem: testing
tags: [watch, lock, attach, poll, rust]

requires:
  - phase: 03-queue
    provides: "watch polls inside the locked jobs loop after announce"
  - phase: 02-reliability
    provides: "runner lock, review attach, and the waiting review result path"
provides:
  - "An empty successful search does not start a second runner: watch runs the attached job and the waiting review prints the stored findings"
  - "watch started while review holds the lock waits, has not called gh search, and exits 0 on SIGTERM and SIGINT"
affects: [03-queue]
actuals:
  tokens: 915
  tasks: 2
  commits: 2
plan_head_before: fba8cfc7d1740bc7bf078896ad76de542617badb
plan_head_after: 117e5049b8ab857727ef5a88a4a65ffbe7c8a70e
tech-stack:
  added: []
  patterns:
    - "Attach tests route search prs to [] so a successful empty poll is not a second runner"
    - "The wait test reads gh.calls() after the waiting line and rejects a search prs argv"
key-files:
  created: []
  modified:
    - tests/watch/attach.rs
    - tests/watch/wait.rs
key-decisions:
  - "No production change. poll::once stays inside jobs() after the lock is announced"
  - "The shared outcome token (no_findings) is not a second copy of the findings report"
patterns-established:
  - "A successful empty poll must not make the waiting review start the agent"
requirements-completed: [QUE-09, QUE-10]
coverage:
  - id: D1
    description: "A review started while watch holds the lock attaches. Watch runs that job. The waiting review prints the stored findings and does not start the agent. Watch stderr has the job line and does not print the findings report. That still holds when search prs succeeds with an empty list."
    requirement: QUE-09
    verification:
      - kind: integration
        ref: "tests/watch/attach.rs#review_while_watch_runs_prints_the_result"
        status: pass
      - kind: integration
        ref: "tests/watch/attach.rs#stopping_a_waiting_review_names_watch"
        status: pass
    human_judgment: false
  - id: D2
    description: "Watch started while review holds the lock prints the waiting line, does not exit, and has not called gh search. SIGTERM and SIGINT exit 0 and leave the one queued job."
    requirement: QUE-10
    verification:
      - kind: integration
        ref: "tests/watch/wait.rs#watch_waits_for_the_lock_and_stops_on_sigterm"
        status: pass
    human_judgment: false

duration: 10min
completed: 2026-11-09
status: complete
---

# Phase 3 Plan 06: Lock handoff with an empty poll Summary

**An empty successful poll does not start a second runner: the waiting review prints the findings, and watch does not search until it holds the lock**

## Performance

- **Duration:** 10 min
- **Started:** 2026-11-09T12:54:50Z
- **Completed:** 2026-11-09T13:04:54Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Both attach tests route `search prs` to `[]` before watch starts, so the poll succeeds instead of failing closed
- The waiting review stdout contains `o/r#2 at` and `: succeeded`, watch stderr contains `job 2 review o/r#2 succeeded` and not the findings report, and one fake claude run covers that pull request
- Watch waiting on a review-held lock is still running, has no `search prs` argv, and exits 0 for both SIGTERM and SIGINT

## Task Commits

Each task was committed atomically:

1. **Task 1: review while watch runs still prints the findings** - `db51ea7` (test)
2. **Task 2: watch waits for the lock and does not search yet** - `117e504` (test)

## Files Created/Modified

- `tests/watch/attach.rs` - Empty search route, findings stay on review stdout, one agent run for the handed-off PR
- `tests/watch/wait.rs` - No `search prs` call while the waiting line is showing, for SIGTERM and SIGINT

## Decisions Made

- No production change. `poll::once` already runs inside `jobs()`, which runs after `announce`. The wait test saw no search call, so the poll was not moved.
- Review stdout and the job line both contain the outcome token `(no_findings)`. The attach test removes the shared word `succeeded` and asserts that the remaining report is not in watch stderr, and that no line is a finding title. That fails if watch reprints the report without failing on the shared outcome token.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. The plan's hand check against live `claude` was not run. The plan names the process tests as the proof while the Anthropic 403 gate is blocked, and this execution did not call the Claude API, the Anthropic API, or live `gh`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

QUE-09 and QUE-10 hold on the Phase 2 lock and attach path with a successful empty poll in the loop. Phase 4 was not started. `.planning/STATE.md`, `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, and Phase 2 UAT were not updated.

---
*Phase: 03-queue*
*Completed: 2026-11-09*

## Self-Check: PASSED

- FOUND: tests/watch/attach.rs, tests/watch/wait.rs
- FOUND: db51ea7, 117e504 as ancestors of HEAD
- commits measured from fba8cfc7d1740bc7bf078896ad76de542617badb: 2
- plan_head_after: 117e5049b8ab857727ef5a88a4a65ffbe7c8a70e
