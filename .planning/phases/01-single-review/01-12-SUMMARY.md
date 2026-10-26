---
phase: 01-single-review
plan: 12
subsystem: testing
tags: [live-checks, claude, gh, fixtures, phase-gate]

requires:
  - phase: 01-single-review
    provides: "01-02 fixture PRs, 01-10 interrupted runs, 01-11 isolation suite"
provides:
  - "src/tests/live: #[ignore] live checks for REV-03, REV-04, REV-05 and REV-10 with temp roots and a real-state guard"
  - "Phase 1 gate results and the owner hand-check list"
affects: [phase-verification, phase-pr]

actuals:
  tokens: 3412
  tasks: 2
  commits: 3

plan_head_before: 3c7feb487cf6e195c334f96b8158483b78e2ce85
plan_head_after: 5b662c510e35742ee5e13bf36fd209d7b3b59d23
commits: 3

tech-stack:
  added: []
  patterns:
    - "Live tests run review::run against temp state and cache roots, read the temp database afterwards and assert the owner's real database and runs directory are unchanged"

key-files:
  created:
    - src/tests/live.rs
    - src/tests/live/support.rs
    - src/tests/live/stored.rs
    - src/tests/live/agent.rs
    - src/tests/live/prs.rs
  modified:
    - src/tests.rs
    - Cargo.toml
    - Cargo.lock
    - mise.toml
    - hk.pkl

key-decisions:
  - "The four live tests ran once each; the final test:e2e gate run excluded them (-E 'not test(tests::live::)') so no live test made a second model call"
  - "mise.toml keeps hk as a plain version string after deps:update, because scripts/deps.sh parses that form"

requirements-completed: [REV-03, REV-04, REV-05, REV-10]

coverage:
  - id: D1
    description: "A real CLI run with --max-turns 1 is recorded as failed (agent_error, error_max_turns), never as success"
    requirement: REV-05
    verification:
      - kind: test
        ref: "tests::live::agent::live_max_turns_one_is_failed_not_no_findings (mise run test:e2e)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The clean fixture PR is reviewed by the real agent and stored as succeeded with no_findings"
    requirement: REV-04
    verification:
      - kind: test
        ref: "tests::live::prs::live_clean_pr_has_no_findings (mise run test:e2e)"
        status: pass
    human_judgment: true
  - id: D3
    description: "The obvious-bug fixture PR produces a stored finding at calc.py line 7 RIGHT"
    requirement: REV-03
    verification:
      - kind: test
        ref: "tests::live::prs::live_obvious_bug_is_anchored_to_line_7 (mise run test:e2e)"
        status: pass
    human_judgment: true
  - id: D4
    description: "The fork fixture PR is reviewed the same way, the run head SHA equals gh's current headRefOid and the finding is at calc.py line 7 RIGHT"
    requirement: REV-10
    verification:
      - kind: test
        ref: "tests::live::prs::live_fork_pr_is_reviewed_the_same_way (mise run test:e2e)"
        status: pass
    human_judgment: true

duration: 70min
completed: 2026-10-27
status: complete
---

# Phase 1 Plan 12: Live acceptance and phase gate Summary

**Four #[ignore] live tests drive review::run with the real claude and gh against the owner's fixture PRs and the turn-limit case, all four passed on their single run, and every phase gate except `mise run vuln` (pending the MIT-0 owner decision) passes**

## Performance

- **Duration:** about 70 min (start time was not recorded; estimated from the session)
- **Tasks:** 2 plus one dependency fix-forward commit
- **Files:** 5 created (247 lines in total), 1 modified in src, 4 dependency files moved

## Accomplishments

- `src/tests/live/support.rs`: `live_config` (temp state and cache roots, the real inherited environment), `load_live_prs`, `RealStateGuard` (existence, length and modification time of the real database and runs directory), `review_url` (runs `review::run`, prints the stored run, asserts the guard).
- `src/tests/live/stored.rs`: `StoredRun`, `latest_run` over the temp database (read only), `describe` through `render::sanitize`, `has_finding_at`.
- `src/tests/live/agent.rs` (REV-05) and `src/tests/live/prs.rs` (REV-03, REV-04, REV-10).

## Live results (each test ran once, none retried)

Command: `mise run test:e2e -- --success-output final -E '<filter>'`.

| Test | Result | Duration | Observed |
|------|--------|----------|----------|
| live_max_turns_one_is_failed_not_no_findings | PASS | 8.2 s | `failed (agent_error)`, subtype `error_max_turns`, exit_code 1, terminal_reason `max_turns`, message "Reached maximum number of turns (1)" |
| live_clean_pr_has_no_findings | PASS | 30.8 s | PR 2 at 3a226759707c: `succeeded (no_findings)`; limitation "CI reported no check runs for the head commit" |
| live_obvious_bug_is_anchored_to_line_7 | PASS | 30.4 s | PR 1 at 5c5924713de3: `succeeded (findings: 1)`; `calc.py:7 [high] last_item indexes one past the end` (RIGHT) |
| live_fork_pr_is_reviewed_the_same_way | PASS | 34.0 s | PR 3: head 3d5320d144aea2c7420a9c310df7d24a9901a9ee equals gh's headRefOid; `calc.py:7 [high] first_item returns the second element` (RIGHT) |

The RealStateGuard assertion ran at the end of each PR test and did not fire; the agent test uses only a TempDir. The Task 1 pair ran in one invocation, the Task 2 pair in another.

The coderabbitai[bot] issue comments on the fixture PRs (01-02 note) were in context/discussion.md as untrusted data. None of the four outputs refers to them, and nothing indicates they changed a result.

## Phase gate results

| Command | Exit | Result |
|---------|------|--------|
| `mise run coverage` (before and after the dependency update) | 0 | 164 tests run, 164 passed, 11 skipped; line coverage 93.75% (2895 lines, 181 missed); region 91.39%, function 94.17%; gate is 80% |
| `mise run --force lint` (rustfmt, clippy -D warnings, cargo-machete, gitleaks, actionlint) | 0 | no findings, no leaks, no unused dependencies |
| `mise run vuln` | 4 | advisories ok, bans ok, sources ok, licenses FAILED on MIT-0 only. Pending owner decision (MIT-0): not a fix-forward failure; deny.toml was not edited |
| `mise run --force docs:check` | 0 | 10 links total, 0 errors |
| `mise run deps:check` | 1 before, 0 after | before: hk 2.7.0 to 2.8.1, pkl 0.32.1 to 0.32.2, nextest 0.9.148 to 0.9.149, jsonschema 0.59.0 to 0.60.1, tokio 1.53.3 to 1.54.1; after `deps:update` and the commit: "Rust 1.99.1, mise tools and direct crates are on their latest eligible releases" |
| `mise run test:e2e -- -E 'not test(tests::live::)'` | 0 | 7 run, 7 passed: 3 probe live tests, REV-06, REV-07, REV-08 and isolation_flags_keep_authentication |
| the four live tests | pass | once each, see above |

After the dependency commit: lint exit 0, docs:check exit 0, deps:check exit 0, coverage exit 0 (93.75% lines), vuln still MIT-0 only, and the commit hook ran 164 passing tests.

Not run: the plain unfiltered `mise run test:e2e`. It would repeat the four live model calls, which the owner's instruction and the plan prohibition forbid; the other seven ignored tests were run separately and the four live ones in their own runs.

## Task Commits

1. **Task 1: live support, REV-05 turn limit and REV-04 clean PR** - `32a2a11`
2. **Task 2: REV-03 obvious bug and REV-10 fork PR** - `3c48c17`
3. **Gate fix-forward: dependency update** - `5b662c5`

## Deviations from Plan

**1. [Rule 3 - Blocking] Extra file src/tests/live/stored.rs**
- **Found during:** Task 1
- **Issue:** support.rs with `StoredRun`, `latest_run` and its helpers was over the 100-line limit.
- **Fix:** moved `StoredRun`, `describe`, `has_finding_at` and `latest_run` into `stored.rs`; `support.rs` keeps the config, loader, guard and `review_url`. `latest_run` takes the database path instead of the root.
- **Commit:** `32a2a11`, `3c48c17`

**2. [Rule 3 - Blocking] has_finding_at added in Task 2, not Task 1**
- **Issue:** under `-D warnings` an unused method fails the Task 1 hook.
- **Fix:** the method arrives with its first caller in `3c48c17`.

**3. [Gate fix-forward] Dependencies moved with deps:update**
- **Issue:** `deps:check` reported five items behind (listed above).
- **Fix:** `mise run deps:update`. `mise upgrade --bump` rewrote `hk = "2.8.1"` as `{ version = "2.8.1", workflow = "release.yml" }`, which made `scripts/deps.sh` stop with "unexpected version ''". I set `hk = "2.8.1"` again (mise installs it fine) and reran the script, which updated hk.pkl, jsonschema and tokio. No source change was needed for jsonschema 0.60.1 or tokio 1.54.1. scripts/deps.sh itself was not changed; if mise keeps writing the table form on the next hk bump the script needs a tolerant parser.
- **Commit:** `5b662c5`

**4. [Instruction] Gate `test:e2e` run excluded the live tests** - see Phase gate results.

**Total deviations:** 3 auto-fixed, 1 instruction-driven. **Impact:** none on the requirements.

## Issues Encountered

- `mise run vuln` fails on licenses only (MIT-0 for thiserror-impl, tokio-macros and borrow-or-share), as recorded in deferred-items.md. The owner decides whether to add "MIT-0" to deny.toml [licenses].allow. cargo-deny also prints duplicate-crate warnings for hashbrown and syn; they do not fail the gate.
- A command in my first gate loop used `"--force lint"` as one word under zsh and mise rejected it; the rerun with separate words passed. No tool state changed.

## Owner hand checks (end of phase)

Fixture URLs are in tests/fixtures/live-prs.json (BUG_URL is PR 1, CLEAN_URL PR 2, FORK_URL PR 3).

1. Criterion 1 and 6: run `diffmate review BUG_URL` and `diffmate review FORK_URL`. Each should exit 0 and print a block starting `calc.py:7 [` with a title about the out-of-range index.
2. Criterion 2: run `diffmate review CLEAN_URL`. Expect exit 0 and a first line ending `succeeded (no_findings)`; limitation and improvement lines are fine.
3. Criterion 8: terminal 1 `diffmate review BUG_URL`; while it runs, terminal 2 `diffmate review CLEAN_URL`; terminal 3 `diffmate status`. Terminal 2 prints a line starting `waiting:` and finishes after terminal 1; terminal 3 returns at once and lists the first target running and the second queued.
4. Criterion 4: read the isolation evidence in 01-11-SUMMARY.md and the e2e output above: for REV-06, REV-07, REV-08 each control shows the canary or marker and each isolated run shows none; isolation_flags_keep_authentication passed (observed in this plan's e2e run: REV-06, REV-07, REV-08 and the authentication test all PASS).

Live model output is judgment: if a hand check differs from the stored results above, the owner decides; no test was weakened or retried.

## PR handoff

The orchestrator opens the phase PR after phase verification; this plan opened none and committed nothing to main.

- Branch `gsd/phase-01-single-review` lands through `gh pr create --assignee @me`, then `gh pr comment NUMBER --body "@codex review"`.
- The PR body carries no agent attribution and no "Generated with" footer. It should hold the design rationale gathered in the plan SUMMARYs: the crate table from the Package Legitimacy Audit in 01-RESEARCH.md (why each non-SPEC crate), `core.symlinks false` (01-06), the canonical-URL check (01-04), the context cap values (01-09: pr.md body 64 KiB, diff.patch 1 MiB, discussion.md 256 KiB, ci.md 128 KiB, each rules file 128 KiB, chosen by the planner and flagged for the owner), interrupted runs with a null cause until Phase 2, and the dead-code allowance in tests/support (01-03) plus `#[expect(dead_code)]` on `ReviewResult::run_id`.
- Also mention: the dependency moves in `5b662c5`, the MIT-0 vuln result pending the owner, and the CodeRabbit comments on the private fixture PRs.
- The uncommitted owner edits to .planning/config.json and 01-11-SUMMARY.md, and the untracked .planning/milestone.lock and .planning/state.json, were left untouched.

## Known Stubs

None.

## Threat Flags

None. The live tests add no network endpoint or file access beyond temp directories; T-01-52 is mitigated by RealStateGuard, T-01-53 by #[ignore], T-01-54 by `render::sanitize` on printed findings.

## Self-Check: PASSED

- Files: src/tests/live.rs, live/support.rs, live/stored.rs, live/agent.rs, live/prs.rs present.
- Commits `32a2a11`, `3c48c17` and `5b662c5` are ancestors of HEAD.
