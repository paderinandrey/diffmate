---
phase: 01-single-review
verified: 2026-10-26T22:04:24Z
status: passed
score: 8/8 must-haves verified
covered_files:
  - .planning/phases/01-single-review/01-01-PLAN.md
  - .planning/phases/01-single-review/01-01-SUMMARY.md
  - .planning/phases/01-single-review/01-02-PLAN.md
  - .planning/phases/01-single-review/01-02-SUMMARY.md
  - .planning/phases/01-single-review/01-03-PLAN.md
  - .planning/phases/01-single-review/01-03-SUMMARY.md
  - .planning/phases/01-single-review/01-04-PLAN.md
  - .planning/phases/01-single-review/01-04-SUMMARY.md
  - .planning/phases/01-single-review/01-05-PLAN.md
  - .planning/phases/01-single-review/01-05-SUMMARY.md
  - .planning/phases/01-single-review/01-06-PLAN.md
  - .planning/phases/01-single-review/01-06-SUMMARY.md
  - .planning/phases/01-single-review/01-07-PLAN.md
  - .planning/phases/01-single-review/01-07-SUMMARY.md
  - .planning/phases/01-single-review/01-08-PLAN.md
  - .planning/phases/01-single-review/01-08-SUMMARY.md
  - .planning/phases/01-single-review/01-09-PLAN.md
  - .planning/phases/01-single-review/01-09-SUMMARY.md
  - .planning/phases/01-single-review/01-10-PLAN.md
  - .planning/phases/01-single-review/01-10-SUMMARY.md
  - .planning/phases/01-single-review/01-11-PLAN.md
  - .planning/phases/01-single-review/01-11-SUMMARY.md
  - .planning/phases/01-single-review/01-12-PLAN.md
  - .planning/phases/01-single-review/01-12-SUMMARY.md
  - Cargo.lock
  - Cargo.toml
  - SPEC.md
  - hk.pkl
  - mise.toml
  - src/agent.rs
  - src/agent/claude.rs
  - src/agent/claude/cancel_tests.rs
  - src/agent/claude/contract.rs
  - src/agent/claude/contract_tests.rs
  - src/agent/claude/env_tests.rs
  - src/agent/claude/tests.rs
  - src/agent/envelope.rs
  - src/agent/envelope/fields.rs
  - src/agent/envelope/golden_tests.rs
  - src/agent/envelope/property_tests.rs
  - src/agent/envelope/support.rs
  - src/agent/envelope/synthetic_tests.rs
  - src/agent/envelope/tests.rs
  - src/agent/gitwrap.rs
  - src/agent/gitwrap/tests.rs
  - src/agent/output.rs
  - src/agent/probe.rs
  - src/agent/probe/live_tests.rs
  - src/agent/probe/run.rs
  - src/agent/probe/tests.rs
  - src/agent/request.rs
  - src/agent/schema.rs
  - src/agent/schema/generated_tests.rs
  - src/agent/schema/tests.rs
  - src/agent/spawn.rs
  - src/agent/stop.rs
  - src/agent/stop/tests.rs
  - src/app.rs
  - src/cancel.rs
  - src/cancel/tests.rs
  - src/cli.rs
  - src/context.rs
  - src/context/caps.rs
  - src/context/ci.rs
  - src/context/discussion.rs
  - src/context/files.rs
  - src/context/pr.rs
  - src/context/previous.rs
  - src/context/rules.rs
  - src/context/tests.rs
  - src/context/tests/caps_cases.rs
  - src/context/tests/ci_cases.rs
  - src/context/tests/ci_unknown.rs
  - src/context/tests/cut_property.rs
  - src/context/tests/discussion_cases.rs
  - src/context/tests/discussion_limits.rs
  - src/context/tests/fake_gh.rs
  - src/context/tests/previous_shapes.rs
  - src/diff.rs
  - src/diff/hunk.rs
  - src/diff/path.rs
  - src/diff/tests.rs
  - src/diff/tests/model.rs
  - src/diff/tests/never_panics.rs
  - src/diff/tests/place.rs
  - src/diff/tests/roundtrip.rs
  - src/error.rs
  - src/gh.rs
  - src/gh/checks.rs
  - src/gh/discussion.rs
  - src/gh/discussion/raw.rs
  - src/gh/discussion/raw/shapes.rs
  - src/gh/discussion/tests.rs
  - src/gh/meta.rs
  - src/git.rs
  - src/git/build.rs
  - src/git/cli.rs
  - src/git/config.rs
  - src/git/config_tests.rs
  - src/git/fixture.rs
  - src/git/fixture/poison.rs
  - src/git/isolation_tests.rs
  - src/git/refname.rs
  - src/git/refname/tests.rs
  - src/git/tests.rs
  - src/git/tree.rs
  - src/git/tree_tests.rs
  - src/git/worktree_tests.rs
  - src/lock.rs
  - src/lock/tests.rs
  - src/main.rs
  - src/normalize.rs
  - src/normalize/tests.rs
  - src/normalize/tests/fixture.rs
  - src/normalize/tests/property.rs
  - src/paths.rs
  - src/pr_url.rs
  - src/pr_url/tests.rs
  - src/proc.rs
  - src/procenv.rs
  - src/recovery.rs
  - src/recovery/fake.rs
  - src/recovery/scan.rs
  - src/recovery/tests.rs
  - src/render.rs
  - src/render/findings.rs
  - src/render/sanitize.rs
  - src/render/tests.rs
  - src/render/tests/fixture.rs
  - src/render/tests/sanitize.rs
  - src/review.rs
  - src/review/signals.rs
  - src/runner.rs
  - src/runner/agent_run.rs
  - src/runner/cleanup.rs
  - src/runner/outcome.rs
  - src/runner/resolve.rs
  - src/runner/result.rs
  - src/runner/steps.rs
  - src/runner/tests.rs
  - src/status.rs
  - src/store.rs
  - src/store/finish.rs
  - src/store/jobs.rs
  - src/store/previous.rs
  - src/store/readonly.rs
  - src/store/records.rs
  - src/store/recovery.rs
  - src/store/runs.rs
  - src/store/schema.sql
  - src/store/status.rs
  - src/tests.rs
  - src/tests/isolation.rs
  - src/tests/isolation/auth.rs
  - src/tests/isolation/contract.rs
  - src/tests/isolation/endpoint.rs
  - src/tests/isolation/endpoint_check.rs
  - src/tests/isolation/fixture.rs
  - src/tests/isolation/fixture/context.rs
  - src/tests/isolation/fixture/files.rs
  - src/tests/isolation/fixture/home.rs
  - src/tests/isolation/fixture/layout.rs
  - src/tests/isolation/fixture/ops.rs
  - src/tests/isolation/fixture/scripts.rs
  - src/tests/isolation/fixture/worktree.rs
  - src/tests/isolation/fixture_check.rs
  - src/tests/isolation/harness.rs
  - src/tests/isolation/harness_check.rs
  - src/tests/isolation/http.rs
  - src/tests/isolation/inspect.rs
  - src/tests/isolation/rev06.rs
  - src/tests/isolation/rev07.rs
  - src/tests/isolation/rev08.rs
  - src/tests/live.rs
  - src/tests/live/agent.rs
  - src/tests/live/prs.rs
  - src/tests/live/stored.rs
  - src/tests/live/support.rs
  - tests/agent_contract.rs
  - tests/agent_contract/classification.rs
  - tests/cli.rs
  - tests/concurrency.rs
  - tests/concurrency/helpers.rs
  - tests/context.rs
  - tests/context/helpers.rs
  - tests/context_rules.rs
  - tests/fixtures/claude/cases.json
  - tests/fixtures/claude/contract.json
  - tests/fixtures/claude/error-max-budget.stdout
  - tests/fixtures/claude/error-max-turns.stdout
  - tests/fixtures/claude/error-not-logged-in.stdout
  - tests/fixtures/claude/error-unknown-model.stdout
  - tests/fixtures/claude/success-findings.stdout
  - tests/fixtures/claude/success-no-findings.stdout
  - tests/fixtures/live-prs.json
  - tests/fork.rs
  - tests/git_isolation.rs
  - tests/harness.rs
  - tests/recovery.rs
  - tests/review.rs
  - tests/review_output.rs
  - tests/review_output/helpers.rs
  - tests/review_output/render.rs
  - tests/signals.rs
  - tests/status.rs
  - tests/support/envelope.rs
  - tests/support/fake_call.rs
  - tests/support/fake_claude.rs
  - tests/support/fake_claude.sh
  - tests/support/fake_gh.rs
  - tests/support/fake_gh.sh
  - tests/support/mod.rs
  - tests/support/poison.rs
  - tests/support/sandbox.rs
  - tests/support/upstream.rs
  - tests/tracer/agent_call.rs
  - tests/tracer/mod.rs
  - tests/tracer/stored.rs
covered_digest: "v3:sha256:194169675b08a0c2219343576e9cc98527312f805245e2bc209cf732c913118f"
behavior_unverified: 0
overrides_applied: 0
human_verification:
  - test: "Criteria 1 and 6: run `diffmate review` on the obvious-bug fixture PR (https://github.com/paderinandrey/diffmate-fixtures/pull/1) and the fork fixture PR (.../pull/3)"
    expected: "Each exits 0 and prints a block starting `calc.py:7 [` with a title about the out-of-range index"
    why_human: "Live model output is judgment. The live tests passed once each (01-12 SUMMARY) and were not re-run by the verifier by policy."
  - test: "Criterion 2: run `diffmate review` on the clean fixture PR (.../pull/2)"
    expected: "Exit 0 and a first line ending `succeeded (no_findings)`; limitation and improvement lines are fine"
    why_human: "Same as above: one live model call, judgment on whether the clean verdict is honest"
  - test: "Criterion 8: terminal 1 `diffmate review BUG_URL`; while it runs, terminal 2 `diffmate review CLEAN_URL`; terminal 3 `diffmate status`"
    expected: "Terminal 2 prints a line starting `waiting:` and finishes after terminal 1; terminal 3 returns at once and lists the first target running and the second queued"
    why_human: "Real-process timing across three terminals. The automated twin (tests/concurrency.rs) passes against fakes."
  - test: "Criterion 4: read the isolation evidence in 01-11-SUMMARY.md and the `mise run test:e2e` output for REV-06, REV-07, REV-08 and isolation_flags_keep_authentication"
    expected: "Each control shows its canary or marker, each isolated run shows none, authentication passes under the production flags. Also accept that tool-level confinement (an agent Read of a path outside the worktree, or through a symlink) has no direct control (01-11 coverage D6)."
    why_human: "The isolation tests are #[ignore] and need the real claude CLI; the verifier did not re-run them by policy. The remaining coverage gap (D6) is an owner risk acceptance."
  - test: "Owner decision on `mise run vuln`: add `MIT-0` to deny.toml [licenses].allow (thiserror-impl 2.0.21, tokio-macros, borrow-or-share), or choose another remedy"
    expected: "`mise run vuln` exits 0. Today advisories, bans and sources are ok and licenses FAILED on MIT-0 only."
    why_human: "deny.toml policy is the owner's call. Not a phase success criterion, but the CLAUDE.md verification gate list includes `mise run vuln`."
  - test: "Commit or discard the owner edit in .planning/phases/01-single-review/01-11-SUMMARY.md (records acceptance of the substitute controls, option a, 2026-10-22)"
    expected: "Working tree clean apart from the untracked planning state files"
    why_human: "Uncommitted owner edit; the verifier must not stage it"
---

# Phase 1: Single review Verification Report

**Phase Goal:** I can run `diffmate review <url>` on a PR and get an honest, line-anchored draft from an agent that provably cannot run commands, write files, or load anything from the PR, and a failure is never shown as "no findings".
**Verified:** 2026-10-27
**Status:** human_needed
**Re-verification:** No, initial verification

No code gap found. All eight ROADMAP success criteria are backed by code and by tests that passed. The status is `human_needed` because the owner's hand checks are still open, the `vuln` gate has an undecided license policy, and the model-dependent and isolation e2e tests were not re-run by the verifier.

## Goal Achievement

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A PR with an obvious bug produces a finding anchored to the right line [REV-03] | VERIFIED (test-evidenced) | `src/normalize.rs` keeps `line`/`side` only when `DiffIndex::contains(path, side, line)`; otherwise moves the finding to the body with a warning. `src/diff/hunk.rs` builds the index (proptest/roundtrip tests in `src/diff/tests/`). Live test `live_obvious_bug_is_anchored_to_line_7` (src/tests/live/prs.rs) asserts a stored finding at calc.py line 7 RIGHT from the fixture in tests/fixtures/live-prs.json. 01-12 SUMMARY records it passing once (PR 1, `calc.py:7 [high]`). Not re-run by the verifier. Owner hand check pending. |
| 2 | A clean PR produces `no_findings` [REV-04] | VERIFIED (test-evidenced) | `src/render.rs` prints `succeeded (no_findings)` only for a `Succeeded` result with zero findings. `src/agent/envelope.rs::structured` enforces that `status` agrees with `findings`. Live test `live_clean_pr_has_no_findings` asserts state `succeeded` and result `no_findings`; recorded passing once. Offline twin in tests/review_output.rs. Not re-run. |
| 3 | An agent error with exit code 0 (e.g. a turn limit) produces `failed`, not `no_findings` [REV-05] | VERIFIED | `classify()` in `src/agent/envelope.rs` fails on `is_error`, a `subtype` other than `success`, or non-zero exit before it ever reads `structured_output`. `tests/agent_contract/classification.rs::error_envelope_with_exit_zero_is_failed` feeds the recorded `error-max-turns.stdout` with exit 0, and asserts exit 1, `failed (agent_error)`, no `no_findings` text, and zero drafts. Proptest `success_requires_all_four_checks` pins the four-check rule. Live `--max-turns 1` test asserts `AgentError` with `error_max_turns` (real exit code is 1, per contract.json; the exit-0 case is covered by the recorded-envelope tests). Ran in `mise run coverage` (164 passed). |
| 4 | Claude isolation with canaries and working auth: CLAUDE.md not loaded [REV-06]; `@` import does not leak [REV-07]; hook, plugin, `.mcp.json` do not run or load [REV-08] | VERIFIED (test-evidenced), owner hand check pending | Production vector in `src/agent/claude/contract.rs::production_args` contains `--safe-mode --restricted --strict-mcp-config --tools Read,Grep,Glob --no-session-persistence --permission-prompts none --disable-slash-commands`. Agent env is the `AUTH_ENV` allowlist plus `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_CONFIG_NOSYSTEM=1`, with `env_clear()` in `src/agent/spawn.rs`; no `GH_TOKEN`/`GITHUB_TOKEN` in it. PATH starts with a `git` wrapper (`src/agent/gitwrap.rs`). Tests `rev06.rs`, `rev07.rs`, `rev08.rs` and `auth.rs` in `src/tests/isolation/` each assert a positive control first (message "control vacuous") and require the isolated run to have reached `POST /v1/messages` before accepting absence. The isolated vector is the real `ClaudeCli` with only env overrides. They are `#[ignore]` (real claude CLI), reported passing in 01-11 and 01-12 and by the orchestrator; the verifier did not re-run them. Limits: `rules/CLAUDE.md` as an add-dir memory file and agents/commands/skills fire only in substitute controls (root `CLAUDE.md` with `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1`; an open control without `--tools`/`--disable-slash-commands`). The owner accepted these on 2026-10-22 (option a); the edit recording that is uncommitted. Tool-level confinement (D6) has no direct control. |
| 5 | Git isolation: a smudge filter and `core.fsmonitor` in a test global config, plus `.gitattributes` selecting the filter, do not run during fetch, worktree checkout or diff [REV-09] | VERIFIED | `src/git/cli.rs` clears the env and sets `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_CONFIG_NOSYSTEM=1`; `src/git/config.rs` writes `core.hooksPath` (verified empty), `core.fsmonitor=false`, `core.symlinks=false`; `src/git/build.rs` uses `--no-ext-diff --no-textconv`. `src/git/isolation_tests.rs` and `tests/git_isolation.rs` poison HOME, XDG, `GIT_CONFIG_GLOBAL`, `GIT_CONFIG_PARAMETERS` and `GIT_CONFIG_COUNT/KEY/VALUE` with filters, extdiff, textconv, fsmonitor and a post-checkout hook, run a `control()` that proves all five markers fire under plain git, then assert zero markers after fetch, worktree add, status, diff and removal. These are non-ignored and passed in this verification's coverage run. |
| 6 | A PR from a fork is reviewed the same way [REV-10] | VERIFIED (test-evidenced) | `tests/fork.rs::fork_pr_head_only_under_pull_ref_is_reviewed` has a head reachable only under `refs/pull/7/head` (asserted with `branch --contains`), fetches `+refs/pull/{n}/head:refs/diffmate/pull/{n}/head` from the base repository (`src/git/build.rs::fetch_pr`), and stores the head SHA. `unsafe_base_branch_fails_before_fetch` rejects ref-injection in the base branch. Live `live_fork_pr_is_reviewed_the_same_way` checks the run head SHA equals gh's `headRefOid` and a finding at calc.py:7 RIGHT; recorded passing once. Not re-run. |
| 7 | A missing CLI flag fails the run before the agent starts [REV-11] | VERIFIED | `src/agent/spawn.rs::invoke` runs `probe::check_version` and `probe::check_flags` (production argument vector, empty stdin) before `start()`; the probe is fail-closed (`inconclusive` is a refusal). `tests/agent_contract.rs::missing_flag_fails_before_agent_starts` asserts exit 1, `failed (cli_error)`, the flag name, and `sandbox.claude.run_calls().is_empty()`; old version, unreadable version and inconclusive probe are covered the same way. Probe text is pinned to contract.json by `fake_probe_text_matches_spike`. |
| 8 | A second `review` while the first holds the lock waits, its job runs after the first; `status` works meanwhile [REV-12] | VERIFIED, owner hand check pending | `src/lock.rs` uses an exclusive OS file lock (`File::try_lock`) with a polling `acquire`; `src/review.rs` enqueues first, prints `waiting:` on first contention, then takes the lock, runs recovery and its own job. `status` uses `Store::open_read_only` and never recovers. `tests/concurrency.rs::second_review_waits_for_the_lock_and_status_works` holds run 1 inside the fake claude, asserts the second prints `waiting:`, `status` returns `o/r#1  untracked  running` / `o/r#2  untracked  queued`, the second has no run and a `queued` job, and after release run 2 is created at or after run 1 finished. `tests/recovery.rs` covers a killed runner. |

**Score:** 8/8 truths verified (0 present, behavior-unverified). Six of them (1, 2, 4, 6 and the live halves of 3 and 8) rest partly on e2e results that the verifier did not reproduce; they are carried to human verification rather than counted as unproven.

### Failure is never shown as "no findings" (goal clause)

`src/render.rs` maps `RunFailed` and `JobFailed` to `failed (<kind>): ...` on stderr, `Cancelled` to a run-cancelled line, and `Succeeded` with zero findings to `no_findings`. `src/runner/outcome.rs` sends every `AgentOutcome::Failed` and `Cancelled` through `finish_failed` and never constructs `Succeeded` without a validated `ReviewOutput`. Observed with the release binary and an empty HOME: `diffmate review <url>` with no gh auth printed `failed (cli_error): gh pr view ...` and exited 1; a non-PR URL exited 2; `status` with no database printed `No targets.` and exited 0.

### Deferred Items

None. The signal-handling gaps relative to full SPEC wording (a waiting `review` printing "Stopped waiting; the job stays queued", `interrupted` causes `shutdown`/`user_cancel` stored on the run, job attach/merge, run timeout) belong to Phase 2 criteria (REL-07 to REL-12), not Phase 1 criteria 1 to 8.

### Required Artifacts

`gsd_run query verify.artifacts` on all 12 plans: every artifact exists and is substantive except two pattern misses that are file moves, not stubs. Checked by hand:

| Artifact | Check | Status |
|----------|-------|--------|
| `src/agent/claude.rs` (01-04, pattern `--disable-slash-commands`) | The flag lives in `src/agent/claude/contract.rs::production_args` | VERIFIED |
| `src/render.rs` (01-07, pattern `Unanchored findings:`) | Lives in `src/render/findings.rs:29`; asserted in `src/render/tests.rs` | VERIFIED |
| `schemas/review.schema.json`, `prompts/review.md` | Present; `schema_form` is `full` in contract.json, so no `review.cli.schema.json` is required (and none exists) | VERIFIED |
| `tests/fixtures/claude/*` | contract.json, cases.json and six recorded envelopes present; used by golden tests | VERIFIED |
| `tests/fixtures/live-prs.json` | Three fixture PRs with SHAs and anchors; read by `src/tests/live/support.rs` | VERIFIED |
| `src/tests/live/*`, `src/tests/isolation/*` | Present, `#[ignore = "..."]` with reasons, skipped by `mise run test` (11 skipped) | VERIFIED |

### Key Link Verification

`verify.key-links` reported 22 of 28 links wired across the plans. Each of the 6 misses was checked by hand and is a pattern that moved to a submodule during the 100-line split, not a broken link:

| From | To | Via | Status |
|------|----|-----|--------|
| tests/fixtures/live-prs.json | src/tests/live | `src/tests/live/support.rs:49` reads it | WIRED |
| src/tests/live/prs.rs | src/review.rs | `support.rs:57` calls `crate::review::run` | WIRED |
| src/agent/envelope/tests.rs | cases.json | `golden_tests.rs:37` reads it | WIRED |
| src/git.rs | src/git/config.rs | `src/git/cli.rs:28` `config::write` | WIRED |
| src/tests/isolation/fixture.rs | contract.json | `src/tests/isolation/contract.rs:3` `include_str!` | WIRED |
| `main.rs` -> `review::run` -> `runner::execute_review_job` -> `ClaudeCli` -> `store` | the review pipeline | `src/main.rs:46`, `src/review.rs`, `src/runner.rs` | WIRED |

### Data-Flow Trace (Level 4)

| Artifact | Data | Source | Real data | Status |
|----------|------|--------|-----------|--------|
| `render::review` findings | `ReviewResult::Succeeded.findings` | `normalize()` output of the schema-validated `structured_output`, the same rows written by `finish_succeeded` | Yes | FLOWING |
| `status` rows | `target_statuses()` | read-only SQLite query | Yes | FLOWING |
| `diff.patch` / hunk index | `git diff --no-ext-diff --no-textconv <merge-base> <head>` from the bare clone | Yes | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Whole suite and coverage gate | `mise run coverage` | 164 tests run, 164 passed, 11 skipped; 93.75% line coverage (gate 80%) | PASS |
| Lint | `mise run --force lint` | rustfmt, clippy `-D warnings`, cargo-machete, gitleaks, actionlint all clean | PASS |
| Docs | `mise run --force docs:check` | 10 links, 0 errors | PASS |
| Dependencies current | `mise run deps:check` | all on latest eligible releases | PASS |
| Release build | `mise run build` | `target/release/diffmate` built | PASS |
| CLI surface | `diffmate --help`, `status`, `review not-a-url`, `review <url>` without gh auth (empty temp HOME) | subcommands `review` and `status`; `No targets.` exit 0; exit 2 on a bad URL; `failed (cli_error)` exit 1 | PASS |
| Vulnerability and license policy | `mise run vuln` | exit 4: advisories ok, bans ok, sources ok, licenses FAILED (MIT-0 for thiserror-impl 2.0.21 and others) | FAIL, owner decision pending |
| Live e2e (REV-03/04/05/10) and isolation e2e (REV-06/07/08, auth) | not run | policy: each live check runs once; results are those in 01-11 and 01-12 SUMMARYs | SKIPPED |

### Probe Execution

SKIPPED: no `scripts/*/tests/probe-*.sh` exist and no plan declares one. The Claude CLI flag probe is part of the product (`src/agent/probe`), covered above.

### Requirements Coverage

Every REV ID is claimed by at least one plan's `requirements:` field and appears in REQUIREMENTS.md (all marked Complete in the traceability table). No orphaned Phase 1 requirement.

| Requirement | Source plan(s) | Description | Status | Evidence |
|-------------|----------------|-------------|--------|----------|
| REV-01 | 01-01, 01-05, 01-11 | Spike: isolation checks, envelope fields, full schema through `--json-schema` | SATISFIED | tests/fixtures/claude/contract.json (CLI 2.1.304; `schema_form: full`, envelope fields, isolation first look, controls, `claude_git_calls`); schema loader in `src/agent/schema.rs` removes `$schema` for the CLI and validates locally with the full schema |
| REV-02 | 01-03, 01-04, 01-07, 01-08, 01-09, 01-10 | `review` and `status` end to end | SATISFIED | `src/review.rs` pipeline; tests/review.rs, tests/context*.rs, tests/status.rs, tests/recovery.rs, tests/signals.rs; release binary exercised |
| REV-03 | 01-02, 01-07, 01-12 | Obvious bug anchored to the right line | SATISFIED (live result recorded, hand check pending) | Truth 1 |
| REV-04 | 01-02, 01-05, 01-07, 01-12 | Clean PR yields `no_findings` | SATISFIED (live result recorded, hand check pending) | Truth 2 |
| REV-05 | 01-05, 01-12 | Agent error with exit 0 is `failed` | SATISFIED | Truth 3 |
| REV-06 | 01-11 | Worktree CLAUDE.md not loaded | SATISFIED (e2e result recorded, not re-run) | Truth 4 |
| REV-07 | 01-11 | `@` import does not leak | SATISFIED (e2e result recorded, not re-run) | Truth 4 |
| REV-08 | 01-11 | Hook, plugin, `.mcp.json` do not run or load | SATISFIED (e2e result recorded, not re-run) | Truth 4 |
| REV-09 | 01-06 | Git filter and fsmonitor do not run | SATISFIED | Truth 5 |
| REV-10 | 01-02, 01-06, 01-12 | Fork PR reviewed the same way | SATISFIED (live result recorded, hand check pending) | Truth 6 |
| REV-11 | 01-05 | Missing flag fails before agent start | SATISFIED | Truth 7 |
| REV-12 | 01-08 | Second review waits; status works | SATISFIED | Truth 8 |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src` (non-test) | n/a | `unwrap`, `expect`, `panic!`, `unsafe`: only in `cfg(test)` support modules (`src/recovery/fake.rs`, `src/agent/envelope/support.rs`); `unsafe_code = "forbid"` in Cargo.toml | Info | Conforms to CLAUDE.md |
| repo | n/a | `TBD`, `FIXME`, `XXX`, `TODO`, `HACK`, `todo!`, `unimplemented!`: none in src, tests, prompts, schemas or scripts | None | No debt markers |
| `deny.toml` | n/a | `cargo deny` license allow list lacks `MIT-0` | Warning | `mise run vuln` fails on licenses only (see human verification) |
| `.planning/phases/01-single-review/01-11-SUMMARY.md` | n/a | Uncommitted owner edit | Info | Do not stage; owner to commit |
| `SPEC.md` | n/a | The branch changes SPEC.md in commits `ae373b7` and `f1f4106` (failure kinds, two extra isolation flags, git wrapper PATH). The commit messages attribute them to owner decisions. | Info | The verifier cannot confirm the owner decision from the repo; flagged because CLAUDE.md allows SPEC changes only on the owner's decision |

No stubs: the components that render dynamic data are fed by real queries or real validated agent output; "Known Stubs: None" in 01-11 and 01-12 SUMMARYs matches the grep results.

### Coincidental reliance (advisory)

None recorded. The isolated runs receive the same trust grants as their controls, which makes the negatives stricter, not weaker.

### Human Verification Required

See the `human_verification` list in the frontmatter. The first four are the owner hand checks from 01-12 SUMMARY for ROADMAP criteria 1 and 6, 2, 8 and 4. The last two are the open owner decisions (MIT-0 policy and the uncommitted 01-11-SUMMARY edit).

### Gaps Summary

No gaps against the Phase 1 success criteria. Residual risks to carry forward:

1. `mise run vuln` is red on a license policy question only.
2. The model-dependent and isolation tests are local-only by design, and the verifier took their passing results from the SUMMARYs and the orchestrator rather than from its own run.
3. Isolation evidence is body-substring evidence from a fake endpoint with substitute controls for `rules/CLAUDE.md` and agents/commands/skills; tool-level confinement is not directly exercised. The owner accepted the substitute controls.
4. ROADMAP.md still shows Phase 1 unchecked in the phase list; the orchestrator updates it after sign-off.

---

_Verified: 2026-10-27_
_Verifier: Claude (gsd-verifier)_
