---
phase: 01-single-review
plan: 06
subsystem: git
tags: [rust, git, isolation, refspec, worktree, symlinks, fork-pr, proptest]

requires:
  - phase: 01-single-review
    provides: "01-04 tracer slice (Git trait, GitCli, runner), 01-03 black-box harness (Sandbox, Upstream::pr_head and commit_symlink), 01-05 agent contract"
provides:
  - "Local clone config rewritten on every ensure_clone: core.hooksPath at a checked-empty directory, core.fsmonitor false, core.symlinks false, gc.autoDetach false, maintenance.autoDetach false, remote.origin.url, no default fetch refspec"
  - "validate_branch_name: a base branch that could alter a refspec fails the job before any fetch"
  - "REV-09 as a permanent CI test: plain-git control with five marker kinds, then the same route set through GitCli and through the whole product"
  - "REV-10 black-box fork test: head reachable only through refs/pull/N/head"
  - "End-to-end proof that a PR symlink reaches the agent worktree as a plain file"
affects: [01-07, 01-09, 01-11, 01-12]

actuals:
  tokens: 6900
  tasks: 2
  commits: 2

plan_head_before: 143394771a43b55504d8cb94f8830392fa13a001
plan_head_after: 5194cbe18e9b926eb0206cf977e60d2a34a56dc7
commits: 2

tech-stack:
  added: []
  patterns:
    - "Clone config is written key by key with git config argument vectors on every run; a stray remote.origin.fetch is detected from the key listing and removed"
    - "Poisoned-config tests build the config with git config --file and marker scripts as files, one temp root per test, and fail as control vacuous when the plain-git run does not fire every marker"
    - "FakeClaude::snapshot_tree is opt-in: the fake copies its cwd into the call directory so a test can inspect what the agent saw"

key-files:
  created:
    - src/git/config.rs
    - src/git/refname.rs
    - src/git/refname/tests.rs
    - src/git/tests.rs
    - src/git/config_tests.rs
    - src/git/worktree_tests.rs
    - src/git/isolation_tests.rs
    - src/git/fixture.rs
    - src/git/fixture/poison.rs
    - tests/git_isolation.rs
    - tests/fork.rs
    - tests/support/poison.rs
  modified:
    - src/git.rs
    - src/git/cli.rs
    - src/error.rs
    - tests/support/mod.rs
    - tests/support/fake_claude.rs
    - tests/support/fake_claude.sh

key-decisions:
  - "core.symlinks false is written in the local clone config, which the runner's worktree add shares, so the agent sees a PR symlink as a small file holding the link text; reversible with one config line"
  - "A stray remote.origin.fetch is found by listing local config key names (git config --local --name-only --list) and removed with --unset-all only when present, instead of branching on git's exit status 5"
  - "Branch names are rejected rather than escaped: a subset of git check-ref-format (no leading dash, no component starting with a dot or ending in .lock, no .., @{, trailing dot, or ASCII/Unicode control and whitespace) plus the refspec metacharacters : ? * [ \\ ~ ^"
  - "The hooks directory emptiness check runs first in config::write and fails the clone with HooksNotEmpty; ensure_clone does not empty it"

patterns-established:
  - "Git trait and its signatures are untouched; GitCli::run and GitCli::text are pub(super) in cli.rs so config.rs and the in-crate tests reach them"
  - "Black-box tests that need no helpers from the tracer slice still import mod tracer with #[allow(dead_code)], as tests/agent_contract.rs does"

requirements-completed: [REV-09, REV-10]

coverage:
  - id: D1
    description: "A smudge filter, core.fsmonitor, external diff and textconv drivers and a global core.hooksPath hook, selected by a .gitattributes in the PR head, fire in a plain-git control and do not fire in diffmate's clone, fetch, rev-parse, merge-base, worktree add, status, diff, worktree remove and prune, even with the config offered through GIT_CONFIG_GLOBAL, GIT_CONFIG_PARAMETERS, GIT_CONFIG_COUNT/KEY_0/VALUE_0, GIT_DIR, HOME/.gitconfig and XDG_CONFIG_HOME"
    requirement: REV-09
    verification:
      - kind: unit
        ref: "src/git/isolation_tests.rs#poisoned_configuration_runs_in_the_control_and_never_in_diffmate"
        status: pass
      - kind: integration
        ref: "tests/git_isolation.rs#poisoned_environment_runs_no_git_config"
        status: pass
    human_judgment: false
  - id: D2
    description: "Every git command diffmate builds carries only GIT_CONFIG_GLOBAL=/dev/null, GIT_CONFIG_NOSYSTEM=1 and GIT_TERMINAL_PROMPT=0 as GIT_* variables, whatever the inherited environment holds"
    requirement: REV-09
    verification:
      - kind: unit
        ref: "src/git/isolation_tests.rs#command_environment_carries_no_inherited_git_variable"
        status: pass
    human_judgment: false
  - id: D3
    description: "The local config holds exactly the written keys (plus platform-written core.* keys) after two ensure_clone runs, a pre-existing default fetch refspec is removed, and a non-empty hooks directory fails the clone"
    requirement: REV-09
    verification:
      - kind: unit
        ref: "src/git/config_tests.rs#local_config_has_exactly_the_written_keys_on_every_run"
        status: pass
      - kind: unit
        ref: "src/git/tests.rs#a_non_empty_hooks_directory_fails_the_clone"
        status: pass
    human_judgment: false
  - id: D4
    description: "fetch_pr leaves exactly refs/diffmate/pull/N/head and refs/diffmate/base/BASE; an unsafe base branch is rejected before any fetch, by the validator (proptested) and through the whole product"
    requirement: REV-09
    verification:
      - kind: unit
        ref: "src/git/worktree_tests.rs#fetch_leaves_exactly_the_two_namespaced_refs"
        status: pass
      - kind: unit
        ref: "src/git/refname/tests.rs#names_that_could_alter_a_refspec_are_rejected and #accepted_names_are_safe_in_a_refspec"
        status: pass
      - kind: integration
        ref: "tests/fork.rs#unsafe_base_branch_fails_before_fetch"
        status: pass
    human_judgment: false
  - id: D5
    description: "A PR whose head exists only under refs/pull/N/head (fork case) is fetched, checked out, diffed, reviewed and stored with the right head SHA"
    requirement: REV-10
    verification:
      - kind: integration
        ref: "tests/fork.rs#fork_pr_head_only_under_pull_ref_is_reviewed"
        status: pass
    human_judgment: false
  - id: D6
    description: "A symlink in the PR head is a regular file holding the link text, both in the worktree GitCli creates and in the cwd the agent is started in"
    requirement: REV-09
    verification:
      - kind: unit
        ref: "src/git/worktree_tests.rs#a_pr_symlink_is_checked_out_as_a_plain_file"
        status: pass
      - kind: integration
        ref: "tests/git_isolation.rs#pr_symlink_reaches_the_agent_as_a_plain_file"
        status: pass
    human_judgment: false
  - id: D7
    description: "A real fork PR on GitHub (head repository possibly deleted) behaves the same as the local refs/pull/N/head fixture"
    requirement: REV-10
    verification: []
    human_judgment: true
    rationale: "The live fork PR check belongs to plan 01-12 and needs GitHub and the owner's fixture list"

duration: 12min
completed: 2026-10-20
status: complete
---

# Phase 1 Plan 06: Git isolation and fork PRs Summary

**diffmate's own git use is hardened to the SPEC Git section and proven against a poisoned global config (five config routes, mandatory plain-git control), with refspec-safe branch names, symlinks checked out as plain files, and a fork PR whose head lives only under refs/pull/N/head**

## Performance

- **Duration:** 12 min (2026-10-20T18:21Z to 18:33Z)
- **Tasks:** 2
- **Files:** 18 changed in the plan range (12 created, 6 modified), 837 insertions and 17 deletions

## Accomplishments

- `ensure_clone` now ends with `config::write`: it checks that the hooks directory is empty, then sets `core.hooksPath`, `core.fsmonitor false`, `core.symlinks false`, `gc.autoDetach false`, `maintenance.autoDetach false` and `remote.origin.url` with `git config --local` argument vectors on every run, and removes `remote.origin.fetch` if a previous version or the owner left one. The credential helper stays on `fetch_pr` only.
- `validate_branch_name` runs first in `fetch_pr`. It rejects an empty name, a leading dash, `:`, whitespace and control characters (Unicode included), `..`, `@{`, `@`, glob characters, `\`, `~`, `^`, empty path components (`/a`, `a/`, `a//b`), components starting with `.` or ending in `.lock`, and a trailing dot. The error is `UnsafeBranch`, the runner turns it into a failed job (`failed (cli_error)`) before any ref is fetched.
- REV-09 control: a plain git sequence (init --bare, remote url, fetch with the same refspecs, worktree add, status, diff, diff --no-ext-diff) with `GIT_CONFIG_GLOBAL` at the poisoned config fires all five markers (smudge, fsmonitor, extdiff, textconv, post-checkout hook from a global `core.hooksPath`); the test fails as "control vacuous" otherwise. The isolated run builds `GitCli` from an `InheritedEnv` that carries the poisoned `GIT_CONFIG_GLOBAL`, `GIT_CONFIG_PARAMETERS`, `GIT_CONFIG_COUNT`/`KEY_0`/`VALUE_0`, a decoy `GIT_DIR`, a poisoned `HOME/.gitconfig` and `XDG_CONFIG_HOME/git/config`, runs every Git trait method plus a `status` through the same command builder and finds no marker. The black-box twin does the same through `diffmate review` with the fake gh and claude.
- Mutation check (observed, then reverted): letting `GitCli::new` pass the inherited `GIT_CONFIG_GLOBAL` through makes both the in-crate and the black-box isolation test fail on the marker assertion, so neither is vacuous.
- REV-10: `fork_pr_head_only_under_pull_ref_is_reviewed` pushes PR 7's head only to `refs/pull/7/head` (asserted unreachable from any branch), gh says `isCrossRepository` true with head branch `fork-branch`, and the review prints the finding, stores the PR head as `head_sha` and leaves `refs/diffmate/pull/7/head` at it.
- Symlinks (T-01-26, the 01-04 review note): the worktree comes from `GitCli::worktree_add` after `ensure_clone`, the exact path the runner takes. The unit test checks the checked-out `link` is a regular file whose content is the link target; the black-box test uses the fake claude's opt-in `snapshot_tree` to copy the agent's cwd and checks the same on what the agent was started in.
- Assumption A4: `gc.autoDetach` and `maintenance.autoDetach` are documented keys in the installed git 2.56.0 (`git help -c` lists both and `git help -m config` has entries for both), and `git config` accepts them. Their runtime effect (no detached background `gc` after fetch) is not demonstrated by a test; only the key names are confirmed.

## Task Commits

1. **Task 1: Controlled git environment, local config and refspec safety, with the REV-09 control test** - `629d694` (feat)
2. **Task 2: Black-box git isolation, fork PR and unsafe base branch** - `5194cbe` (test)

**Plan metadata:** the commit that adds this SUMMARY (docs: complete plan)

## Decisions Made

- See `key-decisions` in the frontmatter.
- The command builder stays `GitCli::command` and is private to `crate::git`; the in-crate tests are descendants of that module, so no extra exposure was needed.
- `GitCli::new` already built the SPEC environment in 01-04 (`env_clear`, PATH, HOME, LC_ALL=C, the three git variables, gh variables picked by name), so no inherited `GIT_*` can pass; this plan adds the test that pins it.

## Verification

Observed in this session:

- `mise run --force test -- -E 'test(git::)'`: 10 tests, all passed (before the commit; the hk hook ran the whole suite on both commits: 72 and 76 tests passed, 3 skipped).
- `mise run --force test -- -E 'binary(git_isolation) | binary(fork)'`: 4 passed.
- `mise run --force test`: 76 run, 76 passed, 3 skipped (the live probes).
- `mise run --force lint`: rustfmt, clippy pedantic with warnings denied, cargo-machete, gitleaks and actionlint all exit 0.
- `mise run coverage`: reported 92.43% line coverage in total (gate 80%); no gate-failure message was printed.
- Acceptance greps: `grep -c core.symlinks src/git/config.rs` = 1, `grep -c validate_branch_name src/git.rs` = 1; every file the plan touches is at most 100 lines (git.rs 99, poison.rs 98, cli.rs 90).
- Not run: `mise run vuln` and `mise run deps:check` (no crate was added), `mise run test:e2e`, `mise run docs:check` outside the hook, CI on ubuntu-latest (tests use only POSIX sh, `touch`, `cat` and std unix APIs, as the existing harness does).

## Deviations from Plan

**1. [Rule 3 - Blocking] Extra files for the 100-line commit hook**
- **Issue:** the unit tests, fixtures and the black-box poison helper do not fit the planned test files under 100 lines.
- **Fix:** added `src/git/config_tests.rs`, `src/git/worktree_tests.rs`, `src/git/fixture.rs`, `src/git/fixture/poison.rs`, `tests/support/poison.rs`; branch-name cases and the proptest live in `src/git/refname/tests.rs` (the repo pattern for `pr_url`) instead of `src/git/tests.rs`. `src/git/tests.rs` keeps the shared clone helpers and the hooks and unsafe-branch tests.
- **Committed in:** 629d694, 5194cbe

**2. [Rule 3 - Blocking] Two Error variants in src/error.rs**
- **Issue:** the validator and the empty-hooks check need errors that keep their cause; `error.rs` is not in the plan's file list.
- **Fix:** `Error::UnsafeBranch(String)` and `Error::HooksNotEmpty(PathBuf)`.
- **Committed in:** 629d694

**3. [Rule 3 - Blocking] GitCli::run and GitCli::text moved to cli.rs as pub(super)**
- **Issue:** src/git.rs was at 96 lines and needed nine module declarations and one import.
- **Fix:** the two helpers moved next to the trait impl; the trait, `GitCli`, `new` and the command builder stay in git.rs (99 lines).
- **Committed in:** 629d694

**4. [Rule 1 - Design] remove remote.origin.fetch without branching on exit status 5**
- **Fix:** the key is looked up in `git config --local --name-only --list` and `--unset-all` runs only when present, so no exit status needs parsing out of `Error::Command`.
- **Committed in:** 629d694

**5. [Rule 2 - Missing critical] End-to-end symlink test and FakeClaude::snapshot_tree**
- **Issue:** the prior-wave security note asks for the symlink mitigation to be shown on the path the runner uses, and no plan task covered the black-box side.
- **Fix:** `pr_symlink_reaches_the_agent_as_a_plain_file` in tests/git_isolation.rs plus an opt-in `snapshot_tree` flag in `tests/support/fake_claude.{rs,sh}` (opt-in because other harness tests run the fake with the repo root as cwd).
- **Committed in:** 5194cbe

**Total deviations:** 5 (3 blocking, 1 design, 1 missing-critical). **Impact:** none on behavior beyond the plan; Git trait signatures and the public paths named in the plan are unchanged.

## Issues Encountered

- A regex rename in the fixture first rewrote `--bare` to `--origin`; the first test run showed it and it was fixed before any commit.
- `.planning/config.json`, `.planning/milestone.lock` and `.planning/state.json` were already modified or untracked when the plan began and were left out of every commit, as instructed.

## User Setup Required

None.

## Known Stubs

None.

## Threat Flags

None beyond the plan's threat model. Mitigations present: T-01-23 (env pinned by the environment test, five config routes plus control), T-01-24 (hooks path at a checked-empty directory, `init --template=` from 01-04, control proves a global hooksPath fires), T-01-25 (validate_branch_name before fetch, black-box `main:refs/heads/evil`), T-01-26 (core.symlinks false, unit and black-box tests), T-01-27 (credential helper only in fetch_pr, unchanged), T-01-28 (autoDetach keys written; key names confirmed, runtime effect not tested).

## Next Phase Readiness

- Plan 01-07 can rely on `fetch_pr` returning `Error::UnsafeBranch` for unsafe base branches and on the unchanged Git trait.
- Plan 01-11 can reuse `tests/support/poison.rs` (Poison::write, control) if its isolation checks need a poisoned git config for the agent's git wrapper.
- Plan 01-12 owns the live fork PR check; plan 01-02 still waits on the owner and `tests/fixtures/live-prs.json` was not touched.

## Self-Check: PASSED

All 12 created files exist and commits 629d694 and 5194cbe are ancestors of HEAD (verified with `[ -f ]` and `git merge-base --is-ancestor` in the closing step).

---
*Phase: 01-single-review*
*Completed: 2026-10-20*
