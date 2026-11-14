---
phase: 02-reliability
plan: 06
subsystem: runner
tags: [deduplication, sqlite, attachment, cancellation, proptest]
requires:
  - phase: 02-01
    provides: "Schema v2 force, origin and result_run_id columns"
  - phase: 02-03
    provides: "Durable stop causes and role-specific signals"
  - phase: 02-05
    provides: "Recovery rules and automatic retries bound to their stored revision"
provides:
  - "Same-head successful run reuse with explicit --force and reuse output"
  - "Atomic queued-first review attachment that preserves origin and tracking"
  - "Waiting reviews follow stored results and handle SIGINT and SIGTERM"
  - "Attach invariants and real previous-finding reference checks"
affects: [02-07, 02-09, 02-10, 03-queue, 04-review-and-publish]
actuals:
  tokens: 18849
  tasks: 3
  commits: 3
plan_head_before: f6e56bfd31555b611f38f88e9666df6603ae92fc
plan_head_after: 4348af00d1ccf1378ef7f7ea1404d4caac54622b
commits: 3
tech-stack:
  added: []
  patterns:
    - "An IMMEDIATE transaction selects an existing active job or inserts a new one"
    - "Finished job results use result_run_id, falling back to the latest owned run"
    - "Reuse compares the actual selected revision, including pinned automatic retries"
key-files:
  created:
    - src/store/jobs/attach.rs
    - src/store/jobs/attach/active.rs
    - src/store/result.rs
    - src/store/result/read.rs
    - src/review/wait.rs
    - src/review/print.rs
    - src/runner/result/stored.rs
    - src/runner/run.rs
    - src/store/jobs/tests.rs
    - tests/dedup.rs
    - tests/attach.rs
    - tests/review_output/previous.rs
  modified:
    - src/cli.rs
    - src/main.rs
    - src/review.rs
    - src/runner.rs
    - src/runner/result.rs
    - src/store/jobs.rs
    - src/store/finish.rs
    - src/store/status.rs
key-decisions:
  - "Keep the 02-05 selected revision authoritative for deduplication"
  - "Check job completion before attempting the lock so finished results require no lock"
  - "Preserve reuse transparency for waiting processes by comparing the result run's owning job"
requirements-completed: [REL-04, REL-11, REL-05]
coverage:
  - id: D1
    description: "A repeated same-head review reuses its stored draft and --force creates another run"
    requirement: REL-04
    verification:
      - kind: integration
        ref: "tests/dedup.rs#same_head_reuses_the_run"
        status: pass
      - kind: integration
        ref: "tests/dedup.rs#force_makes_a_new_run"
        status: pass
    human_judgment: false
  - id: D2
    description: "Concurrent reviews attach to one job, with forced running and queued exceptions"
    requirement: REL-11
    verification:
      - kind: integration
        ref: "tests/attach.rs#two_reviews_of_one_url_produce_one_run"
        status: pass
      - kind: integration
        ref: "tests/attach/forced.rs#force_during_a_running_job_produces_a_second_run"
        status: pass
      - kind: integration
        ref: "tests/attach/forced.rs#a_plain_review_attaches_to_the_queued_forced_job"
        status: pass
    human_judgment: false
  - id: D3
    description: "Waiting reviews print completed results without the lock and stop without cancelling jobs"
    requirement: REL-11
    verification:
      - kind: integration
        ref: "tests/attach/finished.rs#a_finished_job_is_printed_without_taking_the_held_lock"
        status: pass
      - kind: integration
        ref: "tests/attach.rs#stopping_a_waiting_review_leaves_its_job_queued"
        status: pass
    human_judgment: false
  - id: D4
    description: "Enqueue sequences preserve the queued-job bound, force rules, origin and tracking"
    requirement: REL-11
    verification:
      - kind: unit
        ref: "src/store/jobs/tests.rs#enqueue_preserves_active_job_and_origin_invariants"
        status: pass
    human_judgment: false
  - id: D5
    description: "Another target's finding ID and an unknown forced-rerun ID are nulled with warnings"
    requirement: REL-05
    verification:
      - kind: integration
        ref: "tests/review_output/previous.rs#previous_id_from_another_target_is_nulled"
        status: pass
      - kind: integration
        ref: "tests/review_output/previous.rs#unknown_id_on_a_forced_rerun_is_nulled"
        status: pass
      - kind: integration
        ref: "tests/context.rs#previous_json_lists_earlier_findings"
        status: pass
    human_judgment: false
  - id: D6
    description: "Status uses the successful run referenced by a reusing job"
    requirement: REL-04
    verification:
      - kind: integration
        ref: "tests/dedup.rs#same_head_reuses_the_run"
        status: pass
    human_judgment: false
  - id: D7
    description: "An automatic retry deduplicates against its pinned head after GitHub moves"
    requirement: REL-04
    verification:
      - kind: unit
        ref: "src/runner/resolve/tests/reuse.rs#dedup_uses_the_pinned_retry_head_instead_of_the_current_github_head"
        status: pass
    human_judgment: false
duration: approximately 18 min
completed: 2026-11-03
status: complete
---

# Phase 2 Plan 06: Run reuse and active-job attachment Summary

**Repeated reviews reuse a successful same-head draft, concurrent requests share an active job, and waiting processes return stored results or stop without cancelling work.**

## Accomplishments

- Added `review --force`, explicit enqueue requests carrying origin and force, and atomic start-time force reads. A non-forced job resolves its revision and then looks up the latest successful run of the same target and exact head before creating any run, worktree, context or agent invocation. Both new success and reuse populate `result_run_id`.
- Added stored-result reading for findings in ordinal order, limitation, future improvements, warnings and failure details. Reused reviews render that stored draft and identify the run and shortened head on stderr. Status follows the result run instead of assuming every job owns its result.
- Implemented queued-first attachment in one IMMEDIATE transaction. Force upgrades an existing queued job, attaches to an already forced running job, or creates a successor to a non-forced running job. Attachment never rewrites origin or target tracking.
- Waiting reviews check job progress on every lock poll, print results from another runner without taking the lock, and recheck state after recovery if they acquire it. SIGINT and SIGTERM while waiting return exit 1 with the specified message and leave the queued job and stop cause unchanged.
- Added a 32-case property test over enqueue/start/finish sequences and tracked/untracked targets. Added real-run checks for invalid previous finding references, plus a typed-fake regression proving deduplication uses an automatic retry's old revision instead of the new GitHub head.

## Task Commits

1. **Task 1: Same-head reuse and --force** — `41f47db` (`feat`).
2. **Task 2: Active attachment and waiting results** — `175f4e5` (`feat`).
3. **Task 3: Attach invariants and real previous finding IDs** — `4348af0` (`test`).

The measured task range contains three commits and 45 changed files. Its 75,394 diff characters divided by four, rounded up, yield 18,849 estimated tokens; this is not a runtime token count. This summary is a separate metadata commit. Shared STATE, ROADMAP, REQUIREMENTS, config and runtime tracking remain owned by the orchestrator.

## Validation

Observed results:

- Task 1 final tracer filter: **43 passed**, exit 0. The earlier sandboxed run also passed 43 tests but ended exit 1 while writing mise's cache; it was not treated as a successful command.
- Task 2 expanded filter: **14 passed**, including both waiting signals, successful and failed results while the lock stays held, concurrent attachment, forced successors, status reuse and retry revision preservation.
- Task 3 expanded filter: **9 passed**, including the property test, real previous-ID checks and pinned retry deduplication.
- Final `NEXTEST_TEST_THREADS=2 mise run --force test`: **251 passed**, 11 ignored live checks, exit 0. Task 1 and Task 2 full suites passed 242 and 247 tests respectively.
- Final `mise run --force lint`: rustfmt, clippy with warnings denied, cargo-machete, gitleaks and actionlint passed.
- All three task commits passed mandatory hk hooks. Hooks reused immediately preceding successful full-suite and lint source caches and ran secrets and large-file checks; no hook was bypassed.
- Final diff inspection and `git diff --check` passed. Every changed Rust file is at most 100 lines. No task commit deleted a tracked file. GSD evaluation-scope resolved all three task commits on this branch. Stub and threat-surface scans found no unfinished implementation markers or unplanned external trust boundary.

Not run in this plan: coverage, release build, live `test:e2e`, dependency freshness or advisory checks. Final coverage and release acceptance belong to 02-10. No live owner state was accessed. Task 3's real CLI reuse, force and concurrent attachment hand check remains deferred to end-of-phase UAT in WINDOWS entry 3; the orchestrator recorded it when closing the wave.

## RED / GREEN Evidence

- [02-06-RED.json](./02-06-RED.json): the compiled same-head test failed because two reviews created two runs instead of one. The force test also showed that the CLI did not recognize `--force`. Both now pass.
- [02-06-ATTACH-RED.json](./02-06-ATTACH-RED.json): the compiled concurrent-review test failed because the second process did not attach. The queued-force attachment and waiting SIGINT cases also failed; the latter exited from the signal instead of returning exit 1. All now pass, including SIGTERM.

Both records preserve the unchanged JUnit reports from actual Nextest runs. GSD returned **RED_EVIDENCE_OK** for each, and the target failures reached their intended behavior assertions. Tests and implementation were committed together only after GREEN because project hooks require passing commits.

Task 3 added tests for behavior already implemented by Task 2 and the existing normalizer. Its first run passed all eight planned tests; no production fix was needed and no artificial RED was introduced. The additional pinned-retry test also passed. These are regression additions, not a claimed new RED/GREEN cycle.

## Deviations from Plan

1. **[Rule 3 — File limit] Split small implementation and test modules.** Enqueue moved into `src/store/jobs/attach.rs` during Task 1; active selection, stored decoding, result rendering, runner execution and test fixtures use submodules to keep every Rust file within 100 lines. All existing enqueue callers were updated, including live-test support and context fixtures omitted from the plan's file list.
2. **[Rule 2 — Required behavior] Check job progress before trying the lock.** The task prose listed lock acquisition first, but the required truth says a finished job exits without taking the lock. The wait loop checks completion first and still rechecks after recovery to close the race. A held-lock integration test proves successful and failed exits through this path.
3. **[Rule 2 — Regression protection] Test the inherited retry binding explicitly.** The fake Git allows only clone/fetch resolution and rejects worktree work; the fake agent rejects any call. With successful runs available for both old and new heads, the retry selects the old-head result and never fetches or starts a worktree/agent.
4. **Task 3 was already GREEN.** The attach property and real previous-ID cases confirmed existing behavior. No speculative production change or fabricated failing test was added.

## Issues Encountered

- Sandbox restrictions blocked mise cache writes and an initial lint launch. Sanctioned escalated runs completed the actual full tests and lint.
- Clippy required a borrowed JSON input and a ready-future signature for the test agent seam. Both were corrected before commits.
- The retry-resolution fixture now finishes its first job before enqueuing a fresh one; otherwise the new attachment semantics correctly return the existing job. Its old-head, new-head and Git-call assertions remain intact.
- The existing dirty `.planning/config.json` and untracked `.planning/milestone.lock` / `.planning/state.json` were preserved and never staged.

## Next Phase Readiness

- **02-07:** `ReviewResult::Succeeded` now includes `reused`, and the stored-success conversion lives in `src/runner/result/stored.rs`. Freshness additions must cover both newly produced and stored/reused results. The former `review_run` helper is now `src/runner/run.rs`.
- **02-09 — mandatory due-time check:** a `review` that attaches to an automatic queued retry with a future `next_attempt_at` currently executes that queued job immediately after acquiring the lock, following this plan's action. Attachment preserves automatic origin and must not implicitly cancel its backoff. When adding watch/debug enqueue, add a runnable/waiting gate for the attached review and a test with a future retry deadline. This plan does **not** claim that attached manual waiters already honor automatic retry delays. The orchestrator accepted this explicit follow-up without expanding 02-06.
- **02-09:** `enqueue_review` returns `Enqueued { job, attached }` and requires `EnqueueRequest { origin, force }`. `start_job` returns `StartedJob { force }`. The existing generic `lock::acquire` has a temporary production dead-code expectation until the watch loop consumes it.
- **Phase 3:** producer-side handling of separately enqueued automatic work, new-head budgets and polling remains Phase 3 work. This plan does not claim producer deduplication is complete.
- **02-10:** final coverage, release, crash-point and human acceptance gates remain with the phase close-out.

No dependencies or external setup changed.

## Self-Check: PASSED

All created files exist, the three task commits are reachable and resolved by GSD, full tests and lint passed, and final diff/source-length checks passed. Shared planning state was not modified or staged.
