---
phase: 03-queue
plan: 04
subsystem: notifications
tags: [osascript, watch, notify, applescript, rust]

requires:
  - phase: 03-queue
    provides: "Watch runs due jobs, prints the outcome from src/watch/job.rs, and plan 03-08 classifies HTTP 429 as transient"
provides:
  - "An automatic success runs osascript with title diffmate: draft ready and body the PR label"
  - "An automatic give-up, including recovery, runs osascript with title diffmate: review failed and body label plus reason"
  - "A retry, a shutdown requeue, a manual job, and a failed osascript do not change the job row"
affects: [03-queue, notifications]
actuals:
  tokens: 5805
  tasks: 2
  commits: 2
plan_head_before: 6ffa4c8d2cecde3d79972c591a5d6867180b46d7
plan_head_after: 4718c1af631f6de0af03ed9676a284766b036f89
commits: 2
tech-stack:
  added: []
  patterns:
    - "osascript is Command::new with args -e and one script, env_clear, then InheritedEnv::tool_env"
    - "Title and body go through render::sanitize, then backslash and quote escaping, then newline flattening"
    - "deliver errors are one notification-failed log line and do not touch the store"
key-files:
  created:
    - src/notify.rs
    - src/notify/script.rs
    - src/notify/recovery.rs
    - src/notify/tests.rs
    - tests/watch/notify.rs
    - tests/watch/notify/support.rs
    - tests/watch/notify/ready.rs
    - tests/watch/notify/give_up.rs
    - tests/watch/notify/recovery.rs
  modified:
    - src/main.rs
    - src/watch/job.rs
    - src/watch.rs
    - src/store/jobs.rs
    - tests/watch/poll.rs
key-decisions:
  - "The notify call sits in src/watch/job.rs after schedule_after_run and the outcome line, because that is where the watch loop moved"
  - "Recovery give-up is seeded as a running automatic job and observed through watch, which the plan allows when a live crash is awkward"
  - "A terminal Cancelled automatic job uses the stored failure_reason as the give-up body"
patterns-established:
  - "Notice::Ready and Notice::GaveUp are decided in src/notify.rs; watch only delivers and logs"
  - "Fake osascript is an executable on the sandbox PATH that records argv and exits with a status file"
requirements-completed: [QUE-01, QUE-08]
coverage:
  - id: D1
    description: "An automatic success notifies with title diffmate: draft ready and body the PR label. The finding body is not in the script or watch stderr."
    requirement: QUE-01
    verification:
      - kind: integration
        ref: "tests/watch/notify/ready.rs#a_ready_automatic_draft_runs_osascript"
        status: pass
      - kind: unit
        ref: "src/notify/tests.rs#ready_and_give_up_titles_are_fixed"
        status: pass
    human_judgment: false
  - id: D2
    description: "A non-zero osascript status is logged and the job stays succeeded with retry_count unchanged."
    requirement: QUE-01
    verification:
      - kind: integration
        ref: "tests/watch/notify/ready.rs#a_failed_osascript_still_succeeds_the_job"
        status: pass
    human_judgment: false
  - id: D3
    description: "An automatic give-up with max_retries 0 and api_error_status 429 notifies with title diffmate: review failed and the envelope reason. The job is failed."
    requirement: QUE-08
    verification:
      - kind: integration
        ref: "tests/watch/notify/give_up.rs#an_automatic_give_up_names_the_reason"
        status: pass
      - kind: unit
        ref: "src/recovery/rules/tests/transient.rs#a_server_error_with_no_budget_names_the_give_up"
        status: pass
    human_judgment: false
  - id: D4
    description: "A retry, a shutdown requeue, and a manual job do not notify. Recovery of a manual crash does not notify. Recovery of an automatic crash with no retries left does."
    requirement: QUE-08
    verification:
      - kind: integration
        ref: "tests/watch/notify/give_up.rs#sigterm_during_an_automatic_run_does_not_notify"
        status: pass
      - kind: integration
        ref: "tests/watch/notify/ready.rs#a_manual_job_records_no_osascript_argv"
        status: pass
      - kind: integration
        ref: "tests/watch/notify/recovery.rs#recovery_notifies_an_automatic_give_up_and_skips_a_manual_crash"
        status: pass
      - kind: unit
        ref: "src/recovery/rules/tests/transient.rs#auth_and_other_agent_errors_fail_immediately"
        status: pass
    human_judgment: false
  - id: D5
    description: "Notification Center shows the ready and give-up titles."
    requirement: QUE-01
    verification: []
    human_judgment: true
    rationale: "Process tests prove the osascript argv. Only the owner can see Notification Center, and that look was not run."
duration: 19min
completed: 2026-11-09
status: complete
---

# Phase 3 Plan 04: macOS Notifications Summary

**Automatic success and give-up call osascript with sanitized titles, and a failed notifier leaves the job unchanged**

## Performance

- **Duration:** 19 min
- **Started:** 2026-11-09T12:18:39Z
- **Completed:** 2026-11-09T12:37:51Z
- **Tasks:** 2
- **Files modified:** 14

## Accomplishments

- `osascript` is one argument vector: the program, `-e`, and one `display notification` script. The environment is cleared and then replaced with `InheritedEnv::tool_env`, the same set git and gh use.
- An automatic success notifies with title `diffmate: draft ready` and body the PR label. An automatic give-up notifies with title `diffmate: review failed` and body `{label}: {reason}`. Title and body pass through `render::sanitize`, then backslash and double-quote escaping, then CR, LF, and Unicode line separators become a space.
- A non-zero osascript status prints one `notification-failed` line. The job state and `retry_count` stay as the runner left them. A retry, a shutdown requeue, and a manual job do not notify. Recovery that fails an automatic job notifies; a requeue, including shutdown, and a manual crash do not.

## Task Commits

Each task was committed atomically:

1. **Task 1: A ready automatic draft runs osascript and a failed osascript still succeeds the job** - `1587d12` (feat)
2. **Task 2: Give-up notification names the reason, including after recovery** - `4718c1a` (feat)

## Files Created/Modified

- `src/notify.rs` - `decide`, `for_result`, and `deliver`
- `src/notify/script.rs` - AppleScript string from sanitized text
- `src/notify/recovery.rs` - Give-up notices for recovery lines that start with `failed` and are automatic
- `src/notify/tests.rs` - Fixed titles, quote and newline escaping, and which results notify
- `src/watch/job.rs` - Deliver after `schedule_after_run` and the outcome line
- `src/watch.rs` - Deliver recovery give-ups after the recovery lines
- `src/store/jobs.rs` - `job_failure_reason` and `job_label`
- `src/main.rs` - `mod notify`
- `tests/watch/poll.rs` - Declares `tests/watch/notify.rs` without editing `tests/watch.rs`
- `tests/watch/notify/support.rs` - Fake osascript on the sandbox PATH
- `tests/watch/notify/ready.rs` - Ready draft, exit 1, and manual silence
- `tests/watch/notify/give_up.rs` - HTTP 429 give-up and SIGTERM silence
- `tests/watch/notify/recovery.rs` - Seeded automatic crash notifies; manual crash does not

## Decisions Made

- The plan still named `src/watch/run.rs` as the call site. `schedule_after_run` and the outcome line now live in `src/watch/job.rs`, so the notify call is there, after both. `src/watch/run.rs` was not edited and is still 69 lines.
- Recovery is covered by marking a running automatic job in SQLite and starting watch, which the plan allows when a live crash is awkward. The give-up itself is a real `debug enqueue --automatic` run with `api_error_status` 429 and `max_retries = 0`.
- An automatic `Cancelled` result whose `JobEnd` is `Failed` has no detail message on the result. `for_result` loads `jobs.failure_reason` for that case only.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Flatten Unicode line separators in the AppleScript string**
- **Found during:** Task 1 (ready draft notification)
- **Issue:** `render::sanitize` keeps U+2028 and U+2029. Either can start a second AppleScript statement after the quote and backslash escaping. CR is already removed by sanitize, so the later CR replacement does not see it.
- **Fix:** After sanitize, escape backslash and double quote, and replace LF, CR, U+2028, and U+2029 with a space.
- **Files modified:** `src/notify/script.rs`, `src/notify/tests.rs`
- **Verification:** `notify::tests::a_quote_newline_or_backslash_cannot_break_the_script` passed. The script is one line and contains the escaped quote and backslash.
- **Committed in:** `1587d12`

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** The call site moved with the watch loop, as the dispatch required. Recovery rules were not changed. No new crate.

## Issues Encountered

None. The optional Notification Center look was skipped. It is not required to prove the argv, and it was not used to touch the Anthropic HTTP 403 gate.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Watch notifies on a ready automatic draft and on the last failed automatic attempt. `tests/watch.rs` was not edited. Later plans in this phase can still change that file. Status 401 and 403 stay terminal. Status 429 stays transient, so `max_retries = 0` is a give-up.

---
*Phase: 03-queue*
*Completed: 2026-11-09*

## Self-Check: PASSED

- FOUND: src/notify.rs, src/notify/script.rs, src/notify/recovery.rs, src/notify/tests.rs, src/watch/job.rs, src/watch.rs, src/watch/run.rs (69 lines), src/store/jobs.rs, tests/watch/notify.rs, tests/watch/notify/support.rs, tests/watch/notify/ready.rs, tests/watch/notify/give_up.rs, tests/watch/notify/recovery.rs
- FOUND: 1587d12, 4718c1a as ancestors of HEAD
- commits measured from 6ffa4c8d2cecde3d79972c591a5d6867180b46d7: 2
- plan_head_after: 4718c1af631f6de0af03ed9676a284766b036f89
