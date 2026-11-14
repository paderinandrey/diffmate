---
phase: 02-reliability
plan: 04
subsystem: cli
tags: [status, freshness, retry, sqlite, proptest]
requires:
  - phase: 02-01
    provides: "Schema v2 stored heads and retry metadata"
  - phase: 01-single-review
    provides: "Read-only store, Gh trait and isolated CLI fixtures"
provides:
  - "One live read-only head check per target with a draft"
  - "Current/stale freshness with timestamped and labelled stored fallback"
  - "Retry budget, due time, requeue and exhaustion text in status"
affects: [02-05, 02-07, 02-10]
actuals:
  tokens: 10631
  tasks: 3
  commits: 3
plan_head_before: d6069456026da757254a3ca0816e4f2c5b463a6d
plan_head_after: ed870a625b627937ea682883a9c33e34525faa13
commits: 3
tech-stack:
  added: []
  patterns:
    - "Pure relative-time, freshness and budget formatters receive the current time"
    - "A single SQLite read returns the latest successful draft independently of the latest run"
key-files:
  created:
    - src/status/freshness.rs
    - src/status/when.rs
    - src/status/budget.rs
    - src/status/tests.rs
    - src/status/tests/fixture.rs
    - src/status/tests/stored.rs
    - src/status/tests/budget.rs
    - src/store/status/latest.rs
    - tests/status/helpers.rs
    - tests/status_freshness.rs
    - .planning/phases/02-reliability/02-04-RED.json
    - .planning/phases/02-reliability/02-04-BUDGET-RED.json
  modified:
    - src/status.rs
    - src/store/status.rs
    - tests/status.rs
    - tests/support/fake_gh.rs
    - .planning/WINDOWS.md
key-decisions:
  - "Compare only the draft and PR heads; base branch movement never affects freshness"
  - "Test monotonic represented durations across minute/hour/day transitions"
  - "Exercise Task 2 black-box scenarios before Task 1 implements their shared behavior"
requirements-completed: [REL-02, REL-03, REL-06]
coverage:
  - id: D1
    description: "Live checks show current or stale from the compared PR head"
    requirement: REL-02
    verification:
      - kind: integration
        ref: "tests/status_freshness.rs#status_shows_an_unmoved_head_as_current"
        status: pass
      - kind: integration
        ref: "tests/status_freshness.rs#status_shows_a_moved_head_as_stale"
        status: pass
    human_judgment: false
  - id: D2
    description: "Failed checks retain the stored head and age, or explicitly show unknown"
    requirement: REL-02
    verification:
      - kind: unit
        ref: "src/status/tests/stored.rs#fallback_uses_the_stored_head_and_latest_successful_run"
        status: pass
      - kind: unit
        ref: "src/status/tests.rs#failed_checks_compare_the_stored_head_and_name_its_age"
        status: pass
      - kind: integration
        ref: "tests/status_freshness.rs#status_reports_a_failed_live_check"
        status: pass
    human_judgment: false
  - id: D3
    description: "Status remains read-only and targets without drafts make no head request"
    requirement: REL-02
    verification:
      - kind: integration
        ref: "tests/status.rs#status_does_not_write"
        status: pass
      - kind: integration
        ref: "tests/status.rs#status_lists_latest_run_per_target"
        status: pass
      - kind: integration
        ref: "tests/status_freshness.rs#status_shows_a_moved_head_as_stale"
        status: pass
    human_judgment: false
  - id: D4
    description: "Advancing only the base branch keeps a draft current"
    requirement: REL-03
    verification:
      - kind: integration
        ref: "tests/status_freshness.rs#a_moved_base_branch_keeps_the_draft_current"
        status: pass
    human_judgment: false
  - id: D5
    description: "Status renders persisted retry timing, requeues and exhausted budgets"
    requirement: REL-06
    verification:
      - kind: unit
        ref: "src/status/tests/budget.rs#queued_failures_show_retry_count_and_remaining_delay"
        status: pass
      - kind: unit
        ref: "src/status/tests/budget.rs#automatic_failures_report_exhausted_and_zero_budgets"
        status: pass
      - kind: unit
        ref: "src/status/tests/stored.rs#a_queued_crash_retry_appends_its_persisted_budget_to_latest"
        status: pass
    human_judgment: false
  - id: D6
    description: "Relative times obey boundaries and remain monotonic as represented durations"
    verification:
      - kind: unit
        ref: "src/status/tests.rs#elapsed_time_labels_are_valid_and_never_go_backwards"
        status: pass
      - kind: unit
        ref: "src/status/tests.rs#elapsed_times_use_whole_units_and_clamp_clock_skew"
        status: pass
      - kind: unit
        ref: "src/status/tests.rs#future_times_use_whole_units_and_due_times_are_now"
        status: pass
    human_judgment: false
  - id: D7
    description: "Release-binary base-branch advance check against real GitHub"
    requirement: REL-03
    verification: []
    human_judgment: true
    rationale: "Deferred to end-of-phase UAT; automated checks used isolated local git and fake gh"
duration: approximately 18 min
completed: 2026-11-03
status: complete
---

# Phase 2 Plan 04: Live status freshness and retry budget Summary

**Status compares each saved draft with one live PR head, labels fallback checks with their stored age, and displays persisted retry budgets without writing state.**

## Accomplishments

- Added live current/stale output using 12-character heads, with sanitized warnings and explicitly labelled stored-head or unknown fallback on failure.
- Kept `Store::open_read_only`, with no lock acquisition, recovery or stored head updates. The SQL uses the latest successful run even when a later run failed. Targets without drafts retain their existing row format and make no GitHub call.
- Added relative times, queued retry timing, `requeued` and automatic budget exhaustion. Retry execution remains in the later plans.
- Added route replacement helpers and 15 tests covering live freshness, base-only pushes, read-only behavior, fallback selection, sanitization, time boundaries, properties and budget output.

## Task Commits

1. **Task 1: Live-head tracer and status read model** — `ff31220` (`feat`).
2. **Task 2: Stored fallback, time boundaries and base movement** — `506d812` (`test`).
3. **Task 3: Persisted retry-budget display** — `ed870a6` (`feat`).

The measured range contains three task commits and 16 changed implementation, test and evidence files. Its 42,524 diff characters divided by four yield 10,631 estimated tokens, not a runtime token count. This summary and the UAT ledger entry are a separate metadata commit. Shared STATE, ROADMAP, REQUIREMENTS, config and handoff artifacts remain owned by the orchestrator.

## Verification

Observed results:

- Task 1 filter (`binary(status_freshness) | binary(status) | binary(recovery) | binary(concurrency)`): **10 passed**.
- Task 2 filter (`test(status::) | binary(status_freshness)`): **11 passed**.
- Task 3 filter (`test(status::) | binary(status)`): **15 passed**, exit 0 after cache-write escalation.
- Final `NEXTEST_TEST_THREADS=2 mise run --force test`: **225 passed**, 11 ignored live checks, exit 0. Earlier complete suites passed 214 tests for Task 1 and 221 for Task 2.
- Final `mise run --force lint`: passed rustfmt, clippy with warnings denied, cargo-machete, gitleaks and actionlint.
- All task commits passed mandatory hk hooks. Later hooks reused only the just-passed full-suite and lint source caches; no hooks were bypassed.
- `git diff --check`, changed source/test file lengths (all at most 100 lines), commit ancestry and deletion checks passed. `check evaluation-scope --plan 02-04 --commits-only --raw` resolved all three reachable commits. Stub and threat-surface inspection found no unfinished implementation or unplanned trust boundary.

Not run here: coverage, release build, live `test:e2e`, dependency/advisory checks or the real-GitHub base-branch hand check. Coverage and release acceptance remain phase gates. The hand check is recorded as entry 2 in `.planning/WINDOWS.md` for end-of-phase UAT; no live state was touched.

## RED / GREEN Evidence

- Before shared freshness implementation, all four black-box scenarios compiled and failed: the old command made no head request and omitted freshness/failure labels. This includes the Task 2 failed-live-check and base-only-move scenarios. The unchanged JUnit report is in [02-04-RED.json](./02-04-RED.json); GSD reported `RED_EVIDENCE_OK` after adding the required target metadata to the evidence record.
- Before budget implementation, three assertions failed, including the persisted crash row: actual `failed (interrupted: crash)` versus expected `failed (interrupted: crash); retry 1 of 2 in 1m`. [02-04-BUDGET-RED.json](./02-04-BUDGET-RED.json) retains the JUnit report and passed GSD's RED validator. The negative-state test already passed against the empty formatter.
- The final implementations pass all these assertions. Task 2's pure boundary tests passed against the behavior already required by Task 1, so they needed no formatter correction.

## TDD Gate Compliance

Actual RED was observed before shared behavior and budget implementation. Failing tests were committed with their passing implementation because mandatory project hooks require green commits. No failing-test commit or hook bypass was used.

## Deviations from Plan

1. **Task ordering:** Task 2's black-box tests were written and exercised with the tracer before Task 1 implemented the same fallback/base behavior. Task 2 then added pure and seeded-store coverage and strengthened failed-check sanitization and route restoration assertions.
2. **[Rule 3 — Blocking] File-length limit:** Moved existing status test helpers into `tests/status/helpers.rs` and split new seeded/budget tests under `src/status/tests/` to keep each source file within 100 lines.
3. **Contradictory property wording:** Raw numeric labels decrease at unit boundaries (`59m` to `1h`). The property checks monotonic represented durations and valid label shapes while preserving every exact boundary case from the plan.

## Issues Encountered

- A host-wide subprocess launch stall affected ordinary git calls, ps and unrelated tests during the first Task 1 hook. The executor interrupted only its observed Nextest process; the commit did not occur. Rerunning with two concurrent tests passed. Subsequent full suites used the same setting.
- A Task 3 filtered run passed all tests but mise could not write its task cache under the sandbox. The identical command was rerun with escalation and exited 0.
- Initial compilation required explicit checked conversion from SQLite's signed PR number to `u64`; initial clippy feedback required non-overlapping time ranges. Both were corrected before the introducing commit.

## Next Phase Readiness

Plans 02-05 and 02-07 can populate retry and stored-head metadata consumed by these status rows. The owner-facing real-GitHub base-branch check remains end-of-phase UAT. No dependency or external setup changes are required.

## Self-Check: PASSED

All created files exist, all task commits are reachable, the final suite and lint passed, the diff and file-length checks passed, and no unrelated planning changes were staged.
