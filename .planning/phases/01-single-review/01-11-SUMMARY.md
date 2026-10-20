---
phase: 01-single-review
plan: 11
subsystem: testing
tags: [rust, tokio, claude-cli, isolation, canaries, fake-endpoint, positive-controls]

requires:
  - phase: 01-single-review
    provides: "01-01 spike contract (fake endpoint, controls, trust keys, tool set, git calls), 01-04/01-05 ClaudeCli with PATH git wrapper, TestOverrides and with_overrides, 01-10 unchanged AgentRequest::new and ClaudeCli::run"
provides:
  - "src/tests/isolation: recording fake Anthropic endpoint, malicious-PR fixture, run harness for the isolated vector, the control vector and an open control"
  - "REV-06, REV-07 and REV-08 as #[ignore] tests under `mise run test:e2e`, each with a positive control that fails as vacuous when it observes nothing"
  - "isolation_flags_keep_authentication: one live model call under the production flags and the real allowlisted environment"
  - "Non-ignored CI tests for the endpoint parser, the fixture (canaries, trust keys, git filter control) and the inspection helpers"
affects: [01-12]

actuals:
  tokens: 9400
  tasks: 2
  commits: 2

plan_head_before: 24ba8b11c0695d58694c714ec08ed995c9218877
plan_head_after: 0bf134d129ec5494e8337d21e26bb608c3d28242
commits: 2

tech-stack:
  added: ["tokio dev feature net (pulls socket2 0.6.5, MIT OR Apache-2.0, published 2026-07-13)"]
  patterns:
    - "Every isolation negative is paired with a control run in the same fixture; the test asserts the control first and the message says 'control vacuous'"
    - "Absence is a byte-substring search over every raw request body, with a separate check that the run reached POST /v1/messages"
    - "One throwaway TempDir per test holds HOME, CLAUDE_CONFIG_DIR, worktree, context, scripts, markers and the git shim log; listeners bind 127.0.0.1:0"
    - "The settings.json env block points ANTHROPIC_BASE_URL at a second recording endpoint, so the control's model request lands there and is read from there"

key-files:
  created:
    - src/tests.rs
    - src/tests/isolation.rs
    - src/tests/isolation/contract.rs
    - src/tests/isolation/endpoint.rs
    - src/tests/isolation/endpoint_check.rs
    - src/tests/isolation/http.rs
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
    - src/tests/isolation/inspect.rs
    - src/tests/isolation/rev06.rs
    - src/tests/isolation/rev07.rs
    - src/tests/isolation/rev08.rs
    - src/tests/isolation/auth.rs
  modified:
    - src/main.rs
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "The two controls that contract.json records as false are handled by designing a different control that does fire, as 01-01 and the dispatch instruct, instead of a checkpoint: a CLAUDE.md at the add-dir root with CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1 stands in for rules/CLAUDE.md, and an open control that also drops --tools and --disable-slash-commands shows the agents/commands/skills canary"
  - "The expected isolated tool set is exactly Read, Grep, Glob, StructuredOutput, DeferredToolPlaceholder (owner decision A1); a CI test pins the constant to contract.json"
  - "The AGENTS.md control hides CLAUDE.md (AGENTS.md loads only when CLAUDE.md is absent) and the isolated run is repeated without CLAUDE.md so that negative is meaningful too"
  - "The isolated runs receive the same pre-granted trust keys, import approval and add-dir env var as the controls, so only the isolation flags differ"
  - "The secondary listener is a recording FakeAnthropic, not a connection counter"

patterns-established:
  - "observe(pr, secondary, Mode) runs one vector against a fresh primary endpoint and returns the requests of both endpoints"
  - "Each run gets its own prompt directory, because the git wrapper installer creates RUN/prompt/bin/git with create_new"

requirements-completed: [REV-01, REV-06, REV-07, REV-08]

coverage:
  - id: D1
    description: "REV-06: with the production flags a canary in the worktree CLAUDE.md, in AGENTS.md (CLAUDE.md absent), and in a CLAUDE.md at the add-dir root is not in any request body, the appended review rules are, and the tool set is the five accepted names; controls without --safe-mode, --restricted and --strict-mcp-config show the canaries"
    requirement: REV-06
    verification:
      - kind: e2e
        ref: "src/tests/isolation/rev06.rs#rev06_worktree_claude_md_canary_is_not_loaded (mise run test:e2e)"
        status: pass
    human_judgment: false
  - id: D2
    description: "REV-07: a canary behind an @ import of a file outside the worktree is in the control's bodies (approval pre-granted) and not in the isolated bodies"
    requirement: REV-07
    verification:
      - kind: e2e
        ref: "src/tests/isolation/rev07.rs#rev07_outside_import_canary_is_not_leaked (mise run test:e2e)"
        status: pass
    human_judgment: false
  - id: D3
    description: "REV-08: SessionStart hook, apiKeyHelper, plugin hook and MCP server markers and a settings env redirect fire in the control and not in the isolated run; the secondary listener gets no new connection; no mcp__ tool; no agents/commands/skills canary; every CLI git call carries GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1"
    requirement: REV-08
    verification:
      - kind: e2e
        ref: "src/tests/isolation/rev08.rs#rev08_hooks_plugins_and_mcp_do_not_run_or_load (mise run test:e2e)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Authentication keeps working under the production flags and the agent environment allowlist: one live call returns AgentOutcome::Succeeded"
    requirement: REV-01
    verification:
      - kind: e2e
        ref: "src/tests/isolation/auth.rs#isolation_flags_keep_authentication (mise run test:e2e)"
        status: pass
    human_judgment: false
  - id: D5
    description: "The fake endpoint records Content-Length, chunked and HEAD requests byte for byte and answers 400; the fixture's canaries are distinct ASCII tokens, its trust keys come from contract.json, and its global git filter fires with plain git and not with GIT_CONFIG_GLOBAL=/dev/null"
    requirement: REV-08
    verification:
      - kind: unit
        ref: "src/tests/isolation/endpoint_check.rs, fixture_check.rs, harness_check.rs (run by mise run test in CI)"
        status: pass
    human_judgment: false
  - id: D6
    description: "Tool-level confinement (the agent reading an outside file or following a symlink through a tool call) and rules/CLAUDE.md as an add-dir memory file have no direct control"
    verification: []
    human_judgment: true
    rationale: "A 400-only endpoint cannot exercise tool calls, and the spike recorded that rules/CLAUDE.md does not load even in the unisolated control; a live-model probe would be needed for direct evidence"

duration: 25min
completed: 2026-10-20
status: complete
---

# Phase 1 Plan 11: Claude isolation suite Summary

**REV-06, REV-07 and REV-08 are proven on CLI 2.1.304 with canaries and positive controls against a local recording Anthropic endpoint, and one live call shows authentication still works under the production flags**

## Performance

- **Duration:** about 25 min (start not recorded; counted from the 01-10 close-out commit at 22:50 local to this SUMMARY)
- **Tasks:** 2
- **Files:** 25 changed in the plan range (22 created, 3 modified), 1093 insertions and 1 deletion

## Accomplishments

- `FakeAnthropic` accepts on `127.0.0.1:0`, reads the head and the body by Content-Length or chunked encoding, records method, path and raw body, counts accepted connections, and answers 400 with `connection: close`. A non-ignored test sends a 300000-byte binary body, a chunked body with an extension and trailer, and a HEAD request, and compares the recorded bodies byte for byte.
- `MaliciousPr` builds, under one TempDir, a one-commit worktree with a CLAUDE.md that imports an outside file, an AGENTS.md, `.claude/settings.json` (SessionStart hook, apiKeyHelper, an env block redirecting `ANTHROPIC_BASE_URL`, an enabled plugin under `.claude/skills`), `.mcp.json`, agents, commands and skills carrying a canary, and a `.gitattributes` filter; a context directory with a root CLAUDE.md and `rules/CLAUDE.md`; a throwaway HOME and CLAUDE_CONFIG_DIR with `.claude.json` built from the contract's `trust_keys`; and a git shim that logs `GIT_CONFIG_GLOBAL`, `GIT_CONFIG_NOSYSTEM` and argv before exec'ing the real git. Canaries are `dmcanary` plus 16 hex digits from `RandomState`.
- The harness drives `ClaudeCli::with_overrides` in three modes: isolated (production vector), control (minus `--safe-mode --restricted --strict-mcp-config`) and open control (also minus `--tools Read,Grep,Glob --disable-slash-commands`). `assert_reached` makes every negative fail unless the run sent a `POST /v1/messages` body.
- All four ignored tests pass with `mise run test:e2e -- -E 'test(tests::isolation::)'` (4 run, 4 passed, 19 s).

### Evidence per REV item (observed on the owner's machine, CLI 2.1.304)

**REV-06 (worktree CLAUDE.md is not loaded)**
- Control: bodies contain the CLAUDE.md canary and the canary of the CLAUDE.md at the add-dir root. With CLAUDE.md hidden, the control contains the AGENTS.md canary.
- Isolated: none of the CLAUDE.md, AGENTS.md (run with and without CLAUDE.md), context-root and `rules/CLAUDE.md` canaries; tool names are exactly Glob, Grep, Read, StructuredOutput, DeferredToolPlaceholder; a body contains "You review one pull request.".

**REV-07 (outside @ import is not leaked)**
- Control (import approval and trust pre-granted): bodies contain the import canary.
- Isolated (same pre-grants): bodies contain neither the import canary nor the CLAUDE.md canary.

**REV-08 (hooks, apiKeyHelper, settings env, plugin, MCP, agents/commands/skills)**
- Control: markers `session_start_hook`, `api_key_helper`, `plugin_hook` and `mcp_server` exist, and the settings env redirect delivered the model request to the secondary endpoint (1 connection). The open control shows the agents/commands/skills canary.
- Isolated: no marker file (including the global-git-filter marker), zero new secondary connections, no `mcp__` tool, exact five-name tool set, no canary of any kind, and every logged CLI git call starts with `GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1` (the log must be non-empty because `claude_git_calls.runs_git` is true).

**Authentication**
- `isolation_flags_keep_authentication` returned `AgentOutcome::Succeeded` for a tiny typo-fix context with the real allowlisted environment (one model call; cost not captured).

**Non-vacuity**
- Mutation check, observed and reverted: giving the isolated run the control's removed flags makes rev06 fail at the canary assertion, rev07 at the import assertion and rev08 at the marker assertion (`left: ["session_start_hook", "api_key_helper", "plugin_hook", "mcp_server"]`, `right: []`).

## Task Commits

1. **Task 1: Fake Anthropic endpoint, malicious-PR fixture and run harness** - `7dee043` (test)
2. **Task 2: REV-06, REV-07, REV-08 with positive controls, and the live authentication check** - `0bf134d` (test)

**Plan metadata:** the commit that adds this SUMMARY (docs: complete plan)

## Files Created/Modified

- `src/tests/isolation/endpoint.rs`, `http.rs` - recording listener and its HTTP request reader and 400 reply
- `src/tests/isolation/fixture.rs` and `fixture/*.rs` - the malicious PR, its layout, scripts, worktree files, home and context
- `src/tests/isolation/harness.rs`, `inspect.rs` - run modes, `observe`, byte-substring search, tool name extraction, reached check
- `src/tests/isolation/rev06.rs`, `rev07.rs`, `rev08.rs`, `auth.rs` - the four `#[ignore]` tests
- `src/tests/isolation/*_check.rs` - non-ignored CI tests
- `Cargo.toml`, `Cargo.lock` - tokio dev feature `net`

## Decisions Made

- See `key-decisions` in the frontmatter.
- Crate note for the PR description: no direct crate was added. Enabling tokio's `net` dev feature adds socket2 0.6.5 (tokio's own dependency, MIT OR Apache-2.0, published 2026-07-13, more than 3 days old). `mise run vuln` still reports advisories, bans and sources ok.

## Deviations from Plan

**1. [Rule 3 - Blocking] tokio `net` was not a dev feature**
- **Issue:** the plan says 01-04 added it; the 01-04 SUMMARY records that it did not.
- **Fix:** `tokio = { ..., features = ["net", "test-util"] }` under dev-dependencies; Cargo.lock gains socket2 0.6.5.
- **Committed in:** 7dee043

**2. [Rule 1 - Bug] The secondary listener must record, not only count**
- **Issue:** the settings env block sends the control's model request to the secondary endpoint, so a connection counter that closes connections hid the control's bodies (no canary ever seen) and made the CLI retry for 189 s.
- **Fix:** the secondary is a `FakeAnthropic`; `connections()` counts accepts and `observe` returns the bodies of both endpoints. `ConnectionCounter` from the plan's interfaces does not exist.
- **Committed in:** 7dee043

**3. [Rule 3 - Blocking] Extra files for the 100-line hook, and small interface additions**
- `http.rs`, `contract.rs`, `inspect.rs` (holds `bodies_contain`, `tool_names`, `assert_reached` instead of harness.rs), `fixture/{context,files,home,layout,ops,scripts,worktree}.rs`, `fixture_check.rs`, `harness_check.rs`. `MaliciousPr` gained `bin`; the harness gained `Mode`, `observe`, `run_open_control` and `EXPECTED_TOOLS`; `FakeAnthropic` gained `connections()` and `take()`.
- **Committed in:** 7dee043, 0bf134d

**4. [Rule 3 - Blocking] `#[ignore = "reason"]`**
- **Issue:** clippy (warnings denied) rejects `#[ignore]` without a reason, so the plan's `grep -c '#\[ignore\]'` prints 0 for the four files. `grep -c ignore` prints 1 per file and the tests are skipped by `mise run test` (7 skipped in the suite).

**5. [Rule 1 - Design] Controls for the two items the spike recorded as false**
- `contract.json` has `rules_claude_md_in_add_dir` and `agents_commands_skills` false. Instead of a checkpoint, the dispatch and the 01-01 SUMMARY name the controls to build, and they fire: a CLAUDE.md at the context root (with `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1`, which is required: without it the root file did not load, observed) and an open control without `--tools` and `--disable-slash-commands`. The owner should confirm this reading of "design those controls accordingly".

**6. [Rule 1 - Design] AGENTS.md control**
- AGENTS.md loads only when CLAUDE.md is absent, so rev06 hides CLAUDE.md for one control run and one isolated run and restores it.

**7. [Rule 3 - Blocking] Separate prompt directory per run**
- The git wrapper installer uses `create_new` on `PROMPT_DIR/bin/git`; reusing a prompt file made the second run fail with "File exists". `prompt_for` copies the prompt into a new directory per run.

**8. [Rule 3 - Blocking] A temporary smoke test**
- `smoke.rs` kept Task 1's harness functions used (dead code is denied) and was used to debug the endpoint and controls; Task 2 deleted it.

**Total deviations:** 8 (1 bug, 4 blocking, 3 design or process notes: items 5, 6 and 8). **Impact:** none on the asserted behavior; the suite is stricter than the plan in three places (isolated runs get the controls' pre-grants, the open control, the AGENTS.md pair).

## Issues Encountered

- `.planning/config.json`, `.planning/milestone.lock` and `.planning/state.json` were already modified or untracked and were left out of every commit. `tests/fixtures/live-prs.json` and `deny.toml` were not touched.
- `mise run vuln` reports `licenses FAILED` for `borrow-or-share 0.2.4` (MIT-0), pre-existing and unrelated; advisories, bans and sources are ok. Left for the owner as in 01-10.
- Positive controls that could not fire in this harness: the global git clean filter cannot fire through the CLI because the PATH wrapper always sets `GIT_CONFIG_GLOBAL=/dev/null`, so its control is a direct plain-git run in `fixture_check.rs`; `rules/CLAUDE.md` as an add-dir memory file never loads, so its negative is covered by the root-file control only (D6 in coverage).

## Verification

Observed in this session:

- `mise run test:e2e -- -E 'test(tests::isolation::)'`: 4 run, 4 passed.
- `mise run --force test`: 164 run, 164 passed, 7 skipped (also run by the hk pre-commit hook on both commits). The non-ignored isolation tests (10) pass in it.
- `mise run --force lint`: rustfmt, clippy pedantic with warnings denied, cargo-machete, gitleaks, actionlint all exit 0.
- `mise run coverage`: 91.41% line coverage (gate 80%).
- `mise run deps:check`: all eligible releases.
- Acceptance: `grep -c 127.0.0.1 src/tests/isolation/endpoint.rs` = 1; every file under `src/tests/isolation` is at most 96 lines.
- Not run: `mise run docs:check` outside the hook; the hook ran it.

## User Setup Required

None.

## Known Stubs

None.

## Threat Flags

None beyond the plan's threat model. Mitigations present: T-01-47 (control asserted before every negative, 'control vacuous' messages, mutation check), T-01-48 (secondary endpoint must see no new connection), T-01-49 (hooks, scripts and markers live in the TempDir; throwaway HOME and CLAUDE_CONFIG_DIR), T-01-51 (live test uses the allowlisted environment and temp directories; one call), T-01-SC (no direct crate added).

## Next Phase Readiness

- Plan 01-12 can rely on `mise run test:e2e` covering isolation; the live-PR fixture list in plan 01-02 still waits on the owner.
- If the CLI is upgraded, rerun `mise run test:e2e`; the tool-set constant is pinned to `contract.json` and fails in CI if either changes alone.

## Self-Check: PASSED

All 22 created files exist and commits 7dee043 and 0bf134d are ancestors of HEAD (verified with `[ -f ]` and `git merge-base --is-ancestor` in the closing step).

---
*Phase: 01-single-review*
*Completed: 2026-10-20*
