---
phase: 01-single-review
plan: 04
subsystem: cli
tags: [rust, tokio, rusqlite, jsonschema, clap, git-worktree, claude-cli, sqlite]

requires:
  - phase: 01-single-review
    provides: "01-01 spike contract (tests/fixtures/claude/contract.json) and 01-03 black-box harness (tests/support)"
provides:
  - "diffmate review URL end to end: CLI, SQLite job, gh, bare clone and worktree, context files, isolated claude, full-schema validation, one-transaction storage, cleanup, terminal output"
  - "SQLite migration 1: targets, jobs, runs, drafts, findings with SPEC state CHECKs"
  - "Agent environment: auth allowlist, GIT_CONFIG_GLOBAL and GIT_CONFIG_NOSYSTEM, PATH git wrapper (owner decision A1)"
  - "Traits Git, Gh, Agent with GitCli, GhCli, ClaudeCli adapters"
affects: [01-05, 01-06, 01-07, 01-08, 01-09, 01-10, 01-11, 01-12]

actuals:
  tokens: 19500
  tasks: 1
  commits: 2

plan_head_before: 84bada4866dc942b32f6004c2cdfd35778704ac5
plan_head_after: dce9bde3362dfacdc7b301ee1e4b959d7f68b7d0
commits: 2

tech-stack:
  added: [tokio 1.53.3, rusqlite 0.40.2 (bundled), jsonschema 0.59.0 (default-features = false), serde 1.0.229, serde_json 1.0.154, clap 4.6.7, thiserror 2.0.21, proptest 1.12.0 (dev)]
  patterns:
    - "Single binary crate; traits Git, Gh, Agent with native async-in-trait and generic runner"
    - "Every subprocess: env_clear plus explicit environment, argument vectors, kill_on_drop, bounded capture"
    - "rusqlite only inside spawn_blocking, writes with TransactionBehavior::Immediate"
    - "Run directory removed on every exit path once a run exists; cleanup problems become warnings"

key-files:
  created:
    - src/cli.rs
    - src/app.rs
    - src/error.rs
    - src/paths.rs
    - src/pr_url.rs
    - src/pr_url/tests.rs
    - src/procenv.rs
    - src/proc.rs
    - src/store.rs
    - src/store/schema.sql
    - src/store/jobs.rs
    - src/store/runs.rs
    - src/store/finish.rs
    - src/store/records.rs
    - src/gh.rs
    - src/git.rs
    - src/git/cli.rs
    - src/context.rs
    - src/agent.rs
    - src/agent/claude.rs
    - src/agent/spawn.rs
    - src/agent/gitwrap.rs
    - src/agent/gitwrap/tests.rs
    - src/agent/envelope.rs
    - src/agent/output.rs
    - src/agent/schema.rs
    - src/runner.rs
    - src/runner/steps.rs
    - src/runner/cleanup.rs
    - src/runner/outcome.rs
    - src/review.rs
    - src/render.rs
    - tests/review.rs
    - tests/tracer/mod.rs
    - tests/tracer/agent_call.rs
    - tests/tracer/stored.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/main.rs
    - tests/cli.rs
    - tests/harness.rs

key-decisions:
  - "ClaudeCli installs the PATH git wrapper itself (prompt/bin/git next to the prompt file, outside worktree and context) and execs the first git found on the inherited PATH, so 01-05 does not own it"
  - "Check 4 of the SPEC envelope order (status agrees with findings) is in classify now, because ReviewOutput.status must be read and the check is one line"
  - "previous_finding_id from the model is stored as NULL with a run warning until 01-07 normalizes it against previous.json, so the findings foreign key cannot abort the transaction"
  - "tokio gets the fs feature so file writes, existence checks and directory removal do not need hand-made spawn_blocking wrappers"

patterns-established:
  - "Tests assert the agent invocation against tests/fixtures/claude/contract.json (production_flags with CONTEXT_DIR, PROMPT_FILE, CLI_SCHEMA placeholders and auth_env_allowlist), so the constant and the contract cannot drift silently"
  - "Black-box helpers live in tests/tracer/ (not tests/support), new test files may reuse them with mod tracer"

requirements-completed: [REV-02]

coverage:
  - id: D1
    description: "diffmate review URL runs the full pipeline against a local upstream with fake gh and claude, prints the anchored finding, stores job, run, draft and finding RUN.1, and leaves no worktree, metadata or run directory"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/review.rs#tracer_review_prints_anchored_finding"
        status: pass
    human_judgment: false
  - id: D2
    description: "Agent invocation is the contract production vector, cwd is the run worktree, stdin names only the context directory, environment is the allowlist plus the git isolation variables with no GH_TOKEN and a git wrapper first on PATH"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/tracer/agent_call.rs via tests/review.rs#tracer_review_prints_anchored_finding"
        status: pass
      - kind: unit
        ref: "src/agent/gitwrap/tests.rs#wrapper_forces_git_isolation_variables_and_execs_the_real_git"
        status: pass
    human_judgment: false
  - id: D3
    description: "A failure before the revision is known fails the job with a reason naming gh and creates no run; an agent error envelope stores a failed run with the envelope fields and no draft"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/review.rs#review_fails_before_run_when_gh_fails"
        status: pass
      - kind: integration
        ref: "tests/review.rs#agent_error_envelope_stores_a_failed_run"
        status: pass
    human_judgment: false
  - id: D4
    description: "Usage errors exit 2 (no arguments, non-github.com URL before any database is created) and --version prints diffmate 0.1.0"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/cli.rs#no_arguments_is_a_usage_error"
        status: pass
      - kind: integration
        ref: "tests/cli.rs#review_rejects_a_non_github_url_before_touching_the_database"
        status: pass
      - kind: integration
        ref: "tests/cli.rs#version_prints_name_and_version"
        status: pass
    human_judgment: false
  - id: D5
    description: "Strict PR URL grammar: lower-cased owner and repo, no path traversal or leading dash, proptested"
    requirement: REV-02
    verification:
      - kind: unit
        ref: "src/pr_url/tests.rs#valid_urls_parse_to_lower_cased_parts and #arbitrary_input_never_yields_an_unsafe_name"
        status: pass
    human_judgment: false

duration: 23min
completed: 2026-10-20
status: complete
---

# Phase 1 Plan 04: Tracer slice Summary

**diffmate review URL works end to end: CLI, SQLite job, gh, bare clone and worktree, isolated claude with the PATH git wrapper, full-schema validation, one-transaction run, draft and findings, cleanup on every exit, proven by one black-box tracer test**

## Performance

- **Duration:** 23 min (2026-10-20T17:39:54Z to 18:03Z)
- **Tasks:** 1 (tracer)
- **Files:** 41 changed in the plan range (36 created, 5 modified), 3648 insertions including Cargo.lock

## Accomplishments

- The full pipeline runs on the happy path against a local upstream whose PR head exists only under `refs/pull/1/head`: the job is enqueued, `gh pr view` resolves the PR, the bare clone is created with `core.hooksPath` at an empty directory, the PR head and base are fetched into `refs/diffmate/*`, the merge-base is computed, the run row is inserted only then, the worktree and context files are written, claude runs, the output is validated against the full schema, and run, draft and finding `1.1` commit in one transaction.
- Cleanup removes the worktree, runs `git worktree prune` and deletes the run directory on success and on failure; the tracer test asserts an empty runs directory and a single `worktree` entry in `git worktree list --porcelain`.
- The agent invocation equals `production_flags` from `contract.json` in order, with the prompt on stdin (only the absolute context directory), cwd at the run worktree, and an environment of the 17-name allowlist (PATH rebuilt) plus `GIT_CONFIG_GLOBAL=/dev/null` and `GIT_CONFIG_NOSYSTEM=1`. The test injects `GH_TOKEN` into diffmate's environment and asserts it never reaches the agent.
- Owner decision A1 is implemented here: a `git` wrapper that exports both git variables and execs the first real `git` on the inherited PATH sits first on the agent's PATH (`RUN/prompt/bin/git`, outside the worktree and the context directory, removed with the run directory). A unit test runs it against a fake real git whose environment carries conflicting values.
- Failure handling on the path: gh failure fails the job and creates no run; an error envelope (golden `error-max-turns.stdout`) stores `failure_kind agent_error` with `subtype`, `errors`/`terminal_reason` in the JSON detail and no draft.

## Task Commits

1. **Tracer: `diffmate review URL` end to end** - `59cf120` (feat)
2. **Lock tokio at 1.53.3** - `dce9bde` (chore): `cargo add` had resolved tokio 1.54.0 (published 2026-10-19, under the 3-day policy); `cargo update -p tokio --precise 1.53.3`, `mise run deps:check` passes.

**Plan metadata:** the commit that adds this SUMMARY (docs: complete plan)

## Decisions Made

- See `key-decisions` in the frontmatter. The agent environment and wrapper are fully owned by this plan; 01-05 adds the version and flag probe and D-01 edge classification on top and does not need to build the wrapper.
- `PATH` for claude is `RUN/prompt/bin` followed by the inherited PATH. If an inherited PATH has no `git`, `ClaudeCli::new` fails with `git was not found in PATH`.
- `Error::Command` carries the program and its (truncated) arguments, so a failed `gh pr view` names `gh` in the job failure reason.

## Verification

Observed in this session:

- `mise run --force test` (19 tests: harness 6, cli 3, review 3, bin unit tests 7): all passed.
- `mise run --force test -- -E 'binary(review) | binary(cli) | test(pr_url::)'`: 10 passed, 9 skipped.
- `mise run --force lint`: rustfmt, clippy pedantic with warnings denied, cargo-machete, gitleaks, actionlint all exit 0.
- `mise run deps:check`: "Rust 1.99.1, mise tools and direct crates are on their latest eligible releases".
- `mise run coverage`: exit 0, **91.63% line coverage** (96 of 1147 lines missed; gate is 80%).
- Acceptance greps: `grep -c 'CREATE TABLE' src/store/schema.sql` = 5; `--disable-slash-commands` and `--permission-prompts` each appear once in `src/agent/claude.rs`; no file above 100 lines among `src/**`, `tests/review.rs`, `tests/cli.rs`, `tests/tracer/*`; the unwrap/expect/panic grep outside tests prints nothing; Cargo.toml has no rustix and no tokio signal feature.
- The hk pre-commit hook (lint, large-files, secrets, docs, test, actions) passed on both commits without `--no-verify`.
- Not run: `mise run vuln` (the plan lists it for the phase gate, not for this plan), `mise run test:e2e`, `mise run docs:check` outside the hook.

Crate versions added (all newest releases at least 3 days old on 2026-10-20, locked): tokio 1.53.3 (rt, macros, process, time, io-util, sync, fs; dev: test-util), rusqlite 0.40.2 (bundled), jsonschema 0.59.0 (default-features = false), serde 1.0.229 (derive), serde_json 1.0.154, clap 4.6.7 (derive), thiserror 2.0.21, proptest 1.12.0 (dev). Non-SPEC crates and their purpose for the PR description: serde and serde_json (envelope, agent output and gh JSON), clap (subcommands, usage exit code 2, `--version`), thiserror (errors that keep their cause), proptest (URL parser, required by CLAUDE.md); all were audited OK in the research Package Legitimacy Audit.

## Deviations from Plan

**1. [Rule 2 - Missing critical] Envelope check 4 (status agrees with findings) in the tracer**
- **Found during:** envelope classification
- **Issue:** SPEC requires it, and `ReviewOutput.status` would otherwise be an unread field (dead_code is denied).
- **Fix:** `classify` returns `invalid_output` when `status` disagrees with `findings`. 01-05 can test it against fixtures.
- **Files modified:** src/agent/envelope.rs

**2. [Rule 2 - Missing critical] previous_finding_id stored as NULL with a warning**
- **Found during:** storage design
- **Issue:** `findings.previous_finding_id` is a foreign key; a model-supplied id with no earlier run would abort the whole transaction. Normalization against `previous.json` is 01-07.
- **Fix:** the runner stores NULL and adds a run warning naming the finding; 01-07 replaces this with real normalization.
- **Files modified:** src/runner/outcome.rs

**3. [Rule 3 - Blocking] Extra files for the 100-line commit hook**
- **Issue:** several modules exceeded 100 lines after rustfmt.
- **Fix:** added src/store/records.rs, src/store/finish.rs, src/git/cli.rs, src/agent/spawn.rs, src/agent/output.rs, src/runner/steps.rs, src/runner/cleanup.rs, src/runner/outcome.rs and tests/tracer/*. Public paths named in the plan interfaces are kept except where noted in item 5.

**4. [Rule 3 - Blocking] tokio `fs` feature; no `net` dev feature**
- The plan lists the feature set without `fs`; file writes, `try_exists` and `remove_dir_all` in async code use it instead of hand-made `spawn_blocking`. The dev `net` feature is not needed until the isolation endpoint (01-11) and was left out.

**5. [Rule 3 - Blocking] Interface items not created because nothing uses them yet**
- `Paths::lock`, `Paths::runs_dir`, `Paths::cache_dir` (dead code is denied by clippy). 01-08 adds `lock`. `OutputFinding` is not re-exported from `crate::agent` (it lives in `agent::output`); `ReviewResult` carries `#[expect(dead_code)]` because `run_id` is not read yet. Whichever plan first reads it must delete that attribute (the lint fails with "unfulfilled expectation" otherwise).

**6. [Rule 1 - Bug] tests/harness.rs asserted the old scaffold behavior**
- `sandbox_uses_spec_paths` called the binary with no arguments and expected success; with clap that is a usage error. It now runs `--version`.

**7. [Rule 3 - Blocking] tokio resolved to 1.54.0**
- Fixed in `dce9bde`, see Task Commits.

**Total deviations:** 7 (2 missing-critical, 4 blocking, 1 bug). **Impact:** none on behavior beyond the SPEC; interfaces keep their names.

## Issues Encountered

- A plan file list of 27 files cannot stay under the 100-lines-per-file hook without helper modules; see deviation 3.
- Tracer limitations left to later plans, as in the plan objective: flag probe and version check (01-05), git hardening and fork test (01-06), anchor normalization and the full D-04 layout (01-07, the tracer prints unanchored findings inline as `PATH [sev] TITLE`), lock, recovery and `status` (01-08), discussion/ci/previous/rules context (01-09), signals (01-10).
- `git worktree add` leaves `RUN/worktree` inside the run directory; `cleanup::run` removes via git first, then deletes the directory.

## User Setup Required

None.

## Known Stubs

None. Print layout for unanchored findings is a deliberate tracer simplification owned by 01-07.

## Threat Flags

None beyond the plan's threat model. Mitigations present: T-01-10 (strict URL grammar, proptest), T-01-11 (env_clear plus lists, `GH_TOKEN` absence asserted), T-01-12 (argument vectors, PR text only in files), T-01-13 (`sanitize` at print time), T-01-14 (0700 directories, 0600 database and context files), T-01-15 (parameterized SQL, CHECK constraints), T-01-16 (bounded capture with draining), T-01-17 (canonical URL check).

## Next Phase Readiness

- Ready for 01-05 (CLI version and flag probe, D-01 classification against the golden envelopes in `tests/fixtures/claude`): `envelope::classify` and `production_args` are the extension points; `AUTH_ENV` mirrors `contract.json` and the tracer test checks the vector against the contract.
- Plan 01-11 can build `ClaudeCli::new` from an `InheritedEnv` with a PATH that holds its own logging git shim: the wrapper execs the first `git` on that PATH and the shim then sees `GIT_CONFIG_GLOBAL=/dev/null`.
- Plan 01-02 still waits on the owner; `tests/fixtures/live-prs.json` was not touched.

## Self-Check: PASSED

Created files and commits verified below (see the commands in the closing step).

---
*Phase: 01-single-review*
*Completed: 2026-10-20*
