---
phase: 02-reliability
plan: 05
subsystem: runner
tags: [recovery, retry, sqlite, cancellation, proptest]
requires:
  - phase: 02-01
    provides: "Schema v2 retry metadata and job origins"
  - phase: 02-02
    provides: "Validated max_retries settings"
  - phase: 02-03
    provides: "Durable first stop cause and process cancellation"
provides:
  - "Atomic cause-preserving recovery and bounded retries across restarts"
  - "Due review selection in stable job order"
  - "Shared live-runner retry transitions and explicit requeue results"
  - "Automatic retries retain the first stored revision when the PR head changes"
affects: [02-06, 02-07, 02-09, 02-10, 03-queue]
actuals:
  tokens: 23136
  tasks: 3
  commits: 3
plan_head_before: ee32ee8822a8c16d68ae7e98225db0975c9ca5e8
plan_head_after: 4c290be574896d6e14a6aa111b91a7d79f0fa9cd
commits: 3
tech-stack:
  added: []
  patterns:
    - "Pure failure rules shared by recovery and live outcomes"
    - "Run failure and job transition commit in one IMMEDIATE transaction"
    - "Existing run revisions bind automatic retry jobs without another migration"
key-files:
  created:
    - src/recovery/rules.rs
    - src/recovery/rules/failure.rs
    - src/recovery/rules/tests.rs
    - src/store/recovery/resolve.rs
    - src/store/retry.rs
    - src/store/retry/queue.rs
    - src/store/retry/revision.rs
    - src/runner/cancelled.rs
    - src/runner/outcome/succeeded.rs
    - src/runner/process.rs
    - src/runner/resolve/tests.rs
  modified:
    - src/recovery.rs
    - src/store/recovery.rs
    - src/store/runs.rs
    - src/store/records.rs
    - src/runner.rs
    - src/runner/resolve.rs
    - src/runner/outcome.rs
    - src/runner/result.rs
    - src/render.rs
    - src/review.rs
    - src/config.rs
key-decisions:
  - "SPEC wins over the plan's current-head retry deferral: automatic retries retain their first stored revision"
  - "The job stores the budget for its bound revision; Phase 3 owns producer-side per-head deduplication across jobs"
  - "Keep the Phase 1 pre-run manual cancellation explanation; live run failure reasons use recovery's canonical cause text"
requirements-completed: [REL-06, REL-07]
coverage:
  - id: D1
    description: "A crashing automatic job uses two persisted retries and fails its third attempt across reopened stores"
    requirement: REL-06
    verification:
      - kind: unit
        ref: "src/recovery/tests/budget.rs#an_automatic_job_that_crashes_every_time_is_retried_within_the_budget"
        status: pass
      - kind: unit
        ref: "src/recovery/rules/tests.rs#arbitrary_failure_sequences_never_exceed_the_budget"
        status: pass
    human_judgment: false
  - id: D2
    description: "Stored shutdown, user_cancel and timeout causes survive recovery; a second recovery changes nothing"
    requirement: REL-07
    verification:
      - kind: unit
        ref: "src/recovery/tests/causes.rs#stored_causes_survive_recovery_and_a_second_recovery_is_inert"
        status: pass
      - kind: unit
        ref: "src/recovery/tests/orphan.rs#jobs_without_runs_follow_the_same_rules"
        status: pass
    human_judgment: false
  - id: D3
    description: "Queue selection returns only due review jobs, lowest ID first, and exposes the earliest future attempt"
    requirement: REL-06
    verification:
      - kind: unit
        ref: "src/store/retry/tests.rs#queue_selects_only_due_reviews_in_id_order"
        status: pass
    human_judgment: false
  - id: D4
    description: "Live cancellation and timeout use the same rules and return a requeued or failed JobEnd"
    requirement: REL-07
    verification:
      - kind: unit
        ref: "src/runner/outcome/tests/retry.rs#automatic_shutdown_and_timeout_apply_retry_rules_live"
        status: pass
      - kind: unit
        ref: "src/runner/outcome/tests/terminal.rs#terminal_errors_and_user_cancel_end_the_job_live"
        status: pass
      - kind: unit
        ref: "src/runner/tests.rs#an_automatic_shutdown_before_the_run_requeues_without_creating_a_run"
        status: pass
    human_judgment: false
  - id: D5
    description: "An automatic retry keeps its stored head and base revision when GitHub reports a different head"
    requirement: REL-06
    verification:
      - kind: unit
        ref: "src/runner/resolve/tests.rs#automatic_retries_keep_the_stored_revision_when_the_pr_head_moves"
        status: pass
    human_judgment: false
  - id: D6
    description: "Existing manual signal handling and killed-runner cleanup remain correct"
    requirement: REL-07
    verification:
      - kind: integration
        ref: "tests/recovery.rs#killed_runner_is_recovered_by_next_runner"
        status: pass
      - kind: integration
        ref: "tests/signals.rs#sigint_then_sigterm_keeps_user_cancel"
        status: pass
      - kind: unit
        ref: "src/render/tests/retry.rs#cancellation_reports_when_the_job_was_requeued"
        status: pass
    human_judgment: false
duration: approximately 21 min
completed: 2026-11-03
status: complete
---

# Phase 2 Plan 05: Durable recovery and retry rules Summary

**Recovery and live outcomes preserve stop causes, persist bounded retry schedules, and keep automatic retries on the recorded revision even after the PR head moves.**

## Accomplishments

- Added `JobOrigin`, pure `Failure` / `RetryState` / `JobNext` rules, and the 60-second first retry followed by 300-second later retries. The configured maximum is copied at the first retryable failure and survives configuration changes and store reopenings. Zero retries fails immediately with an explicit exhausted budget.
- Recovery resolves every running run and review job in one IMMEDIATE transaction. Shutdown requeues automatic work without consuming budget; user cancellation and terminal agent failures never retry. Timeout remains `timeout`, and missing causes become `interrupted: crash`. Worktree removal and pruning remain separate cleanup steps, and recovery never signals stored process IDs.
- Added due-only review selection and the earliest future retry deadline. Publication kinds and non-queued rows are excluded, and job ID defines stable order.
- Live outcome persistence applies the same rules in the transaction that finishes the run. `JobEnd` reports failure or requeue, and cancellation output explicitly says when the job is queued again. Cancellation before creating a run preserves manual behavior and requeues automatic shutdown without creating artifacts.
- Added a stored-revision lookup for automatic jobs. `resolve` reads fresh GitHub metadata and prepares the controlled clone, then reuses the first run's head, merge-base, base tip and base ref before any new fetch. Manual jobs and automatic jobs with no run resolve the current revision.

## Task Commits

1. **Task 1: Persist bounded crash recovery retries** — `75ab7be` (`feat`).
2. **Task 2: Select due reviews and preserve stop rules** — `2f59859` (`feat`).
3. **Task 3: Share live retry rules and retain reviewed heads** — `4c290be` (`feat`).

The measured range contains three task commits and 47 changed implementation, test and evidence files. Its 92,542 diff characters divided by four, rounded up, yield 23,136 estimated tokens; this is not a runtime token count. This summary is a separate metadata commit. Shared STATE, ROADMAP, REQUIREMENTS, config and runtime tracking remain owned by the orchestrator.

## Verification

Observed results:

- Task 1 filter (`test(recovery::) | binary(recovery) | binary(signals)`): **6 passed**.
- Task 2 filter (`test(recovery::) | test(store::retry::)`): **11 passed**, including the failure-sequence proptest, cause preservation, zero budget, idempotency and queue boundaries.
- Final Task 3 filter (`test(runner::) | test(render::) | binary(signals) | binary(review)`): **26 passed**.
- Final `NEXTEST_TEST_THREADS=2 mise run --force test`: **240 passed**, 11 ignored live checks, exit 0. Earlier task hooks passed the full 226-test and 235-test suites.
- Final `mise run --force lint`: rustfmt, clippy with warnings denied, cargo-machete, gitleaks and actionlint all passed.
- All three commits passed mandatory hk hooks. Task 3 reused the immediately preceding successful full-suite and lint source caches and ran secrets and large-file checks; no hooks were bypassed.
- `git diff --check`, commit ancestry, deletion and source-length checks passed. Every changed Rust file has at most 100 lines. Recovery's top-level and store modules still contain no `kill` or `signal` text. Stub and threat-surface scans found no unfinished implementation or new unplanned trust boundary. GSD evaluation-scope resolved all three task commits on this branch.

Not run in this plan: coverage, release build, live `test:e2e`, dependency freshness or advisory checks. Coverage and release acceptance belong to the final phase gate. This plan added no human-only verification deferral and did not access live owner state.

## RED / GREEN Evidence

- [02-05-RED.json](./02-05-RED.json): the crash tracer compiled and failed with `failed / 0 / NULL` instead of `queued / 1 / 2`. The completed tracer now passes through all three attempts and store reopenings.
- [02-05-QUEUE-RED.json](./02-05-QUEUE-RED.json): the queue test compiled against the empty query seam and failed because the future persisted attempt was missing. The implemented queries pass the deadline, ordering and exclusion assertions.
- [02-05-LIVE-RED.json](./02-05-LIVE-RED.json): live automatic shutdown ended `failed` instead of `queued` before shared outcome rules were wired.
- [02-05-HEAD-RED.json](./02-05-HEAD-RED.json): retry resolution returned `new-head` instead of the recorded `old-head`. It now preserves all four revision fields while initial and manual resolution still fetch current refs.

Each record preserves the unchanged JUnit report from the actual failing Nextest execution. GSD's classifier returned **RED_EVIDENCE_OK** for all four records; the failures reached their planned assertions. Tests and implementation were committed together only after GREEN because mandatory project hooks require passing commits.

## Deviations from Plan

1. **[Rule 1 — Spec conflict] Retain the head during retries now.** The plan deferred same-head retries and described a budget per job reviewing the current head. SPEC Queue rules requires retries on the same head. The orchestrator confirmed the source-of-truth correction. Existing runs provide the binding, so no schema or SPEC change was needed. The job's counter now belongs to its bound revision. Producer-side deduplication across separately enqueued jobs for one target/head remains Phase 3 work.
2. **[Rule 3 — File limit] Split implementation and tests into small modules.** Recovery row handling, queue queries, revision lookup, successful outcomes, process-ID recording and test fixtures moved into submodules to retain the 100-line limit. `finish_failed` groups kind and stop cause in a tuple to keep the API within Clippy's argument limit.
3. **Canonical live failure reasons.** A failed run's job stores the same cause text recovery uses, replacing the older `run N failed (...)` wrapper. The signal integration assertion now checks the exact canonical reason while preserving every process, cause, exit and cleanup assertion. Manual cancellation before any run keeps its earlier explanatory reason.

## Issues Encountered

- Rust's SQLite API reads PR numbers as signed integers; queue rows use checked conversion to `u64`.
- Clippy required typed diagnostic empty-value assertions and ready futures in the hand-written fake implementations. An initial Task 2 hook failed its lint step; no commit occurred, the assertions were corrected, and the next hook passed the whole suite.
- The pre-existing dirty `.planning/config.json` and untracked `.planning/milestone.lock` / `.planning/state.json` were preserved and never staged.

## Next Phase Readiness

- **02-06 must preserve automatic retry binding.** `runner::resolve` now takes `Store` and `EnqueuedJob`; `Store::retry_revision(job)` returns the first recorded revision only for automatic jobs. Deduplication must compare the revision actually selected, including a pinned retry, rather than replace it with the newest GitHub head. Manual and initial jobs continue to resolve current refs.
- **02-09 can consume the queue seam.** `next_runnable_job(now_ms)` and `earliest_next_attempt()` have real SQLite implementations; their production dead-code expectations remain only until the watch runner consumes them.
- **Phase 3 owns automatic producer semantics.** New heads require separate jobs/budgets; multiple separately enqueued automatic jobs for the same head are not globally deduplicated by this plan. Transient `agent_error` classification also remains the explicitly assigned Phase 3 work; this phase treats every agent error as terminal.
- **02-10 owns kill-during-recovery acceptance and the final coverage/release gates.** Recovery and live run/job updates already use IMMEDIATE transactions.

No dependency or external setup changes are required.

## Self-Check: PASSED

All created implementation and evidence files exist, all three task commits are reachable, GSD resolves their evaluation scope, final tests and lint passed, and the final diff and source-length checks passed. No unrelated planning state was staged.
