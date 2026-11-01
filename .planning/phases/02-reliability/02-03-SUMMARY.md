---
phase: 02-reliability
plan: 03
subsystem: runner
tags: [signals, cancellation, sqlite, process-groups, rust]
requires:
  - phase: 02-01
    provides: "Schema v2 jobs.stop_reason and the existing runs.interrupted_cause"
  - phase: 01-single-review
    provides: "Owned process-group cleanup, cancellation channels and CLI test fixtures"
provides:
  - "Durable first stop reason before review cancellation is requested"
  - "Ctrl-C user_cancel and SIGTERM shutdown classifications on runs and jobs"
  - "Timeout classification, pre-run cause preservation and completed-result precedence"
affects: [02-05, 02-06, 02-07, 02-09, 02-10]
actuals:
  tokens: 9056
  tasks: 2
  commits: 2
plan_head_before: 84637c20c496e14955ea3fc813e8bf020592009d
plan_head_after: 19616d3d868c1b63463774c45ff639dd4f6b09e2
commits: 2
tech-stack:
  added: []
  patterns:
    - "A conditional SQLite UPDATE records only the first stop reason"
    - "Signals await durable storage before setting the cancellation channel"
    - "Only Cancelled outcomes consult the stop reason; completed outcomes retain their result"
key-files:
  created:
    - src/runner/outcome/failed.rs
    - src/runner/outcome/tests.rs
    - src/runner/outcome/tests/fixture.rs
    - src/runner/outcome/tests/preservation.rs
    - tests/signals/helpers.rs
    - tests/signals/process.rs
    - .planning/phases/02-reliability/02-03-RED.json
    - .planning/WINDOWS.md
  modified:
    - src/cancel.rs
    - src/cancel/tests.rs
    - src/agent.rs
    - src/store/jobs.rs
    - src/store/runs.rs
    - src/review.rs
    - src/review/signals.rs
    - src/runner/outcome.rs
    - src/runner/result.rs
    - src/runner/tests.rs
    - tests/signals.rs
    - tests/support/fake_claude.rs
    - tests/support/fake_claude.sh
key-decisions:
  - "Follow the plan's completed-result precedence; a stored later signal remains job history"
  - "Keep all production process signalling on the existing locally spawned process-group path"
  - "Use a TERM-resistant fixture child to make the second-signal cleanup window observable"
requirements-completed: [REL-09, REL-07]
coverage:
  - id: D1
    description: "Ctrl-C and SIGTERM persist their distinct causes and clean the owned process group"
    requirement: REL-09
    verification:
      - kind: integration
        ref: "tests/signals.rs#sigint_during_agent_run_stops_group_and_records_interrupted"
        status: pass
      - kind: integration
        ref: "tests/signals.rs#sigterm_during_agent_run_is_handled_the_same_way"
        status: pass
    human_judgment: false
  - id: D2
    description: "The first stored reason survives a second signal and is available for recovery"
    requirement: REL-07
    verification:
      - kind: unit
        ref: "src/runner/outcome/tests.rs#the_first_stored_stop_reason_wins"
        status: pass
      - kind: integration
        ref: "tests/signals.rs#sigint_then_sigterm_keeps_user_cancel"
        status: pass
    human_judgment: false
  - id: D3
    description: "A pre-run stop preserves its cause and creates no run or filesystem artifacts"
    requirement: REL-09
    verification:
      - kind: unit
        ref: "src/runner/tests.rs#shutdown_before_the_run_exists_keeps_its_cause_and_creates_no_run"
        status: pass
      - kind: unit
        ref: "src/runner/tests.rs#user_cancel_before_the_run_exists_keeps_its_cause_and_creates_no_run"
        status: pass
    human_judgment: false
  - id: D4
    description: "Timeout cancellation and completed success or failure retain their correct classifications"
    verification:
      - kind: unit
        ref: "src/runner/outcome/tests.rs#a_timeout_cancellation_is_a_timeout_without_an_interrupted_cause"
        status: pass
      - kind: unit
        ref: "src/runner/outcome/tests/preservation.rs#a_successful_result_that_finished_first_keeps_its_draft"
        status: pass
      - kind: unit
        ref: "src/runner/outcome/tests/preservation.rs#a_failed_result_that_finished_first_keeps_its_original_failure"
        status: pass
    human_judgment: false
  - id: D5
    description: "Release-binary Ctrl-C hand check with the owner's fixture PR and real Claude process"
    requirement: REL-09
    verification: []
    human_judgment: true
    rationale: "Deferred to end-of-phase UAT; this plan used isolated fake CLI processes and did not touch live state"
duration: not recorded
completed: 2026-11-01
status: complete
---

# Phase 2 Plan 03: Durable stop causes Summary

**Review cancellation stores its first cause before signalling the agent, preserving user_cancel, shutdown and timeout through outcome recording and cleanup.**

## Performance

- Start time was not recorded; production closeout was at 2026-11-01T00:17:44Z.
- Tasks: 2. Production and test commits changed 20 files, including RED evidence.
- Actual token estimate: 36,223 realized diff characters divided by four, rounded up to 9,056. This is not a runtime token count.
- The owner asked to stop at the nearest clean point during Task 2. This plan is the stopping boundary; the next plan is 02-04.

## Accomplishments

- Added `StopReason::{Timeout, Shutdown, UserCancel}` and atomic first-reason storage on the job. The signal loop awaits that write before requesting cancellation and retains the first reason on later signals.
- Ctrl-C on an executing review records `failed (interrupted: user_cancel)`; SIGTERM records `failed (interrupted: shutdown)`. Both keep exit code 1 and the existing cancellation message. The run, job, worktree, run directory and process-group assertions pass.
- Cancelled timeout outcomes record `failure_kind = timeout` with no interrupted cause. Already-completed success and failure outcomes keep their result even when a later signal was stored.
- Stops before the run exists include the stored cause in the failed job reason and create no run, repository or run directory. The no-reason Phase 1 fallback remains covered.

## Task Commits

1. **Task 1: Persist causes and classify signal cancellations** — `7228269` (`feat`).
2. **Task 2: Preserve precedence and pre-run causes** — `19616d3` (`fix`).

Plan metadata is the commit adding this summary and the deferred hand-check ledger entry. Shared STATE, ROADMAP, REQUIREMENTS, config and pause handoff remain owned by the orchestrator.

## Verification

Observed results:

- `mise run --force test -- -E 'binary(signals) | binary(review) | binary(recovery)'`: 6 passed, 0 skipped. Repeated with escalation so process inspection was permitted.
- `mise run --force test -- -E 'binary(signals) | test(cancel::) | test(runner::)'`: 15 passed; 206 excluded by the filter, including ignored live checks.
- Final `mise run --force test`: **210 passed**, 11 ignored live checks; exit 0. The suite included all final source changes.
- Final `mise run --force lint`: passed rustfmt, clippy with warnings denied, cargo-machete, gitleaks and actionlint.
- Mandatory hk hooks passed on both task commits. Task 1's hook ran the full 202-test suite; Task 2's hook reused the just-passed full-suite and lint results through mise's source cache and ran secrets, large-files and docs checks. No hooks were bypassed.
- `git diff --check` passed. Every changed source/test file is at most 100 lines. Both task commits contain no tracked-file deletions. Stub and threat-surface scans found no unfinished implementation or new trust boundary.

Not run in this plan: coverage, release build, live `test:e2e`, dependency/advisory checks or the real-Claude Ctrl-C hand check. Coverage and release acceptance remain at the phase gate. The hand check is recorded as an open `unrun-verify` item in `.planning/WINDOWS.md` for end-of-phase UAT.

## RED / GREEN Evidence

The shutdown-before-run regression compiled and ran, then failed its database assertion: expected `interrupted before the run started (shutdown)`, observed `interrupted before the run started`. The correction reads the durable job cause when constructing that reason. Both shutdown and user-cancel cases pass, alongside the original absent-cause case.

The GSD evidence checker does not parse Nextest's console format. The test was rerun against the unfixed implementation using Nextest's JUnit reporter; its original XML and console evidence are saved in [02-03-RED.json](./02-03-RED.json). `check tdd-red-evidence` returned **RED_EVIDENCE_OK**, with one discovered and failed target test. The implementation was restored only after that accepted report, then the final suite passed.

## TDD Gate Compliance

The failing test and implementation share the green Task 2 commit because the project's mandatory hooks require each commit to pass. Actual RED was observed and machine-validated; no failing commit or hook bypass was used.

## Deviations from Plan

1. **[Rule 3 — Blocking] Split helpers to maintain the file limit.** Failure persistence moved to `src/runner/outcome/failed.rs`; outcome fixtures, preservation tests and signal helpers use small submodules. Public behavior and the planned store interfaces are unchanged.
2. **[Rule 1 — Bug] Make process assertions reliable.** The signal test's `ps` helper now fails on inspection errors instead of interpreting them as process exit. Process tests ran with scoped escalation after sandbox denial was observed. The two-signal fixture uses a child that inherits ignored SIGTERM, giving the second signal an observable cleanup window before the existing SIGKILL escalation.
3. **Project commit policy takes precedence over RED commits.** RED evidence is committed with the passing implementation as described above.

## Issues Encountered

- Initial lint feedback required formatting changes; a speculative, unnecessary lint expectation was removed before Task 1 committed.
- The initial GSD RED parser rejection was a report-format mismatch, resolved with unmodified JUnit output from a real failing Nextest run.
- Existing dirty `.planning/config.json`, `.planning/milestone.lock` and `.planning/state.json` were neither changed nor staged by this executor.

## Next Phase Readiness

Resume with **02-04-PLAN.md**. Plans 02-05 and 02-07 can consume the durable stop reason for recovery and timeout control. Automatic-job requeue rules remain in their assigned later plans. The original agent-group signaller remains unchanged and never uses PID or PGID values read from SQLite.

No user setup is required for this plan. Work stops after its metadata commit as requested.

## Self-Check: PASSED

All created implementation and evidence files exist, both task commits are reachable, and the final diff and source-length checks pass. Verification results above were observed directly.
