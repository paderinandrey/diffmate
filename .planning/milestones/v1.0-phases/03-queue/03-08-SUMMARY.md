---
phase: 03-queue
plan: 08
subsystem: queue
tags: [retry, agent_error, rust, failure-classification]
requires:
  - phase: 02-reliability
    provides: "The automatic retry budget, after_failure, and FailureDetail.api_error_status"
provides:
  - "Failure::TransientAgent shares the timeout and crash retry budget"
  - "is_transient classifies 408, 429, and 500-599 as retryable and keeps 401 and 403 terminal"
affects: [03-queue, retry, give-up notification]
actuals:
  tokens: 3423
  tasks: 1
  commits: 1
plan_head_before: 5d082c3bb225e2d4a01417f1d979f189bfdf7fc8
plan_head_after: c4390e8ba842678372e0e9526451d548b1dbddde
commits: 1
tech-stack:
  added: []
  patterns:
    - "Crash, Timeout, and TransientAgent share one automatic after_failure arm"
    - "A present non-null api_error_status is classified only as a JSON number; text tokens are scanned only when the status is missing or null"
key-files:
  created:
    - src/recovery/rules/transient.rs
    - src/recovery/rules/tests/transient.rs
    - src/recovery/rules/tests/tokens.rs
    - src/recovery/rules/tests/support.rs
    - src/runner/outcome/tests/transient.rs
  modified:
    - src/recovery/rules/failure.rs
    - src/recovery/rules.rs
    - src/recovery/rules/tests.rs
    - src/runner/outcome/failed.rs
    - src/runner/outcome/tests.rs
key-decisions:
  - "Failure::TransientAgent text() stays agent_error so the stored kind and the give-up sentence do not invent a new failure name"
  - "401 and 403 are terminal even when the message contains a retry token. A non-numeric status, including the string 429, does not fall through to the token scan"
  - "The token scan is an ASCII case-insensitive substring match on message, compact errors JSON, subtype, and terminal_reason. stderr is not scanned, and api_error is not a token"
patterns-established:
  - "Retryable automatic failures are one match arm in after_failure: Crash, Timeout, and TransientAgent"
requirements-completed: [QUE-08]
coverage:
  - id: D1
    description: "A transient agent_error (status 429, and 408 or 500-599) uses the same automatic budget as a timeout: 60s, then 300s, then a give-up that names no retries left. max_retries 0 fails immediately with that sentence. Shutdown still requeues without incrementing retry_count."
    requirement: QUE-08
    verification:
      - kind: unit
        ref: "src/recovery/rules/tests/transient.rs#a_rate_limit_uses_the_same_budget_as_a_timeout"
        status: pass
      - kind: unit
        ref: "src/recovery/rules/tests/transient.rs#a_server_error_with_no_budget_names_the_give_up"
        status: pass
      - kind: unit
        ref: "src/recovery/rules/tests/transient.rs#shutdown_still_requeues_without_spending_a_retry"
        status: pass
      - kind: unit
        ref: "src/recovery/rules/tests/tokens.rs#retryable_statuses_ignore_the_message"
        status: pass
      - kind: integration
        ref: "src/runner/outcome/tests/transient.rs#a_rate_limit_requeues_and_auth_failures_do_not"
        status: pass
    human_judgment: false
  - id: D2
    description: "Status 401 and 403 stay terminal even when the message contains a retry token, and a Not logged in message with no status fails immediately. Terminal(AgentError) still fails the job."
    requirement: QUE-08
    verification:
      - kind: unit
        ref: "src/recovery/rules/tests/transient.rs#auth_and_other_agent_errors_fail_immediately"
        status: pass
      - kind: unit
        ref: "src/recovery/rules/tests/table.rs#terminal_causes_and_manual_failures_never_retry"
        status: pass
      - kind: integration
        ref: "src/runner/outcome/tests/terminal.rs#terminal_errors_and_user_cancel_end_the_job_live"
        status: pass
      - kind: integration
        ref: "src/runner/outcome/tests/transient.rs#a_rate_limit_requeues_and_auth_failures_do_not"
        status: pass
    human_judgment: false
  - id: D3
    description: "When api_error_status is missing or null, only the fixed token list is transient. api_error is not transient, and stderr is not scanned."
    requirement: QUE-08
    verification:
      - kind: unit
        ref: "src/recovery/rules/tests/tokens.rs#a_missing_status_matches_only_the_fixed_tokens"
        status: pass
    human_judgment: false
duration: 16min
completed: 2026-11-09
status: complete
---

# Phase 3 Plan 08: Transient agent_error retry Summary

**A transient agent_error for HTTP 408, 429, or 500-599 uses the existing timeout retry budget, and status 401 and 403 stay terminal.**

## Performance

- **Duration:** 16 min
- **Started:** 2026-11-09T09:48:44Z
- **Completed:** 2026-11-09T10:05:09Z
- **Tasks:** 1
- **Files modified:** 10

## Accomplishments

- `Failure::TransientAgent` shares the automatic arm with timeout and crash: the same delays (60s, then 300s), the same stored `max_retries`, and the same give-up sentence. `text()` is `agent_error`. `Terminal(AgentError)` still fails the job, and the existing table test was not changed.
- `is_transient` treats a JSON number 408, 429, or 500 through 599 as transient. A number 401 or 403 is terminal even when the message contains a retry token. Any other present non-null status, including the string `"429"`, is terminal and does not fall through to the text scan.
- When the status is missing or null, the message, the compact `errors` JSON, `subtype`, and `terminal_reason` are scanned for a fixed ASCII case-insensitive token list. `api_error` is not a token. `stderr_excerpt` is not scanned.
- `outcome/failed.rs` maps `AgentError` plus `is_transient` to `TransientAgent` before `after_failure`. A live 429 consumes retry 1 at 60s and retry 2 at 300s, then fails with a reason containing `no retries left`. Status 403 and `Not logged in` with no status fail immediately.

## Task Commits

Each task was committed atomically:

1. **Task 1: Transient agent_error uses the retry budget** - `c4390e8` (feat)

## Files Created/Modified

- `src/recovery/rules/failure.rs` - Adds `Failure::TransientAgent` whose text is `agent_error`
- `src/recovery/rules.rs` - Retries `TransientAgent` on the timeout and crash arm
- `src/recovery/rules/transient.rs` - `is_transient` for status codes and the fixed token list
- `src/runner/outcome/failed.rs` - Maps a transient `AgentError` to `TransientAgent`
- `src/recovery/rules/tests/transient.rs` - Budget, auth, zero-budget, and shutdown cases
- `src/recovery/rules/tests/tokens.rs` - Status boundaries and the token scan
- `src/recovery/rules/tests/support.rs` - Shared failure builders for those tests
- `src/runner/outcome/tests/transient.rs` - Live requeue for 429 and immediate failure for 403 and login

## Decisions Made

- A present non-null `api_error_status` decides the retry by itself. Text is consulted only when that field is missing or JSON null, so a 403 cannot be turned into a retry by a token in the message.
- The give-up reason stays `agent_error; no retries left (N of N used)` because `text()` did not change the failure name.
- The rules tests call `is_transient` directly. A separate outcome test locks the `failed.rs` match, so removing that arm fails the suite.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split the rules tests under the 100-line hook**
- **Found during:** Task 1 (Transient agent_error uses the retry budget)
- **Issue:** rustfmt left `src/recovery/rules/tests/transient.rs` at 159 lines. The hk large-files hook rejects a commit that adds more than 100 lines to one file outside `.planning/` and `Cargo.lock`.
- **Fix:** Budget cases stayed in `tests/transient.rs` (81 lines). The token and status matrix moved to `tests/tokens.rs` (59). Shared builders moved to `tests/support.rs` (32). The live outcome test is its own 94-line file.
- **Files modified:** `src/recovery/rules/tests/transient.rs`, `src/recovery/rules/tests/tokens.rs`, `src/recovery/rules/tests/support.rs`, `src/runner/outcome/tests/transient.rs`
- **Verification:** The pre-commit large-files step passed, and the filtered rules and outcome tests passed
- **Committed in:** `c4390e8` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** File layout only. The classification and the retry budget match the plan.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `mise run --force test -- -E 'test(recovery::rules) | test(runner::outcome)'` exited 0 after the final source change: 19 passed, 329 skipped.
- `mise run --force lint` exited 0: rustfmt, clippy with warnings denied, cargo-machete, gitleaks (no leaks), and actionlint.
- The task commit's pre-commit hook ran `mise run test` and exited 0: 337 passed, 11 skipped. Large-files, secrets, and lint passed. Docs and actions were skipped as sources up-to-date. `mise run test:e2e` was not run, and the Claude API was not called.

## Next Phase Readiness

- A transient `agent_error` now retries on the same head and budget as a timeout. Plan 03-04 can notify when that budget is exhausted; this plan does not send the notification.
- Phase 2 UAT was not modified and no UAT item was marked.

## Self-Check: PASSED

- FOUND: `src/recovery/rules/transient.rs`
- FOUND: `src/recovery/rules/tests/transient.rs`
- FOUND: `src/recovery/rules/tests/tokens.rs`
- FOUND: `src/recovery/rules/tests/support.rs`
- FOUND: `src/runner/outcome/tests/transient.rs`
- FOUND: `c4390e8` as an ancestor of HEAD

---
*Phase: 03-queue*
*Completed: 2026-11-09*
