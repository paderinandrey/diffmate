---
phase: 02-reliability
plan: 08
subsystem: runner
tags: [timeout, cancellation, sqlite, retries, paused-time]
requires:
  - phase: 02-03
    provides: "Durable stop reasons and shared agent process-group cancellation"
  - phase: 02-05
    provides: "Automatic retry budgets and revision binding"
  - phase: 02-06
    provides: "Review-job execution and attachment seams"
provides:
  - "Configured deadline covering preparation and agent execution"
  - "Timeout persisted before cancellation through the existing process-group stop path"
  - "First-writer stop reasons, transient SQLite failure handling and automatic timeout retries"
  - "Fourteen paused-time timeout and cancellation regressions"
affects: [02-09, 02-10, 03-queue]
actuals:
  tokens: 7550
  tasks: 2
  commits: 2
plan_head_before: f13a2a72923ce3d18967710ad79166c16644621e
plan_head_after: a9a790291773e3a134ef05a510b11e464c02879a
commits: 2
tech-stack:
  added: []
  patterns:
    - "Merged watch cancellation retains the owned work future until graceful completion"
    - "A failed timeout write retries once per second and emits only one warning"
key-files:
  created:
    - src/runner/agent_run/deadline.rs
    - src/runner/agent_run/fakes.rs
    - src/runner/agent_run/fakes/git.rs
    - src/runner/agent_run/tests.rs
    - src/runner/agent_run/tests/fixture.rs
    - src/runner/agent_run/tests/completion.rs
    - src/runner/agent_run/tests/edges.rs
    - src/runner/agent_run/tests/persistence.rs
    - src/runner/agent_run/tests/persistence_run.rs
    - src/runner/agent_run/timing_tests.rs
    - .planning/phases/02-reliability/02-08-RED.json
  modified:
    - src/runner/agent_run.rs
    - src/store/runs.rs
    - src/config.rs
    - src/agent/request.rs
    - .planning/WINDOWS.md
key-decisions:
  - "Follow SPEC: a failed timeout write must not trigger cancellation; retain work and retry until a cause is durable"
  - "Keep the first persisted reason when timeout and signal race, including a signal written during timeout persistence failure"
  - "Keep the deadline inside run_agent so preparation and agent share one budget without changing runner callers"
requirements-completed: [REL-12, REL-13]
coverage:
  - id: D1
    description: "Manual jobs time out at 60 seconds and remove their worktree and run directory"
    requirement: REL-12
    verification:
      - kind: unit
        ref: "src/runner/agent_run/tests.rs#a_run_over_the_timeout_is_failed_timeout"
        status: pass
      - kind: unit
        ref: "src/runner/agent_run/tests/persistence_run.rs#timing_out_during_preparation_removes_the_worktree"
        status: pass
    human_judgment: false
  - id: D2
    description: "The default is fifteen minutes and a result one second before the configured deadline is kept"
    requirement: REL-13
    verification:
      - kind: unit
        ref: "src/runner/agent_run/tests/completion.rs#default_timeout_is_fifteen_minutes"
        status: pass
      - kind: unit
        ref: "src/runner/agent_run/tests/completion.rs#an_agent_that_finishes_a_second_early_is_kept"
        status: pass
    human_judgment: false
  - id: D3
    description: "Durable cancellation preserves the first reason through SQLite failure and signal races"
    requirement: REL-12
    verification:
      - kind: unit
        ref: "src/runner/agent_run/tests/persistence.rs#a_failed_stop_write_never_cancels_before_a_durable_retry"
        status: pass
      - kind: unit
        ref: "src/runner/agent_run/tests/persistence_run.rs#a_run_keeps_the_signal_persisted_during_a_failed_timeout_write"
        status: pass
      - kind: unit
        ref: "src/runner/agent_run/timing_tests.rs#a_ready_outer_stop_wins_over_a_ready_deadline"
        status: pass
    human_judgment: false
  - id: D4
    description: "Automatic timeout failures use the existing retry budget and one-minute retry delay"
    requirement: REL-12
    verification:
      - kind: unit
        ref: "src/runner/agent_run/tests/edges.rs#an_automatic_job_that_times_out_is_requeued"
        status: pass
    human_judgment: false
  - id: D5
    description: "Cancellation uses the existing process-group stop and does not interrupt graceful cleanup again"
    requirement: REL-12
    verification:
      - kind: unit
        ref: "src/runner/agent_run/timing_tests.rs#an_outer_stop_disables_the_deadline_during_graceful_cleanup"
        status: pass
      - kind: integration
        ref: "tests/signals.rs#sigint_during_agent_run_stops_group_and_records_interrupted"
        status: pass
    human_judgment: false
  - id: D6
    description: "Release-binary real Claude one-minute timeout, status and process cleanup check"
    requirement: REL-12
    verification: []
    human_judgment: true
    rationale: "Deferred to end-of-phase UAT; automated timeout checks use fakes and paused time"
duration: approximately 17 min
completed: 2026-11-03
status: complete
---

# Phase 2 Plan 08: Run timeout Summary

**Preparation and agent execution share the configured timeout, and the runner persists its cause before stopping the agent and recording `failed (timeout)`.**

## Accomplishments

- Wrapped the existing `run_agent` body with a merged cancellation receiver and one deadline. The budget begins immediately after run creation, covers preparation and the agent's CLI work, and leaves graceful shutdown and cleanup to the existing cancellation path. No runner caller or stored-result seam changed.
- Added `Store::set_stop_reason_for_run` through `Store::call`. Its conditional update preserves the first recorded reason. A successful timeout write precedes the merged cancellation notification, so the existing outcome store records `timeout` without an interrupted cause. Manual jobs end failed; automatic jobs use the existing retry rules and keep their revision binding.
- Activated `Settings::run_timeout` in production. Tests prove the one-minute override, fifteen-minute default, successful completion at 59 seconds, and both preparation-only and combined preparation/agent timeout behavior.
- Added paused-time deadline, ordering, cleanup and SQLite fault tests. A temporary failed write leaves cancellation unset until persistence succeeds. A signal that is written while timeout persistence fails keeps `user_cancel`, and the whole job records `interrupted` with that cause. Concurrent updates select exactly one first writer.
- Kept each changed Rust source file at or below 100 lines by splitting fake Git, test fixtures and related cases into small modules. No new dependency or production schema change was needed.

## Task Commits

1. **Task 1: Configured timeout tracer** — `01e4eb4` (`feat`).
2. **Task 2: Boundary, retry and persistence regressions** — `a9a7902` (`test`).

The measured task range contains two commits and 15 changed files. Its 30,199 diff characters divided by four, rounded up, yield 7,550 estimated tokens; this is not a runtime token count. This summary and the two WINDOWS entries are a separate metadata commit. Shared STATE, ROADMAP, REQUIREMENTS, config and runtime tracking remain owned by the orchestrator.

## Verification

Observed results:

- Task 1 planned runner/signal filter: **16 passed**, exit 0.
- Task 1 full `NEXTEST_TEST_THREADS=2 mise run --force test`: **258 passed**, 11 ignored live checks, exit 0.
- Task 2 first expanded filter: **26 passed** after correcting test-future ownership. Two additional preparation/concurrent-persistence cases brought the final planned filter to **28 passed**, exit 0.
- Final `NEXTEST_TEST_THREADS=2 mise run --force test`: **271 passed**, 11 ignored live checks, exit 0. All tests and builds ran without overlapping rebuilds.
- Final `mise run --force lint`: rustfmt, clippy with warnings denied, cargo-machete, gitleaks and actionlint passed. Two duration-style warnings in new tests were corrected before this successful run.
- Both task commits passed mandatory hk hooks, using only the immediately preceding successful full-suite and lint source caches. Hooks ran secrets and large-file checks; Markdown checks reported zero errors. No hook was bypassed.
- Diff inspection, `git diff --check`, file-existence and source-length checks passed. No task commit deleted a tracked file. GSD evaluation-scope resolved both reachable task commits. Stub and threat-surface scans found no unfinished implementation or unplanned trust boundary.

Not run in this plan: coverage, release build, live `test:e2e`, dependency freshness or advisory checks. Final coverage and release acceptance remain with 02-10. The real Claude one-minute hand check is deferred to end-of-phase UAT and recorded in [WINDOWS entry 5](../../WINDOWS.md). No live owner database or configuration was accessed.

## RED / GREEN Evidence

[02-08-RED.json](./02-08-RED.json) preserves the unchanged JUnit report from the actual failing tracer. The compiled review stayed pending until the 61-second test guard instead of finishing at its 60-second deadline. GSD returned **RED_EVIDENCE_OK**, and the failure reached the intended completion assertion. The implementation now passes the same tracer and its exact time, run/job classification and cleanup assertions.

Task 2's cases confirmed behavior already implemented by Task 1. They were GREEN on their first compiled execution, so no artificial RED or speculative production fix was added. The initial test ownership compilation errors were not RED evidence. Tests and implementation were committed together only after GREEN because project hooks require passing commits.

## Deviations from Plan

1. **[Rule 2 — SPEC precedence] Retry a failed stop-reason write before cancellation.** The plan said to warn and cancel even after a failed write. SPEC's Agent contract requires a stored reason before any signal. The timeout path instead retains the owned work future and retries persistence once per second, with a single immediate warning and a single retained warning. Cancellation begins after a successful database operation, including when another cause already won. The orchestrator accepted this correction; SPEC was not edited.
2. **[Rule 3 — File limit] Split fake and test modules.** Fake Git and job-level cases use submodules instead of putting every case in the plan's two test files. All modified sources stay within 100 lines.
3. **[Rule 2 — Failure and ordering coverage] Exercise temporary write failure and concurrent signal persistence.** Real SQLite triggers reject timeout writes in temporary fixture databases. Tests prove both recovery to `timeout` and preservation of a signal written during that failure window. The automatic retry timestamp is checked against the wall-clock interval around execution because persisted retry deadlines use Unix milliseconds, while run timers use paused Tokio time.
4. **Task 2 was already GREEN.** The expanded tests required no further production behavior change. Mandatory hooks prevented failing-test commits; the actual Task 1 RED evidence remains committed separately from the test runner's temporary report file.

## Issues and Limits

- **Permanent database unavailability prevents safe timeout completion.** The owned work future remains retained while writes are retried; no cancellation signal is emitted without a durable cause, and no terminal result is invented. This is the consequence of SPEC's ordering requirement, rather than a guarantee that an unavailable SQLite store can still enforce a completed stop at the deadline.
- **02-09 follow-up:** the pre-existing `src/review/signals.rs` path logs persistence failure and then cancels anyway. That conflicts with the same SPEC ordering rule, lies outside this plan's ownership, and is recorded in [WINDOWS entry 6](../../WINDOWS.md). The orchestrator accepted the mandatory 02-09 follow-up. Timeout persistence itself follows the corrected rule.
- The existing dirty shared planning files and untracked milestone/runtime state were preserved and never staged.

## Next Phase Readiness

- **02-09:** preserve the timeout's durable-first cancellation semantics when correcting signal handling. The queued retry due-time follow-up from 02-06 remains unchanged.
- **02-10:** run final coverage/release checks and complete WINDOWS entry 5's real CLI timeout hand check. The automated timeout tests use only typed fakes under paused time; existing real-process signal tests continue to prove the shared stop path.

No external setup or dependency changes are required.

## Self-Check: PASSED

All 15 task files exist, both task commits are reachable and resolved by GSD, final tests and lint passed, and diff/source-length checks passed. Shared planning state was not modified or staged. The outstanding real CLI hand check and signal-path persistence follow-up are explicitly recorded in WINDOWS.
