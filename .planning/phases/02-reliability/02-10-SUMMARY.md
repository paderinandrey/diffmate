---
phase: 02-reliability
plan: 10
subsystem: reliability
tags: [crash-recovery, process, retry, signals, testing]
requires:
  - phase: 02-05
    provides: "Durable retry budgets, cause-preserving recovery and pinned revisions"
  - phase: 02-06
    provides: "Atomic attachment and stored-result reuse"
  - phase: 02-07
    provides: "Stored freshness and post-agent head checks"
  - phase: 02-08
    provides: "Durable-first run deadlines"
  - phase: 02-09
    provides: "Watch, debug enqueue, persistent signal listeners and retry due checks"
provides:
  - "Fifteen feature-only named crash points and twenty-two real-process crash tests"
  - "A repeated manual request completes after recovering its own crashed attachment"
  - "Crash-budget and stored-stop-cause proofs across watch restarts"
  - "Final local gate evidence and explicit external live-gate blocker"
affects: [03-queue, phase-02-verification, phase-02-uat]
actuals:
  tokens: 13001
  tasks: 3
  commits: 3
plan_head_before: b34fa770acc0377c96987bcb96e3308f3a0905e4
plan_head_after: f0406a4f9af1205775294378afd1ccac5fa7e106
commits: 3
tech-stack:
  added: []
  patterns:
    - "Cargo feature removes the crash-point call and implementation from default builds"
    - "Synchronize process crashes with stderr markers and agent FIFOs"
    - "Use typed recovery results to distinguish a manual crash from stored stop causes"
key-files:
  created:
    - src/crash.rs
    - src/review/acquire.rs
    - src/recovery/cleanup.rs
    - tests/crash.rs
    - tests/crash/helpers.rs
    - tests/crash/matrix.rs
    - tests/crash/matrix/assertions.rs
    - tests/crash/restart.rs
    - tests/crash/budget.rs
    - tests/crash/process.rs
    - tests/crash/stop_stored.rs
    - tests/crash/manual_stop.rs
  modified:
    - Cargo.toml
    - mise.toml
    - hk.pkl
    - src/main.rs
    - src/review.rs
    - src/review/signals.rs
    - src/recovery.rs
    - src/store/recovery.rs
    - src/runner.rs
    - src/runner/process.rs
    - src/runner/run.rs
    - src/runner/outcome/succeeded.rs
    - src/runner/outcome/failed.rs
    - src/runner/result.rs
    - src/watch.rs
    - .planning/WINDOWS.md
key-decisions:
  - "Only the review process that recovers its own attached manual crash creates a new manual job; stored shutdown, user_cancel and automatic retries keep their rules"
  - "Keep force intent on that replacement request without reviving or rewriting the failed job"
  - "Enforce at most one run per manual job and exact bounded attempts for automatic jobs"
  - "Place stored crash points only after a successful durable outcome or reuse write"
  - "Advance hk to 2.10.0 after the freshness gate, preserving the parser-compatible plain pin"
  - "Leave the final phase gate blocked on Anthropic account verification without weakening tests"
requirements-completed: [REL-01, REL-06, REL-07]
coverage:
  - id: D1
    description: "SIGKILL at each runner step preserves database and worktree invariants after restart"
    requirement: REL-01
    verification:
      - kind: integration
        ref: "tests/crash/matrix.rs#fourteen named crash cases"
        status: pass
      - kind: integration
        ref: "tests/crash.rs#killed_while_the_agent_runs_leaves_no_duplicate_and_no_worktree"
        status: pass
    human_judgment: false
  - id: D2
    description: "A manual restart preserves force intent and a saved reused result survives another crash"
    requirement: REL-01
    verification:
      - kind: integration
        ref: "tests/crash/restart.rs#restarting_a_crashed_forced_review_keeps_its_force_intent"
        status: pass
      - kind: integration
        ref: "tests/crash/restart.rs#a_reused_result_survives_a_crash_after_storage"
        status: pass
    human_judgment: false
  - id: D3
    description: "A repeatedly crashing automatic job makes three attempts and persists its exhausted budget"
    requirement: REL-06
    verification:
      - kind: integration
        ref: "tests/crash/budget.rs#crash_every_time_exhausts_the_budget_across_watch_restarts"
        status: pass
    human_judgment: false
  - id: D4
    description: "A stored shutdown requeues automatic work; user_cancel and manual stops remain terminal after SIGKILL"
    requirement: REL-07
    verification:
      - kind: integration
        ref: "tests/crash/stop_stored.rs#shutdown_cause_survives_a_crash_and_requeues"
        status: pass
      - kind: integration
        ref: "tests/crash/stop_stored.rs#user_cancel_cause_survives_a_crash_and_is_not_retried"
        status: pass
      - kind: integration
        ref: "tests/crash/manual_stop.rs#manual shutdown and user_cancel regression cases"
        status: pass
    human_judgment: false
  - id: D5
    description: "Default release builds contain neither the crash-control variable nor the reached message"
    requirement: REL-01
    verification:
      - kind: other
        ref: "mise run build; binary search for DIFFMATE_CRASH_POINT and crash point reached"
        status: pass
    human_judgment: false
  - id: D6
    description: "The final live Claude gate still needs restored Anthropic account access"
    verification:
      - kind: e2e
        ref: "src/tests/isolation/auth.rs#isolation_flags_keep_authentication"
        status: fail
      - kind: e2e
        ref: "src/tests/live#four remaining model-backed checks"
        status: unknown
    human_judgment: true
    rationale: "Anthropic returned HTTP 403 requiring account verification; this external state must change before the required live checks can pass"
  - id: D7
    description: "Release-binary live SIGKILL preparation, running-agent and retry-budget hand checks"
    requirement: REL-01
    verification: []
    human_judgment: true
    rationale: "Deferred to end-of-phase UAT and recorded in WINDOWS entries 9 and 10"
duration: approximately 35 min
completed: 2026-11-03
implementation_status: complete
phase_gate: blocked
status: halted
---

# Phase 2 Plan 10: Crash proofs and final gates Summary

**Fifteen feature-only crash points and twenty-two process tests prove restart cleanup, bounded retries and retained stop causes. Implementation is committed; the final live gate is blocked by an Anthropic account restriction.**

## Accomplishments

- Added `crash-points`, enabled it for the normal Nextest and coverage tasks, and gated the crash integration target with `required-features`. Each selected point flushes its stderr marker and awaits indefinitely until the test kills the process. The default release binary contains neither `DIFFMATE_CRASH_POINT` nor `crash point reached`.
- Instrumented enqueue, lock acquisition, database recovery, completed recovery, job start, fetch, revision resolution, run creation, worktree creation, context writing, agent invocation, agent PID persistence, cleanup, durable outcome/reuse storage and durable signal-cause storage. Existing split modules determine the call sites; watch lock/recovery points belong in `src/watch.rs`.
- The matrix checks every manual boundary, including early attachment, a started job without a run, interrupted runs, saved outcomes and reuse. Every case checks no running rows, at most one run per manual job, empty run directories and only the bare clone in worktree metadata. Successful runs are checked against the actual fixture head.
- Fixed the tracer-discovered restart failure. A new review previously attached to a dead manual job, recovered it as failed and returned exit 1. Recovery now reports typed `manual_crashes`; only the review process that itself recovers its attached job issues a fresh manual enqueue under the runner lock, with the new request's force intent. The failed job/run remain intact. Automatic backoff, pinned revisions, tracking, durable-first cancellation and stored stop causes are unchanged.
- An automatic job killed during every attempt performs exactly three runs across four watch starts, exposes `retry 1 of 2` and `retry 2 of 2`, then persists `no retries left (2 of 2 used)`. A different queued job succeeds afterward. The test advances only `jobs.next_attempt_at` to zero; existing paused-time tests still prove the real delay rules.
- Killing immediately after stop persistence preserves `shutdown` and `user_cancel`. Automatic shutdown produces one failed run and one successful retry on the same job without using budget. User cancellation is terminal. Two extra manual cases prove the new restart behavior never re-enqueues a job with a stored shutdown or user-cancel cause. A further case kills the reuse path after its durable result is saved.

## Task Commits

1. **Task 1: Feature-only agent crash tracer and manual restart correction** — `701f182` (`feat`).
2. **Task 2: Every-step matrix and force-intent regression** — `8c20f8a` (`test`).
3. **Task 3: Restart budgets, durable causes and final gate execution** — `f0406a4` (`test`).

The measured task range contains **3 commits**, **33 changed files** and **52,003 diff characters**. Dividing characters by four and rounding up gives **13,001 estimated tokens**, not a runtime token count. This summary and WINDOWS updates are a separate metadata commit. Shared STATE, ROADMAP, REQUIREMENTS, config and runtime state remain owned by the orchestrator.

## Verification

All commands below ran sequentially with respect to builds, tests and lint. Test/coverage runs and commit hooks used `NEXTEST_TEST_THREADS=2`; hooks also used `HK_JOBS=1`.

| Command | Observed result |
| --- | --- |
| `mise run --force test -- -E 'binary(crash) \| binary(recovery)'` | Final Task 1 run: exit 0, **2 passed** |
| `mise run --force test -- -E 'binary(crash)'` | Task 2: exit 0, **16 passed**; Task 3: exit 0, **22 passed** |
| `mise run --force test` | Task 1: **298 passed**; Task 2: **313 passed**; final: **319 passed**, **11 ignored live checks**, exit 0 (178.763 seconds for the final suite) |
| `mise run --force lint` | Final exit 0: rustfmt, clippy with warnings denied, cargo-machete, gitleaks and actionlint |
| `mise run build && ! rg -a -q 'DIFFMATE_CRASH_POINT\|crash point reached' target/release/diffmate` | Exit 0 for both the tracer and final release builds; neither string occurs in the final binary |
| `mise run coverage` | Exit 0 with the feature suite; **94.80% line coverage**, 4,594 lines and 239 missed, above the unchanged 80% gate |
| `mise run deps:check` | First exit 1 for hk 2.9.0 → 2.10.0; after the forward update, exit 0 for Rust 1.99.1, mise tools and direct crates |
| `mise run deps:update` | Updated hk, then exited 2 because mise rewrote the pin as a table; restored an equivalent plain `"2.10.0"` pin and updated the `hk.pkl` import before the passing freshness check |
| `mise run vuln` | Exit 0: advisories, bans, licenses and sources passed; existing non-fatal duplicate-version warnings for `hashbrown` and `syn` |
| `mise run --force docs:check` | Exit 0, zero errors; Task 3's hook also checked the updated WINDOWS ledger with zero errors |
| `mise run test:e2e` | **Exit 100: 5/11 executed, 4 passed, 1 failed; 6 not run because of fail-fast** |
| `mise run test:e2e -- -E 'test(tests::isolation::rev07::) \| test(tests::isolation::rev08::)'` | Exit 0, **2 passed**; completes the two remaining local endpoint isolation controls without another model call |

All three task commits passed mandatory hooks. Cached source checks were reused only after the corresponding successful full suite/lint execution. No hook was bypassed. Diff inspection, `git diff --check`, deletion checks and source-length checks passed: all **27 changed Rust files are at most 100 lines**. GSD evaluation-scope resolved all three reachable task commits. No stub, new network endpoint, auth path or schema change was introduced.

## External Authentication Gate — BLOCKED

`isolation_flags_keep_authentication` reached Claude **2.1.314** and received an `agent_error` carrying `api_error_status = 403` and `terminal_reason = api_error`. Anthropic's response says the account cannot receive replies and requires account verification. This is an external account gate, not a missing flag or a test assertion to relax. The full `test:e2e` gate has **not passed**.

The first run passed all three CLI probes and the worktree CLAUDE.md isolation control. The separate follow-up passed the outside-import and hooks/plugins/MCP controls against local recording endpoints. Thus **six non-model live checks passed**, **one account-backed check failed**, and these **four model-backed checks remain unrun**:

- `tests::live::agent::live_max_turns_one_is_failed_not_no_findings`
- `tests::live::prs::live_clean_pr_has_no_findings`
- `tests::live::prs::live_obvious_bug_is_anchored_to_line_7`
- `tests::live::prs::live_fork_pr_is_reviewed_the_same_way`

Read-only `gh pr view` of the existing clean fixture succeeded and returned the recorded fixture head, so GitHub fixture access is available. No further Claude model calls were made after the 403. The owner's diffmate database and configuration were not modified. No test was weakened, ignored anew or waived.

The external failure and four unrun checks are recorded in [WINDOWS entries 11 and 12](../../WINDOWS.md). On 2026-11-03 the owner explicitly chose **“Сохранить live gate как заблокированный”** (keep the live gate blocked). No automatic Claude API retry is authorized by this handoff. A later owner-directed continuation can restore account access and rerun `NEXTEST_TEST_THREADS=2 mise run test:e2e`. Phase completion remains pending that gate, end-of-phase UAT and the orchestrator's code review/verifier.

## RED / GREEN Evidence

- [02-10-RED.json](./02-10-RED.json): the compiled tracer completed a review without announcing `agent-running`; the selected crash point now stops it exactly there.
- [02-10-RESTART-RED.json](./02-10-RESTART-RED.json): with crash instrumentation active, rerunning the same manual review returned exit 1 after recovering the attached failed job. The corrected path now passes the exact two-job/two-run assertions and succeeds.
- [02-10-MATRIX-RED.json](./02-10-MATRIX-RED.json): the compiled enqueue case completed without announcing `enqueued`; the complete matrix now passes.

Each file retains the unchanged JUnit report from the actual failing command, timestamps and a semantic assessment. GSD returned **RED_EVIDENCE_OK** for all three. Test fixture/compiler corrections were not counted as RED evidence. Task 3's tests exercised prior behavior and were GREEN on their first compiled execution; no artificial failure or speculative production change was added. Tests and implementation were committed together only after GREEN because hooks require passing commits.

## Deviations from Plan

1. **[Rule 1 — Restart bug] Complete the explicitly repeated manual request.** The tracer exposed the failed attachment behavior described above. A typed recovery report and a lock/recovery helper fix it without turning manual jobs into automatic retries or treating stored stop causes as crashes. The orchestrator accepted this correction. SPEC was not edited.
2. **[Rule 2 — Automatic retry invariants] Apply one-run-per-job only to manual jobs.** Automatic retries intentionally produce several runs for one job. Cleanup checks retain the strict manual invariant; budget and shutdown cases independently assert exact automatic attempt counts and terminal states.
3. **[Rule 3 — Current module layout and file limit] Follow existing seams and split small helpers.** Process recording, run execution, outcome persistence and watch startup use their actual modules. Recovery cleanup and review acquisition were split to keep every modified Rust file within 100 lines; matrix assertions have their own small module.
4. **[Rule 1 — Agent-start fixture] Do not wait on a nonexistent agent.** `agent-start` is before the agent is invoked, so the test asserts no fake call exists. FIFO holding/releasing applies only to `agent-running`, where the fake is known to have started. Holding a fake at `agent-start` would deadlock the test.
5. **[Project rule — Dependency freshness] Upgrade hk to 2.10.0.** The final freshness gate required it. The updater's known plain-string parser issue was corrected in the resulting configuration without pinning back or changing any crate/toolchain version.
6. **[External gate] Stop model-backed verification after Anthropic's 403.** Completed the unaffected local controls and GitHub read diagnostic, committed implementation, and retained a halted summary with explicit pending checks. No waiver or false complete phase status was recorded.

## Remaining Hand Checks

The release-binary live SIGKILL budget check and kills during preparation/agent execution were not performed. They remain in [WINDOWS entries 9 and 10](../../WINDOWS.md), in addition to earlier phase UAT entries. The feature-build matrix and existing isolated tests are automated evidence, not claims that those live hand checks occurred.

## PR Handoff

After the blocked live gate, phase review/verifier and UAT are satisfied, the orchestrator owns the PR from `gsd/phase-02-reliability`: `gh pr create --assignee @me`, followed by `gh pr comment NUMBER --body "@codex review"`. No push, PR, review request or merge was performed by this executor.

The PR rationale should explain:

- **TOML:** the owner-approved `toml` crate provides serde parsing and source spans for strict integer settings; unknown keys and invalid values fail before enqueue. This concrete need avoids a custom parser.
- **Schema v2:** the owner's `additive-backup` choice preserves v1 data and original columns and writes a private validated backup before transactional addition of job origin, force, stop cause, retry budget/deadline, result reuse, and target head/check time. Earlier values and read-only status behavior are protected by migration tests.
- **Crash testing:** the one approved test-only environment variable selects named crash points behind a cargo feature. Normal tests, hooks and CI coverage exercise it; the release build excludes both the code and control strings.
- **Budget and revision:** retry count, fixed maximum and next-attempt time live on the job, and retries use the first recorded revision. Restarting or changing current configuration does not reset that budget. Shutdown and user cancellation follow separate durable rules.
- **Hidden enqueue and watch:** debug enqueue uses the future producer/attachment transaction without running work, while watch is a runner-only consumer in this phase. Polling/tracking producers remain Phase 3. Durable causes precede cancellation and retry waiting releases the runner lock.
- **Repeated manual review:** a request that itself recovers its crashed manual attachment creates one fresh job while preserving the failed history and force intent. Stored stop causes and automatic jobs never take that path.
- **Validation limits:** local verification is green at 94.80% line coverage; the full live gate remains blocked until Anthropic account access is restored. Carry that result honestly until it has been rerun successfully.

## Self-Check: PASSED

All 33 task files exist and all three task commits are reachable and resolved by GSD. The summary's Markdown links pass with zero errors; diff and source-length checks pass. This verifies the recorded implementation and evidence, **not** the blocked live gate. Shared planning state was preserved. The owner-selected external block and remaining hand checks remain open in WINDOWS.
