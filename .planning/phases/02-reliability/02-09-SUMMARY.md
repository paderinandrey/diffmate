---
phase: 02-reliability
plan: 09
subsystem: runner
tags: [watch, queue, signals, retry, locking, sqlite]
requires:
  - phase: 02-05
    provides: "Due-only queue selection, automatic retry budgets and revision binding"
  - phase: 02-06
    provides: "Atomic job attachment, stored results and review lock waiting"
  - phase: 02-07
    provides: "Stored freshness and post-agent head checks"
  - phase: 02-08
    provides: "Durable-first timeout cancellation and configured deadlines"
provides:
  - "Runner-only watch with UTC events, lock waiting and role-specific shutdown"
  - "Hidden debug enqueue using the same attachment transaction as review"
  - "Durable signal persistence retries before cancellation, retaining owned work"
  - "Attached reviews preserve automatic backoff and execute only their own job"
  - "Live lock-holder roles and accurate waiting-review cancellation messages"
affects: [02-10, 03-queue, 04-review-and-publish]
actuals:
  tokens: 16227
  tasks: 3
  commits: 3
plan_head_before: 7590eaddfe4b61e6b46318aacec288f997c32df9
plan_head_after: 33bdd8fe9fbfd3f787c9230710aa851ebab00865
commits: 3
tech-stack:
  added: []
  patterns:
    - "Persistent signal listeners span lock waiting, recovery and job execution"
    - "Signal persistence retries once per second and warns once before cancelling"
    - "Check retry deadlines before acquiring the lock and again after recovery"
key-files:
  created:
    - src/debug.rs
    - src/watch.rs
    - src/watch/run.rs
    - src/watch/wait.rs
    - src/watch/events.rs
    - src/watch/events/date.rs
    - src/lock/role.rs
    - src/store/jobs/due.rs
    - src/review/signals/tests.rs
    - src/review/wait/tests.rs
    - tests/watch.rs
    - tests/rerun.rs
  modified:
    - src/cli.rs
    - src/main.rs
    - src/lock.rs
    - src/review.rs
    - src/review/signals.rs
    - src/review/wait.rs
    - src/store/jobs.rs
    - src/store/retry.rs
    - src/store/retry/queue.rs
    - .planning/WINDOWS.md
key-decisions:
  - "Follow SPEC: failed signal-cause persistence must never trigger cancellation"
  - "Release the runner lock while an attached automatic retry is not due"
  - "Read a role only while its lock is actually held, avoiding stale watch promises"
  - "Keep watch restricted to existing review jobs; producers and GitHub polling remain Phase 3"
requirements-completed: [REL-08, REL-10, REL-11, REL-13]
coverage:
  - id: D1
    description: "Hidden enqueue queues or attaches without config, GitHub, tracking changes, a lock or a run"
    requirement: REL-10
    verification:
      - kind: integration
        ref: "tests/watch/debug.rs#enqueue_ignores_config_and_lock_and_attaches_without_running"
        status: pass
      - kind: integration
        ref: "tests/watch.rs#debug_is_hidden_from_help"
        status: pass
    human_judgment: false
  - id: D2
    description: "Watch runs automatic jobs and prints UTC event lines without findings"
    requirement: REL-08
    verification:
      - kind: integration
        ref: "tests/watch.rs#watch_runs_an_automatic_job_and_stops_when_idle"
        status: pass
      - kind: unit
        ref: "src/watch/events/tests.rs#utc_roundtrips_milliseconds"
        status: pass
      - kind: unit
        ref: "src/watch/events/outcomes.rs#successful_events_count_findings_without_printing_them_and_identify_reuse"
        status: pass
    human_judgment: false
  - id: D3
    description: "Both shutdown signals stop the process group, preserve budget, requeue once and allow restart"
    requirement: REL-08
    verification:
      - kind: integration
        ref: "tests/watch/shutdown.rs#sigterm_during_an_automatic_run_requeues_it"
        status: pass
      - kind: integration
        ref: "tests/watch/shutdown.rs#a_second_sigterm_during_the_stop_is_ignored"
        status: pass
      - kind: integration
        ref: "tests/watch/wait.rs#watch_waits_for_the_lock_and_stops_on_sigterm"
        status: pass
    human_judgment: false
  - id: D4
    description: "A temporary SQLite failure keeps the agent owned and alive until a stop cause is durable"
    requirement: REL-08
    verification:
      - kind: unit
        ref: "src/review/signals/tests.rs#a_failed_signal_write_does_not_cancel_until_persisted"
        status: pass
      - kind: integration
        ref: "tests/watch/persistence.rs#a_failed_signal_write_keeps_the_owned_agent_until_the_retry"
        status: pass
    human_judgment: false
  - id: D5
    description: "Attached reviews honor retry backoff and cancellation without executing unrelated jobs"
    requirement: REL-10
    verification:
      - kind: integration
        ref: "tests/rerun.rs#review_runs_only_the_rerun_it_attached_to"
        status: pass
      - kind: integration
        ref: "tests/rerun/delayed.rs#an_attached_retry_waits_until_due_and_runs_only_its_job"
        status: pass
      - kind: integration
        ref: "tests/rerun/cancel.rs#stopping_a_delayed_attachment_preserves_origin_and_budget"
        status: pass
      - kind: unit
        ref: "src/review/wait/tests.rs#a_delayed_job_waits_without_holding_the_lock_then_becomes_runnable"
        status: pass
    human_judgment: false
  - id: D6
    description: "Waiting reviews name the live watch holder and print jobs completed by watch"
    requirement: REL-11
    verification:
      - kind: integration
        ref: "tests/watch/attach.rs#stopping_a_waiting_review_names_watch"
        status: pass
      - kind: integration
        ref: "tests/watch/attach.rs#review_while_watch_runs_prints_the_result"
        status: pass
    human_judgment: false
  - id: D7
    description: "Watch loads validated configuration at process start before opening the store"
    requirement: REL-13
    verification:
      - kind: integration
        ref: "tests/watch/debug.rs#watch_rejects_invalid_configuration_before_opening_the_store"
        status: pass
    human_judgment: false
  - id: D8
    description: "Release-binary live watch shutdown/restart and attach-to-rerun hand checks"
    requirement: REL-08
    verification: []
    human_judgment: true
    rationale: "Deferred to end-of-phase UAT in WINDOWS entries 7 and 8"
duration: approximately 25 min
completed: 2026-11-03
status: complete
---

# Phase 2 Plan 09: Watch runner and queued re-runs Summary

**Watch executes the due review queue with timestamped events, durable shutdown and lock-holder roles; attached reviews preserve retry delays and leave unrelated work queued.**

## Accomplishments

- Added visible `watch` and hidden `debug enqueue URL [--automatic] [--force]`. Debug uses the existing enqueue transaction, preserves origin and tracking on attachment, reads no config and calls neither GitHub nor Claude. Watch loads config once through the same startup path as review.
- Watch takes the runner lock, announces its role, performs recovery and runs due review jobs in ID order. Lock and idle waits use one-second Tokio intervals; the next retry can shorten the idle interval. Events include UTC timestamps, starts, finding counts, reuse, failures, retry budgets and shutdown. Watch stdout remains empty and finding text stays out of its events.
- Shared signal handling now retains the owned job future while it stores the first cause. A failed write warns once and retries once per second. Only a successful store operation permits cancellation. The job then completes its existing process-group stop, persistence and cleanup path; subsequent signals cannot overwrite a requeued job. SIGINT and SIGTERM mean shutdown for watch.
- Watch exits cleanly while waiting for the lock or idle. During an automatic run, shutdown records the cause, removes its worktree, requeues without consuming budget and exits before another queued job starts. Real fake-process tests verify agent/descendant disappearance and a successful restarted watch.
- Added a Store read seam for queued retry deadlines. A review waiting for an automatic retry leaves the lock available; after recovery it checks again because recovery may have just scheduled a retry. Attachment, cancellation and eventual due execution preserve automatic origin and budget and never start unrelated jobs.
- The locked file records `review` or `watch` without releasing its OS lock. Reading the role checks that the lock is still held, so an old `watch` line cannot falsely promise future execution. Review and watch retain their signal listeners across runner transitions.

## Task Commits

1. **Task 1: Automatic enqueue and watch tracer** — `f5229f8` (`feat`).
2. **Task 2: Durable shutdown during runs and lock waiting** — `d29c61f` (`fix`).
3. **Task 3: Retry backoff, lock roles and attach-to-rerun** — `33bdd8f` (`fix`).

The measured task range contains three commits and 35 changed files. Its 64,906 diff characters divided by four, rounded up, yield 16,227 estimated tokens; this is not a runtime token count. This summary and WINDOWS updates are a separate metadata commit. Shared STATE, ROADMAP, REQUIREMENTS, config and runtime tracking remain owned by the orchestrator.

## Verification

Observed results:

- Task 1 tracer filter passed **7 tests**; that command then exited 1 because the sandbox denied mise cache writes. The sanctioned full-suite run passed **277 tests**, with 11 ignored live checks, exit 0. After minor lint fixes, the Task 1 hook again passed all 277 tests.
- Task 2 expanded watch/signal/attachment filter: **18 passed**, exit 0. The added real SQLite trigger-failure case was included in the full suite: **284 passed**, 11 ignored live checks, exit 0.
- Task 3 planned filter expanded for the paused-time retry wait: **27 passed**, exit 0.
- Final `NEXTEST_TEST_THREADS=2 mise run --force test`: **297 passed**, 11 ignored live checks, exit 0. This includes the final event/config tests, all timeout tests, pinned-revision retry reuse and stored freshness checks. Builds, targeted tests, full suites and lint were run sequentially.
- Final `mise run --force lint`: rustfmt, clippy with warnings denied, cargo-machete, gitleaks and actionlint passed.
- All three task commits passed mandatory hk hooks. Hooks reused successful source caches when current and ran secrets and large-file checks; Task 3 Markdown checking reported zero errors. No hook was bypassed.
- Diff inspection, `git diff --check`, commit deletion checks and source lengths passed. Every one of the 32 changed Rust files is at most 100 lines. GSD evaluation-scope resolved all three reachable task commits. No unfinished stub, dependency addition, schema change or unplanned external endpoint was found.

Not run in this plan: coverage, release build, live `test:e2e`, dependency freshness or advisory checks. Final gates belong to 02-10. The two release/live hand checks remain deferred to end-of-phase UAT in [WINDOWS entries 7 and 8](../../WINDOWS.md). No owner database or configuration was accessed.

## RED / GREEN Evidence

- [02-09-RED.json](./02-09-RED.json): the compiled enqueue/watch tracer failed because the binary did not recognize `debug`. The implemented command now passes the full enqueue, event, idle shutdown and stored-origin assertions.
- [02-09-SIGNAL-RED.json](./02-09-SIGNAL-RED.json): after extracting the unchanged signal write into a test seam, a real SQLite trigger caused the existing helper to cancel without a durable cause. The paused-time regression now waits through the fault, and the black-box watch test independently verifies that the owned agent remains alive until the database is repaired.
- [02-09-DUE-RED.json](./02-09-DUE-RED.json): an attached automatic job with a future retry timestamp completed immediately instead of entering the required not-due wait. The same test now observes no run or agent before the deadline becomes due, then exactly one execution of its attached job.

Each record preserves the unchanged JUnit report from an actual failing Nextest run and records its report timestamps. GSD returned **RED_EVIDENCE_OK** for all three records. Their failures reached the intended behavior assertions. Tests and implementation were committed together only after GREEN because project hooks require passing commits. Added edge tests that were already GREEN are regression checks, not invented RED cycles.

## Deviations from Plan

1. **[Rule 2 — SPEC precedence, assigned 02-08 follow-up] Persist signal causes before cancellation.** The inherited signal handler warned and cancelled after a failed SQLite write. It now follows the same durable-first rule as timeout handling, retaining work and retrying with one warning. [WINDOWS entry 6](../../WINDOWS.md) was marked fixed after the signal, SQLite and process-group checks passed. SPEC was not edited.
2. **[Rule 2 — Assigned 02-06 follow-up] Preserve retry backoff on manual attachment.** Added `Store::job_delayed` and gates before lock acquisition and after recovery. Tests cover equality at the deadline, an available lock while delayed, eventual execution, both waiting signals, and a retry delay created by recovery. Origin, budget, pinned revision, result and freshness semantics remain intact.
3. **[Rule 3 — File limit] Split implementation and tests.** UTC arithmetic, event outcome checks, role I/O, lock waiting, Store due reads and black-box cases use small submodules. Temporary dead-code expectations were removed from queue seams now used by watch; the old generic lock-acquire helper remains test-only.
4. **[Rule 1 — Stale role metadata] Verify a live holder when reading its role.** The lock file persists after exit. Its role is returned only when a separate nonblocking lock attempt confirms that the lock is still held. A regression checks role replacement, continued exclusion and absence after release.
5. **[Rule 2 — Signal transition safety] Retain signal listeners across phases.** A process owns one listener set across waiting, recovery and execution. A completed job is accepted before a simultaneously ready signal so cancellation cannot rewrite its already-finished transition; watch observes a remaining signal before taking another job.

## Issues and Limits

- **Permanent SQLite failure prevents safe signal completion.** The process retains its job future and retries storage. It does not signal an agent without a durable cause or fabricate a terminal result. This follows SPEC ordering and matches the existing timeout limitation.
- The initial sandboxed test command could not write mise's cache. Subsequent test, lint and commit operations used sanctioned escalation; process disappearance checks never treated a failed `ps` call as success.
- Clippy identified small assertion, replacement, return-shape and binding issues in new code; all were corrected before the final successful lint. No test expectation was weakened.
- Existing dirty shared planning files and untracked milestone/runtime state were preserved and never staged. No push, PR, live authentication check or owner-state mutation occurred.

## Next Phase Readiness

- **02-10:** Watch/debug can drive the crash and restart scenarios. Run the final coverage, release, advisory and dependency gates and complete the deferred release/live UAT checks. The final suite already includes the 02-07 freshness and 02-08 timeout regressions.
- **Phase 3:** Watch is a runner-only queue consumer. GitHub polling, tracking, debounce, job producers, notifications and transient agent-error classification remain in their assigned phase. No producers were added here.

No external setup or new dependency is required.

## Self-Check: PASSED

All 35 task files exist, all three task commits are reachable and resolved by GSD, final tests and lint passed, and diff/source-length checks passed. Shared planning state was not modified or staged. The two outstanding human checks and the fixed signal-persistence issue are recorded in WINDOWS.
