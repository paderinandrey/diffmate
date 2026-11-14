---
phase: 02-reliability
plan: 07
subsystem: runner
tags: [freshness, sqlite, rendering, cancellation, regression]
requires:
  - phase: 02-04
    provides: "Read-only live status freshness and stored-head fallback"
  - phase: 02-05
    provides: "Automatic retries pinned to their first recorded revision"
  - phase: 02-06
    provides: "Same-head reuse, job attachment and stored-result decoding"
provides:
  - "Runner-only last-known head and check time at job start and after successful analysis"
  - "First-line stale warnings for fresh, reused and attached review results"
  - "Successful reviews survive failed head rechecks with durable warnings"
  - "Base-only movement and stored status fallback regression coverage"
affects: [02-08, 02-09, 02-10, 03-queue]
actuals:
  tokens: 6997
  tasks: 2
  commits: 2
plan_head_before: d0166b30bd3cc5256396724a9ba6c9077566aec5
plan_head_after: 41fdd714a4500eedd5eb7831de17c94c35f0af36
commits: 2
tech-stack:
  added: []
  patterns:
    - "Fresh successful outcomes use the same stored-result decoder as reused and attached results"
    - "Post-agent head checks race the existing cancellation receiver"
key-files:
  created:
    - src/store/targets.rs
    - src/render/tests/freshness.rs
    - tests/stale_review.rs
    - tests/stale_review/edges.rs
    - tests/status_freshness/failure.rs
    - .planning/phases/02-reliability/02-07-RED.json
    - .planning/phases/02-reliability/02-07-SANITIZE-RED.json
  modified:
    - src/runner.rs
    - src/runner/run.rs
    - src/runner/result.rs
    - src/runner/result/stored.rs
    - src/runner/outcome.rs
    - src/runner/outcome/succeeded.rs
    - src/store/result.rs
    - src/render.rs
    - tests/status_freshness.rs
    - tests/migration.rs
    - .planning/WINDOWS.md
key-decisions:
  - "Persist the current GitHub head independently of the revision selected for a pinned retry"
  - "Use SQLite as the common success-result source so freshness is identical for every consumer"
  - "Sanitize the new external head display using the existing terminal sanitizer"
requirements-completed: [REL-02, REL-03]
coverage:
  - id: D1
    description: "A push during a run produces a stale first line and stores the newly observed head"
    requirement: REL-02
    verification:
      - kind: integration
        ref: "tests/stale_review.rs#push_during_run_makes_the_draft_stale"
        status: pass
    human_judgment: false
  - id: D2
    description: "Attached and reused results retain freshness independently of the reviewed revision"
    requirement: REL-02
    verification:
      - kind: integration
        ref: "tests/stale_review.rs#push_during_run_makes_the_draft_stale"
        status: pass
      - kind: unit
        ref: "src/runner/resolve/tests/reuse.rs#dedup_uses_the_pinned_retry_head_instead_of_the_current_github_head"
        status: pass
    human_judgment: false
  - id: D3
    description: "A failed recheck keeps the successful draft, start head and a stored warning"
    requirement: REL-02
    verification:
      - kind: integration
        ref: "tests/stale_review/edges.rs#failed_recheck_keeps_the_run"
        status: pass
      - kind: integration
        ref: "tests/status_freshness/failure.rs#status_falls_back_to_the_stored_head"
        status: pass
    human_judgment: false
  - id: D4
    description: "A base-only move keeps the draft current and a subsequent review reuses it"
    requirement: REL-03
    verification:
      - kind: integration
        ref: "tests/stale_review/edges.rs#base_move_during_run_is_not_stale"
        status: pass
      - kind: integration
        ref: "tests/status_freshness.rs#a_moved_base_branch_keeps_the_draft_current"
        status: pass
    human_judgment: false
  - id: D5
    description: "Freshness changes only the first line and external head text cannot inject controls"
    requirement: REL-02
    verification:
      - kind: unit
        ref: "src/render/tests/freshness.rs#stale_heads_change_only_the_first_line"
        status: pass
      - kind: unit
        ref: "src/render/tests/freshness.rs#the_known_head_cannot_insert_terminal_controls_or_another_line"
        status: pass
    human_judgment: false
  - id: D6
    description: "Release-binary push-during-review check against real GitHub"
    requirement: REL-02
    verification: []
    human_judgment: true
    rationale: "Deferred to end-of-phase UAT; automated checks used isolated local git and fake gh/claude"
duration: approximately 16 min
completed: 2026-11-03
status: complete
---

# Phase 2 Plan 07: Stored head freshness Summary

**The runner records the last observed PR head, successful reviews report stale drafts on their first line, and failed rechecks preserve the draft with a durable warning.**

## Accomplishments

- Added `Store::set_known_head` through the existing blocking SQLite seam. The runner writes `PrMeta.head_sha` and `head_checked_at` after resolution, before reuse or run creation, and reads/stores the head again after a successful agent outcome. Cancellation skips the additional request through the existing cancellation race.
- Added `known_head` to successful results. Fresh successes now decode their committed draft through the same SQLite reader used by reused and attached jobs. The reader includes the target's latest stored head. The FIFO push tracer proves both executing and attached reviews print the same stale result and still exit 0.
- Appended the stale suffix only when the known head differs from the reviewed head. Heads use 12-character display; the new external head field is sanitized and flattened to one line. Findings, limitation and future improvements remain unchanged.
- Kept automatic retry binding: a retry still selects and deduplicates its recorded old revision while separately remembering the current GitHub head. The inherited regression now asserts both values.
- Added FIFO-based base-only movement and recheck-failure regressions. A base move does not change freshness or prevent reuse. A failed recheck keeps the start head, successful run and persisted warning. Status can now use the runner's saved head and timestamp when its live request fails; status itself remains read-only.

## Task Commits

1. **Task 1: Push-during-run freshness tracer** — `eb1d4c2` (`feat`).
2. **Task 2: Base movement, failure fallback and safe head output** — `41fdd71` (`fix`).

The measured task range contains two commits and 21 changed files. Its 27,985 diff characters divided by four, rounded up, yield 6,997 estimated tokens; this is not a runtime token count. This summary and the UAT ledger entry are a separate metadata commit. Shared STATE, ROADMAP, REQUIREMENTS, config and runtime tracking remain owned by the orchestrator.

## Verification

Observed results:

- Task 1 expanded tracer filter: **17 passed**, including pinned retry reuse. The post-commit tracer feedback filter passed **16 tests**.
- Task 1 full `NEXTEST_TEST_THREADS=2 mise run --force test`: **253 passed**, 11 ignored live checks, exit 0.
- Task 2 planned filter: **10 passed** on its first run. Its final filter, expanded for the sanitizer regression, passed **12 tests**.
- Final `NEXTEST_TEST_THREADS=2 mise run --force test`: **257 passed**, 11 ignored live checks, exit 0. This final run had no concurrent rebuild.
- Final `mise run --force lint`: rustfmt, clippy with warnings denied, cargo-machete, gitleaks and actionlint passed.
- Both task commits passed mandatory hk hooks, reusing only the immediately preceding successful full-suite and lint source caches. Task 2's hook also ran `docs:check` with zero errors. No hook was bypassed.
- Diff inspection, `git diff --check`, changed Rust file lengths and commit deletion checks passed. Every changed Rust file is at most 100 lines. GSD evaluation-scope resolved both reachable task commits. Stub inspection found no unfinished implementation; no dependency or new external endpoint was added.

Not run in this plan: coverage, release build, live `test:e2e`, dependency freshness or advisory checks. Final coverage and release acceptance belong to 02-10. The real GitHub push-during-review hand check is deferred to end-of-phase UAT and recorded in WINDOWS entry 4. No live owner state was accessed.

## RED / GREEN Evidence

- [02-07-RED.json](./02-07-RED.json) preserves the unchanged JUnit report for the compiled push tracer. The review succeeded but omitted the expected stale suffix. GSD returned **RED_EVIDENCE_OK**; the failure reached the intended first-line assertion. The implemented tracer passes and additionally checks attached output and SQLite head/timestamp values.
- [02-07-SANITIZE-RED.json](./02-07-SANITIZE-RED.json) preserves the unchanged JUnit report showing escape, bidi, newline and tab controls in the new known-head suffix. GSD returned **RED_EVIDENCE_OK** after the record named the complete Rust test path. The corrected render passes the exact safe-output assertion.

Task 2's base movement and fallback tests confirmed behavior already implemented by Task 1, so they needed no invented RED or speculative production changes. The sanitizer correction was a separate observed RED/GREEN cycle. Tests and their implementation were committed together only after GREEN because mandatory project hooks require passing commits.

## Deviations from Plan

1. **[Rule 3 — Existing module layout and file limit] Follow the 02-06 splits.** The second head check lives in `src/runner/run.rs`, and stored-success conversion lives in `src/runner/result/stored.rs`. Thus the two `set_known_head` call sites are split across the runner modules instead of both appearing in `src/runner.rs`. Additional render and integration cases use small submodules to retain the 100-line source limit.
2. **[Rule 2 — Consistent stored success] Share the committed result reader.** Successful outcome handling reads its committed result rather than rebuilding a separate payload. This carries `known_head` through fresh, reused and attached paths without a second freshness representation or changes to the public runner signature.
3. **[Rule 1 — Fixture compatibility] Preserve migration and unknown-head coverage.** The migration integration test now distinguishes the untouched migrated target with no head from the reviewed target with a stored head. Existing snapshot preservation checks remain. The unknown-head status case explicitly seeds the legacy NULL fields so that fallback remains tested independently of the new normal runner behavior.
4. **[Rule 2 — Safe rendering] Sanitize the new GitHub-provided head field.** Final inspection found a new direct terminal sink. A failing regression demonstrated it, and the renderer now uses its existing sanitizer and one-line replacement. No dependency or new policy was added.

## Issues Encountered

- The initial push fixture attempted to replace the pull ref with a sibling commit and failed before the behavior assertion. It was repaired by branching from the existing PR head and pushing a descendant. Only the subsequent assertion failure was used as RED evidence. The failed fixture's orphaned fake-agent process was identified by its exact temporary path, stopped, and confirmed absent.
- One intermediate full suite ended with SIGKILL in two unrelated context tests after 141 passes while a later targeted rebuild overlapped it in the shared target directory. The cause was not established; the final full suite ran without concurrent rebuilding and passed all 257 tests. The interrupted run is not reported as passing.
- The existing dirty `.planning/config.json` and untracked `.planning/milestone.lock` / `.planning/state.json` were preserved and never staged.

## Next Phase Readiness

- **02-08:** The successful post-agent freshness read is in `runner::run`, after agent cleanup, and races `CancelRx`. Timeout/agent execution modules were not changed. `execute_review_job` and the outcome-store caller signature remain compatible.
- **02-09:** `ReviewResult::Succeeded` now has `known_head`; existing matches using `..` remain compatible. The inherited future-retry due-time follow-up remains assigned to 02-09 and was not changed here.
- **02-10:** Run the final coverage/release gates and complete WINDOWS entry 4's real GitHub push hand check. The freshness observation window ends at the successful post-agent head read; a later push appears at the next status live check or run, as the plan specifies.

No external setup or dependency changes are required.

## Self-Check: PASSED

All 21 task files exist, both task commits are reachable and resolved by GSD, final tests and lint passed, and diff/source-length checks passed. Shared planning state was not modified or staged. The outstanding real-GitHub hand check is explicitly recorded for phase UAT.
