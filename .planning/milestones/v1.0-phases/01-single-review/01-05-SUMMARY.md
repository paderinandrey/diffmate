---
phase: 01-single-review
plan: 05
subsystem: agent
tags: [rust, claude-cli, version-probe, envelope-classification, jsonschema, proptest, cli_error, agent_error, invalid_output]

requires:
  - phase: 01-single-review
    provides: "01-01 spike contract and golden envelopes, 01-03 black-box harness, 01-04 tracer slice (ClaudeCli, envelope::classify, PATH git wrapper)"
provides:
  - "Version and flag probe before every agent run (REV-11): minimum 2.1.304 as a parsed triple, exact production vector with empty stdin, fail closed as cli_error"
  - "Complete D-01 classification of the result envelope with stored subtype, errors, api_error_status, terminal_reason, cost and duration (REV-05, REV-04)"
  - "Tests that pin the CLI schema argument, the agent environment allowlist and the production vector to tests/fixtures/claude/contract.json"
  - "cfg(test) TestOverrides and ClaudeCli::with_overrides, args_for, env_vars for plans 01-11 and 01-12"
affects: [01-06, 01-07, 01-11, 01-12]

actuals:
  tokens: 16000
  tasks: 3
  commits: 3

plan_head_before: 690965dc2fe9aa59ff88d687602747402f43be4a
plan_head_after: 97cb078f9e47b8cdbc659e5306b2f90089daf6eb
commits: 3

tech-stack:
  added: []
  patterns:
    - "Probe texts live as constants in src/agent/probe.rs only; the CLI help output is never parsed"
    - "Envelope fields are read leniently from a serde_json object, so new CLI fields never break parsing"
    - "Test-only knobs are cfg(test) and reach the command through one adjust function with a no-op release twin"
    - "Large unit test sets are split into sibling *_tests.rs files to stay under the 100-line hook limit"

key-files:
  created:
    - src/agent/probe.rs
    - src/agent/probe/run.rs
    - src/agent/probe/tests.rs
    - src/agent/probe/live_tests.rs
    - src/agent/claude/contract.rs
    - src/agent/claude/tests.rs
    - src/agent/claude/contract_tests.rs
    - src/agent/claude/env_tests.rs
    - src/agent/envelope/fields.rs
    - src/agent/envelope/support.rs
    - src/agent/envelope/tests.rs
    - src/agent/envelope/synthetic_tests.rs
    - src/agent/envelope/golden_tests.rs
    - src/agent/envelope/property_tests.rs
    - src/agent/schema/tests.rs
    - src/agent/schema/generated_tests.rs
    - tests/agent_contract.rs
    - tests/agent_contract/classification.rs
  modified:
    - src/agent.rs
    - src/agent/claude.rs
    - src/agent/envelope.rs
    - src/agent/schema.rs
    - src/agent/spawn.rs

key-decisions:
  - "A non-zero exit code with an otherwise clean success envelope (is_error false, subtype success, valid output) is classified agent_error: SPEC does not name the case, and only exit 0 may produce succeeded"
  - "An authentication failure is recognized by exactly the signal recorded by the spike (is_error true, terminal_reason api_error, api_error_status null, result starting with Not logged in) and is cli_error; the same envelope with api_error_status 404 stays agent_error"
  - "Probe functions return Result<(), Box<FailureDetail>>: FailureDetail is about 184 bytes and clippy result_large_err would otherwise fire; the public shape of FailureDetail is unchanged"
  - "The probe passes only when stderr holds the recorded input-error text and the exit code is non-zero; any other answer, including exit 0, is flag probe inconclusive and fails closed"

patterns-established:
  - "Preflight order in spawn::invoke: inputs exist, wrapper installed once, run environment built once, version check, flag probe, then the agent with the same environment and vector"

requirements-completed: [REV-01, REV-04, REV-05, REV-11]

coverage:
  - id: D1
    description: "No run starts the agent on a CLI that is older than 2.1.304, whose version cannot be read, that lacks an isolation flag, or whose probe answer is unexpected; the run is stored failed with cli_error"
    requirement: REV-11
    verification:
      - kind: integration
        ref: "tests/agent_contract.rs#missing_flag_fails_before_agent_starts"
        status: pass
      - kind: integration
        ref: "tests/agent_contract.rs#old_cli_version_fails_before_agent_starts"
        status: pass
      - kind: integration
        ref: "tests/agent_contract.rs#unreadable_cli_version_fails_before_agent_starts"
        status: pass
      - kind: integration
        ref: "tests/agent_contract.rs#inconclusive_probe_fails_closed"
        status: pass
    human_judgment: false
  - id: D2
    description: "The probe is proven against the real CLI without a model call: production flags accepted, unknown flag named, a schema with $schema rejected"
    requirement: REV-11
    verification:
      - kind: other
        ref: "mise run test:e2e -- live_probe_ (3 passed)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Exit code 0 is necessary but not sufficient: an error envelope with exit 0 is failed (agent_error) and nothing prints no_findings; status/findings disagreement is invalid_output"
    requirement: REV-05
    verification:
      - kind: integration
        ref: "tests/agent_contract/classification.rs#error_envelope_with_exit_zero_is_failed"
        status: pass
      - kind: integration
        ref: "tests/agent_contract/classification.rs#status_disagreement_is_invalid_output"
        status: pass
      - kind: property
        ref: "src/agent/envelope/property_tests.rs#success_requires_all_four_checks"
        status: pass
    human_judgment: false
  - id: D4
    description: "Every recorded envelope in cases.json is classified with its recorded exit code per D-01, and cost and duration are stored exactly"
    requirement: REV-05
    verification:
      - kind: unit
        ref: "src/agent/envelope/golden_tests.rs#every_recorded_envelope_is_classified_as_d01_says"
        status: pass
      - kind: unit
        ref: "src/agent/envelope/golden_tests.rs#cost_and_duration_are_stored_exactly"
        status: pass
    human_judgment: false
  - id: D5
    description: "no_findings is accepted only with an empty findings list and findings only with a non-empty list"
    requirement: REV-04
    verification:
      - kind: unit
        ref: "src/agent/envelope/synthetic_tests.rs#status_and_findings_must_agree"
        status: pass
    human_judgment: false
  - id: D6
    description: "The CLI schema argument is the full schema minus $schema as recorded by the spike, the full schema is valid draft 2020-12, and the negative corpus is rejected"
    requirement: REV-01
    verification:
      - kind: unit
        ref: "src/agent/schema/tests.rs#cli_schema_argument_is_the_form_recorded_by_the_spike"
        status: pass
      - kind: unit
        ref: "src/agent/schema/tests.rs#full_schema_is_valid_draft_2020_12"
        status: pass
      - kind: unit
        ref: "src/agent/schema/tests.rs#the_negative_corpus_is_rejected"
        status: pass
    human_judgment: false
  - id: D7
    description: "The agent environment is the spike allowlist (names that are set) plus the two git variables; injected GitHub tokens and session variables never reach it; the production vector equals contract.json"
    requirement: REV-01
    verification:
      - kind: unit
        ref: "src/agent/claude/env_tests.rs#agent_env_is_the_allowlist"
        status: pass
      - kind: unit
        ref: "src/agent/claude/contract_tests.rs#allowlist_matches_spike and #production_vector_matches_spike"
        status: pass
    human_judgment: false
  - id: D8
    description: "Whether the clean fixture PR really gets no findings from the model is the owner's end-of-phase check in plan 01-12; diffmate only guarantees the status/findings agreement"
    requirement: REV-04
    verification: []
    human_judgment: true
    rationale: "Model judgment on a real clean PR cannot be asserted by a deterministic test"

duration: 14min
completed: 2026-10-20
status: complete
---

# Phase 1 Plan 05: Agent contract Summary

**Version and flag probe before every run (minimum Claude CLI 2.1.304, exact production vector, fail closed as cli_error), full D-01 envelope classification proven against the spike's golden envelopes, and the schema form, environment allowlist and production vector pinned to contract.json**

## Performance

- **Duration:** 14 min (2026-10-20T18:04Z to 18:19Z)
- **Tasks:** 3
- **Files:** 23 changed in the plan range (18 created, 5 modified), 1388 insertions and 137 deletions

## Accomplishments

- `spawn::invoke` now installs the git wrapper once, builds the run environment once, runs `claude --version` and the flag probe with that environment, and only then starts the agent. A version below 2.1.304 (compared as a parsed triple, so 2.1.40 is older), an unreadable version, an unknown option, a rejected schema, a missing prompt file or any unexpected probe answer fails the run as `cli_error` with no agent call; the black-box tests assert exactly the version and probe calls and an empty `run_calls()`.
- `envelope::classify` follows the D-01 order: stdout over the capture limit or not a JSON object is `invalid_output`; the recorded logged-out signal is `cli_error`; `error_max_structured_output_retries` is `invalid_output`; a non-zero exit, `is_error` true or a subtype other than `success` is `agent_error`; then a missing, null or non-conforming `structured_output` (a string is parsed once) and a status/findings disagreement are `invalid_output`. Subtype, is_error, errors, api_error_status, terminal_reason, exit code and a stderr excerpt are stored in the detail, and cost and duration are kept as reported on success and failure.
- A property test over generated envelopes (exit 0 to 2, is_error true/false/absent, four subtypes, four output shapes) proves `Succeeded` exactly when all four checks hold.
- The CLI schema argument (full schema without `$schema`), the 17-name allowlist and the production vector are compared with `contract.json` at run time, so a re-run spike that changes the contract fails these tests.
- The permanent S0 probe is three `#[ignore]` live tests that pass against CLI 2.1.304 without a model call.

## Task Commits

1. **Task 1: CLI version and flag probe before every run** - `a000689` (feat)
2. **Task 2: Classify agent failures per D-01 against the golden envelopes** - `55ace58` (feat)
3. **Task 3: Pin the CLI schema and the agent environment to the spike contract** - `97cb078` (test)

**Plan metadata:** the commit that adds this SUMMARY (docs: complete plan)

## Decisions Made

- See `key-decisions` in the frontmatter.
- **Non-zero exit with a success envelope is classified `agent_error`.** SPEC's four checks start with exit code 0, so such a run cannot be `succeeded`; SPEC does not say which failure kind it is, and the CLI itself reported the run as failed. The detail keeps the exit code, subtype and the (clean) envelope fields.
- The logged-out signal does not check `total_cost_usd` 0: the other four recorded fields identify it, and a cost check would make the mapping brittle.
- No stderr branch for a logged-out CLI without an envelope: the spike recorded that case as a parseable envelope (`auth_failure.stdout_is_envelope` true), so unparsable stdout is always `invalid_output`.

## Verification

Observed in this session:

- `mise run --force test`: 62 tests run, 62 passed, 3 skipped (the live probes), before the last commit; the hk pre-commit hook re-ran the suite on each commit and passed.
- `mise run --force test -- -E 'test(agent::probe::) | test(agent::claude::) | binary(agent_contract)'`, `-E 'test(agent::envelope::) | binary(agent_contract)'` and `-E 'test(agent::schema::) | test(agent::claude::)'`: all passed, none empty.
- `mise run test:e2e -- live_probe_`: 3 tests run, 3 passed, against the installed CLI 2.1.304.
- `mise run --force lint`: rustfmt, clippy pedantic with warnings denied, cargo-machete, gitleaks and actionlint all exit 0.
- `mise run coverage`: exit 0, 91.26% line coverage (gate 80%).
- Acceptance greps: `MIN_VERSION` appears in src/agent/probe.rs, `--help` appears 0 times there, `error_max_structured_output_retries` appears in src/agent/envelope.rs; no file touched by the plan exceeds 100 lines.
- Not run: `mise run vuln` and `mise run deps:check` (no crate was added), `mise run docs:check` outside the hook.

## Deviations from Plan

**1. [Rule 3 - Blocking] Extra files for the 100-line commit hook**
- **Issue:** probe, claude, envelope, schema and the test files exceeded 100 lines after rustfmt.
- **Fix:** added src/agent/probe/run.rs, src/agent/claude/contract.rs (flags, allowlist, user prompt, TestOverrides; re-exported from claude.rs), src/agent/claude/contract_tests.rs, env_tests.rs, src/agent/envelope/fields.rs, support.rs, synthetic_tests.rs, property_tests.rs, src/agent/schema/generated_tests.rs and tests/agent_contract/classification.rs (loaded with `#[path]` because a test crate root resolves modules next to itself). Public paths named in the plan interfaces hold: `crate::agent::probe::*`, `crate::agent::claude::{ClaudeCli, TestOverrides}`.
- **Committed in:** a000689, 55ace58, 97cb078

**2. [Rule 3 - Blocking] Third live test renamed**
- **Issue:** the plan names it `live_cli_rejects_dollar_schema_and_accepts_cli_schema`, but its verify filter `live_probe_` and the expectation of 3 tests would not match that name.
- **Fix:** renamed to `live_probe_rejects_dollar_schema_and_accepts_cli_schema`.
- **Committed in:** a000689

**3. [Rule 1 - Bug] Tracer ClaudeCli kept PATH outside the environment list**
- **Issue:** the tracer stored the allowlist without PATH plus a separate `path` field, so `env_vars()` could not represent the whole allowlist.
- **Fix:** `env_vars()` now returns the allowlist including PATH plus the two git variables; `run_env(bin)` prefixes the wrapper directory for each run. The agent's environment is unchanged (the tracer black-box test still passes).
- **Committed in:** a000689

**4. [Rule 3 - Blocking] Probe functions return Box<FailureDetail>**
- **Issue:** `Result<(), FailureDetail>` as in the interfaces block triggers clippy `result_large_err` (FailureDetail is about 184 bytes).
- **Fix:** `Result<(), Box<FailureDetail>>`; the type FailureDetail itself is untouched, so plans 01-06 and 01-07 are unaffected.
- **Committed in:** a000689

**5. [Rule 2 - Missing critical] Extra black-box test for an unreadable version**
- **Fix:** `unreadable_cli_version_fails_before_agent_starts` covers the version-parse failure path that the plan's three tests did not reach.
- **Committed in:** a000689

**Total deviations:** 5 (1 bug, 1 missing-critical, 3 blocking). **Impact:** none on behavior beyond the plan; interface names kept.

## Issues Encountered

- `ReviewResult` still carries `#[expect(dead_code)]` from 01-04; nothing in this plan reads `run_id`, so the attribute stays for whichever plan reads it first.
- 01-03 asked this plan to check that the fake's probe text equals the spike's text: `fake_probe_text_matches_spike` does, for both the probe stderr and the version line.

## User Setup Required

None.

## Known Stubs

None.

## Threat Flags

None beyond the plan's threat model. Mitigations present: T-01-18 (version triple and exact-vector probe, fail closed, agent not spawned), T-01-19 (four ordered checks, status agreement, D-01 kinds with stored detail), T-01-20 (full-schema validation plus deny_unknown_fields, permissive envelope), T-01-21 (allowlist test with injected GH_TOKEN, GITHUB_TOKEN, CLAUDECODE, GIT_DIR, AWS_SECRET_ACCESS_KEY, ANTHROPIC_BASE_URL), T-01-22 (TestOverrides is cfg(test)).

## Next Phase Readiness

- Plans 01-11 and 01-12 can use `ClaudeCli::with_overrides(TestOverrides { extra_args, removed_flags, extra_env })`, `args_for` and `env_vars` inside the crate's unit tests; `probe::{check_version, check_flags}` take the program, environment, vector and working directory.
- Plans 01-06 and 01-07 saw no change to AgentOutcome, FailureKind, FailureDetail, ReviewOutput, OutputFinding, AgentRequest or the Agent trait.
- Plan 01-02 still waits on the owner; `tests/fixtures/live-prs.json` was not touched.

## Self-Check: PASSED

All 18 created files exist and commits a000689, 55ace58 and 97cb078 are ancestors of HEAD (verified with `[ -f ]` and `git merge-base --is-ancestor`).

---
*Phase: 01-single-review*
*Completed: 2026-10-20*
