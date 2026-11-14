---
phase: 01-single-review
plan: 01
subsystem: testing
tags: [claude-cli, spike, json-schema, isolation, fake-endpoint, git-isolation]

requires:
  - phase: 01-single-review
    provides: "01-RESEARCH.md Spike Design (REV-01) and model-free probes on CLI 2.1.304"
provides:
  - "tests/fixtures/claude/contract.json: flag contract, minimal auth environment, envelope shape, schema form, fake-endpoint feasibility, tool set, positive controls, CLI git calls"
  - "Six redacted golden envelopes plus cases.json (success with and without findings, error_max_turns, unknown model, logged out, budget limit)"
  - "Owner decision A1: PATH git wrapper in the agent environment (SPEC Agent contract and D-02 amended)"
affects: [01-04, 01-05, 01-06, 01-11]

actuals:
  tokens: 5000
  tasks: 3
  commits: 4

plan_head_before: 6f351c81072985c64be212758fae00779c277b7d
plan_head_after: f1f4106b0ad3c2f02eea47ae84bf99e2531fae1b
commits: 4

tech-stack:
  added: []
  patterns:
    - "Every spike CLI call runs under env -i with an explicit variable list; contract.json stores names only"
    - "Positive controls for isolation checks: trust and import approval pre-written into the throwaway .claude.json"

key-files:
  created:
    - tests/fixtures/claude/contract.json
    - tests/fixtures/claude/cases.json
    - tests/fixtures/claude/success-findings.stdout
    - tests/fixtures/claude/success-no-findings.stdout
    - tests/fixtures/claude/error-max-turns.stdout
    - tests/fixtures/claude/error-unknown-model.stdout
    - tests/fixtures/claude/error-not-logged-in.stdout
    - tests/fixtures/claude/error-max-budget.stdout
  modified:
    - SPEC.md
    - .planning/phases/01-single-review/01-CONTEXT.md

key-decisions:
  - "schema_form is full: the stripped schemas/review.schema.json works end to end, so no schemas/review.cli.schema.json exists"
  - "Agent environment authenticates with PATH, HOME, USER, LOGNAME under --safe-mode --restricted"
  - "DeferredToolPlaceholder is accepted as an inert CLI-internal tool; the CLI 2.1.304 tool set is exactly Glob, Grep, Read, StructuredOutput, DeferredToolPlaceholder"
  - "A1: diffmate puts a git wrapper first on the agent PATH that forces GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1 and execs the real git"

patterns-established:
  - "Later plans read facts from contract.json instead of assuming CLI behavior"

requirements-completed: [REV-01]

coverage:
  - id: D1
    description: "contract.json records the flag contract, auth environment, envelope, schema form, fake-endpoint feasibility, tool set, controls and CLI git calls"
    requirement: REV-01
    verification:
      - kind: other
        ref: "jq -e on tests/fixtures/claude/contract.json (Task 1 to 3 verify commands)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Redacted golden envelopes for every SPEC-relevant outcome with exit codes in cases.json"
    requirement: REV-01
    verification:
      - kind: other
        ref: "Task 2 verify command: jq key checks, redaction grep, mise run lint:secrets"
        status: pass
    human_judgment: false
  - id: D3
    description: "SPEC Agent contract and D-02 state the PATH git wrapper"
    requirement: REV-01
    verification: []
    human_judgment: true
    rationale: "Wording of a specification change approved by the owner as decision A1; no test asserts prose"

duration: 115min
completed: 2026-10-20
status: complete
---

# Phase 1 Plan 01: Claude CLI spike Summary

**Claude CLI 2.1.304 spike: the full output schema works through --json-schema, the isolation flags hold on a local fake Anthropic endpoint, and the CLI's own git calls need a PATH git wrapper (owner decision A1)**

## Performance

- **Duration:** about 115 min elapsed from the first task commit (15:40Z) to the SUMMARY (17:36Z), including one owner checkpoint
- **Tasks:** 3
- **Files modified:** 10 (8 created, 2 modified)

## Spike outcomes

Installed CLI: 2.1.304 (`2.1.304 (Claude Code)`). Minimum CLI version proposal: 2.1.304, as in CONTEXT.md.

### Task 1: flag contract and authentication (S0, S3)

- All nine isolation flags (SPEC flags plus `--permission-prompts none` and `--disable-slash-commands`) are accepted: `probe.flags_accepted` is true. With empty stdin the production vector exits 1 with `Error: Input must be provided either through stdin or as a prompt argument when using --print`. An unknown option prints `error: unknown option '--bogus-flag-xyz'` and exits 1. Keeping `$schema` in the schema is rejected: `--json-schema is not a valid JSON Schema: no schema with key or ref "https://json-schema.org/draft/2020-12/schema"`. A missing prompt file prints `Error: Append system prompt file not found: <path>`.
- Minimal authenticating environment under `--safe-mode --restricted` (`auth_env_required`): `PATH`, `HOME`, `USER`, `LOGNAME`. `auth_env_allowlist` adds `TMPDIR`, locale and the optional auth, proxy and CA names, passed only when set. `auth_env_not_needed` lists the Claude Code session variables checked under `env -i`. Only names are stored.

### Task 2: envelope matrix and schema form (S1, S2)

- `schema_form` is `full`, `schema_file` is null, and `schemas/review.cli.schema.json` was not created. The real `prompts/review.md` with the stripped full schema ended with subtype `success` and `structured_output` for both the planted-bug PR and the clean PR.
- Envelope: a single JSON object on stdout; `structured_output` is a parsed object, not a string. Success fields and error fields are listed in `envelope.fields_success` and `envelope.fields_error` (errors carry `errors` instead of `result` and `structured_output`).
- `--max-turns 1`: exit code 1, subtype `error_max_turns`, `is_error` true, `terminal_reason` `max_turns`.
- Unknown model and logged-out CLI: exit 1, but the envelope has subtype `success` with `is_error` true and `terminal_reason` `api_error` (`api_error_status` 404 for the model, null for the logged-out case with result text starting `Not logged in`, cost 0). Plan 01-05 must key on `is_error` and `terminal_reason`, not on `subtype`, to map these to `cli_error` or `agent_error` per D-01.
- Budget limit: `--max-budget-usd` is accepted; the envelope has subtype `error_max_budget_usd`, `terminal_reason` `budget_exhausted`, exit 1.
- No exit-0 envelope with `is_error` true was observed, so REV-05 step 1 (exit code 0) is not contradicted by any captured case.
- Total live spend across all captures about $0.29 (sum of `total_cost_usd` in the committed envelopes: $0.2876).

### Task 3: fake endpoint, positive controls, CLI git calls (S4, S5)

- `fake_endpoint.works` is true under `--safe-mode --restricted`. A run makes 2 requests, `HEAD /api/hello` and `POST /v1/messages?beta=true`, ends on the 400 reply, does not retry, sends a plain JSON body, and the system prompt contains the appended `# Review rules`. Isolation checks in plan 01-11 can therefore be built on a local listener at no cost.
- `structured_output_tool` is `StructuredOutput`.
- Isolated first look: no canary appeared in the request, none of markers A to D was created, and the second (exfiltration) listener received 0 connections. `tool_names` is Glob, Grep, Read, StructuredOutput and DeferredToolPlaceholder. DeferredToolPlaceholder is an inert CLI-internal tool (empty schema, `defer_loading` true, described as never to be called) and is also present in the unisolated control. The owner accepted it (decision A1), so the expected tool set for plan 01-11 on CLI 2.1.304 is exactly those five names.
- Positive controls (run without `--safe-mode`, `--restricted`, `--strict-mcp-config`, with `projects[path].hasTrustDialogAccepted`, `hasClaudeMdExternalIncludesApproved` and `hasClaudeMdExternalIncludesWarningShown` pre-written): fired for `claude_md`, `import`, `agents_md` (only when CLAUDE.md is absent), `session_start_hook`, `api_key_helper`, `plugin_hook` (plugin enabled as `evilplug@skills-dir`), `mcp_server` and `settings_env_base_url`.
- Two controls did not fire, both recorded false (see Issues Encountered): `rules_claude_md_in_add_dir` and `agents_commands_skills`.
- CLI git calls: `claude_git_calls.runs_git` is true. Of 6 observed calls, 4 (ls-files, status, log, config user.name) had no `GIT_CONFIG_GLOBAL`, so `every_call_had_isolation_env` is false; `GIT_CONFIG_NOSYSTEM=1` was always present. A global clean filter selected by the fixture `.gitattributes` ran (marker E). Mitigation 1, a git wrapper first on the agent PATH exporting `GIT_CONFIG_GLOBAL=/dev/null` and `GIT_CONFIG_NOSYSTEM=1`: all 6 calls carried `/dev/null`, marker E absent. Mitigation 2, `CLAUDE_CODE_DISABLE_GIT_INSTRUCTIONS=1`: 3 calls, marker E absent, but `ls-files` still lacked `GIT_CONFIG_GLOBAL`. `chosen` is `path_git_wrapper`.

### Owner decision A1 and where it is implemented

- SPEC.md "Agent contract" (Environment bullet) now states the PATH git wrapper and why. D-02 in 01-CONTEXT.md is amended, dated 2026-10-20. Commit f1f4106.
- The wrapper is not implemented here. The implementation belongs to the plan that builds the agent environment (01-04/01-05). Plan 01-11 must assert that all CLI git calls carry `GIT_CONFIG_GLOBAL=/dev/null`.

## Accomplishments

- Established the flag contract, minimal auth environment and envelope shape on the installed CLI, with committed redacted golden envelopes
- Proved the full schema works through `--json-schema` once `$schema` is stripped, so no CLI-specific schema file is needed
- Proved a local fake endpoint receives the CLI's requests under the isolation flags and that the isolated run exposes no canary, hook, plugin, MCP server or outbound connection
- Found that the CLI's own git calls bypass `GIT_CONFIG_GLOBAL` and fixed the design with the PATH git wrapper

## Task Commits

1. **Task 1: Probe flag contract and minimal authenticating env** - `916dd91` (test)
2. **Task 2: Capture envelope matrix, decide schema form** - `c2eee2c` (test)
3. **Task 3: Fake endpoint, controls and CLI git calls** - `3ffeb82` (test)
4. **Owner decision A1: SPEC and D-02 amendment** - `f1f4106` (docs)

**Plan metadata:** the commit that adds this SUMMARY (docs: complete plan)

Plan 01-03 ran while this plan waited on the owner, so `git rev-list` over the plan range counts 8 commits; 4 are this plan's (`git log --grep='(01-01)'`) and 4 are plan 01-03's.

## Files Created/Modified

- `tests/fixtures/claude/contract.json` - machine-readable spike outcomes that later plans branch on (82 lines)
- `tests/fixtures/claude/cases.json` - exit code, stderr first line, extra argv and observation per captured envelope
- `tests/fixtures/claude/*.stdout` (six) - raw redacted result envelopes
- `SPEC.md` - Environment bullet of "Agent contract" states the PATH git wrapper
- `.planning/phases/01-single-review/01-CONTEXT.md` - D-02 amended with decision A1 and the accepted tool set

## Decisions Made

- Full schema form; no `schemas/review.cli.schema.json`.
- Accept DeferredToolPlaceholder as an inert CLI-internal tool (owner, A1).
- PATH git wrapper over `CLAUDE_CODE_DISABLE_GIT_INSTRUCTIONS=1`: the env flag still leaves `ls-files` without `GIT_CONFIG_GLOBAL`, the wrapper covers every call.

## Deviations from Plan

### Owner-approved deviations

**1. [Owner decision] Isolated tool set has five names, not four**
- **Found during:** Task 3
- **Issue:** The plan's acceptance criterion and stop condition expect exactly Glob, Grep, Read and the structured-output tool. The CLI also lists DeferredToolPlaceholder.
- **Fix:** Escalated to the owner, who approved A1 and accepted it. Recorded in `isolated_first_look.tool_names` and `isolated_first_look.accepted`.
- **Files modified:** tests/fixtures/claude/contract.json
- **Committed in:** 3ffeb82

**2. [Owner decision] SPEC.md and D-02 changed**
- **Found during:** Task 3
- **Issue:** The CLI does not pass `GIT_CONFIG_GLOBAL` to every git call it runs, so the SPEC environment alone does not isolate the owner's global git config.
- **Fix:** The owner approved the SPEC change; SPEC.md "Agent contract" and D-02 amended in a separate commit.
- **Files modified:** SPEC.md, .planning/phases/01-single-review/01-CONTEXT.md
- **Committed in:** f1f4106

**3. [Plan criterion] controls key count**
- Extra notes on control results live in a separate top-level `controls_notes` key so `.controls | keys | length` stays 11 as the acceptance criterion requires.

---

**Total deviations:** 2 owner-approved, 1 minor layout choice
**Impact on plan:** None on scope; both owner decisions strengthen isolation.

## Issues Encountered

- Two positive controls did not fire, and plan 01-11 must know:
  - `rules_claude_md_in_add_dir` is false, also with `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1`; only a CLAUDE.md at the add-dir root loads. The canary for `CONTEXT_DIR/rules/CLAUDE.md` cannot be proven by a control that loads it, so 01-11 needs a different positive control (a CLAUDE.md at the context directory root) for that check.
  - `agents_commands_skills` is false for the plan's control vector; it fires only when both `--tools` and `--disable-slash-commands` are dropped. 01-11 must drop both flags in that control.
- No non-API traffic was observed: the only requests were `HEAD /api/hello` and `POST /v1/messages?beta=true` to the configured base URL.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: git-config-exec | SPEC.md | The CLI runs git itself and can apply the owner's global git config (clean filters selected by a PR .gitattributes); mitigated by the PATH git wrapper to be implemented in 01-04/01-05 |

## Next Phase Readiness

- Plans 01-04 and 01-05 read `auth_env_allowlist`, `production_flags`, `schema_form` and the envelope facts; 01-05 also builds the PATH git wrapper in the agent environment.
- Plan 01-11 branches on `fake_endpoint`, `isolated_first_look`, `controls` and `claude_git_calls`, asserts the five-name tool set and that every CLI git call carries `GIT_CONFIG_GLOBAL=/dev/null`.
- Plan 01-02 still waits on the owner; `tests/fixtures/live-prs.json` was not touched.

## Self-Check: PASSED

All created files exist and commits 916dd91, c2eee2c, 3ffeb82 and f1f4106 are on the branch.

---
*Phase: 01-single-review*
*Completed: 2026-10-20*
