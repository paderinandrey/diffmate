# Phase 1: Single review - Context

**Gathered:** 2026-10-20
**Status:** Ready for planning

<domain>
## Phase Boundary

`diffmate review <url>` and `diffmate status` for one PR at a time: an isolated agent run in a worktree, an honest line-anchored draft printed to the terminal, and a failure never shown as "no findings". SPEC.md slice 1 and ROADMAP.md Phase 1 fix the scope. No discussion phase ran; these decisions come from the owner's answers to the open questions in 01-RESEARCH.md.

</domain>

<decisions>
## Implementation Decisions

### Failure kinds
- **D-01:** Failure kinds follow SPEC.md "Run" → "Failure kinds" (added 2026-10-20): `cli_error` covers run preparation (`gh`, `git`, filesystem), the CLI binary, version, flags, and authentication; `agent_error` covers a parseable error envelope; `invalid_output` covers unparsable stdout, a missing or schema-violating `structured_output`, a `status`/`findings` disagreement, and exhausted structured-output retries. The envelope's `subtype`, `errors`, `api_error_status`, and `terminal_reason` are stored with the failure.

### Agent environment
- **D-02:** The agent invocation adds `--permission-prompts none` and `--disable-slash-commands` to the SPEC isolation flags, and the agent environment sets `GIT_CONFIG_GLOBAL=/dev/null` and `GIT_CONFIG_NOSYSTEM=1` (SPEC.md "Agent contract", updated 2026-10-20). The spike confirms both flags are accepted on the target CLI version; the CLI flag check (REV-11) covers them. Amended 2026-10-20 (owner decision A1, from the 01-01 spike): the CLI runs git itself and passes `GIT_CONFIG_GLOBAL` to only some of those calls, so diffmate also puts a `git` wrapper first on the agent's `PATH` that forces `GIT_CONFIG_GLOBAL=/dev/null` and `GIT_CONFIG_NOSYSTEM=1` and then execs the real `git` (SPEC.md "Agent contract" updated). The isolated tool set for CLI 2.1.304 is Glob, Grep, Read, StructuredOutput and the inert CLI-internal DeferredToolPlaceholder, accepted by the owner.

### Signals
- **D-03:** Phase 1 has a minimal SIGINT/SIGTERM handler for a `review` that runs a job: it terminates the agent's process group, marks the run `failed`, and removes the worktree. The agent PID/PGID is stored on the run row (nullable) so later recovery can kill a leftover group. Stop-cause bookkeeping (`shutdown`/`user_cancel`/`crash`) and retry semantics stay in Phase 2 (REL-08, REL-09).

### Command output
- **D-04:** `review` prints plain text: one block per finding (`path:line [severity] title`, then the body), unanchored findings in a separate list, then the limitation and future improvements; warnings go to stderr. Exit code 0 for `succeeded`, 1 for `failed`, 2 for usage errors. `status` prints one row per target with tracking (`untracked` in Phase 1) and the latest run state.

### Claude's Discretion
Adopt the 01-RESEARCH.md recommendations for the remaining open questions without further owner input:
- the run row is created as `running` once the revision is resolved; earlier failures fail the job with a reason and no run;
- minimum Claude CLI version 2.1.304, compared as a parsed `major.minor.patch` triple;
- `prompts/review.md` and `schemas/review.schema.json` are embedded with `include_str!`, and the prompt is materialized per run outside the agent-visible directories;
- fixed per-file size caps for the context files with a first-line `TRUNCATED:` marker, cutting `diff.patch` on a hunk boundary;
- anchor semantics for LEFT and renamed paths as in Pitfall 13;
- `Paths` built from explicit roots, live checks as `#[ignore]` tests that inject temp roots, no new environment variables;
- only `github.com` hosts in Phase 1;
- a static user prompt that names the absolute context directory and nothing from the PR;
- no deduplication of two `review` commands for one URL in Phase 1, so the REV-12 test uses two different PRs.

</decisions>

<specifics>
## Specific Ideas

The fixture PRs for the live checks (an obvious bug, a clean change, a fork) are created by the owner, so the plan needs a human-action checkpoint for them.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `SPEC.md` — source of truth for states, concurrency, storage, git, agent input and contract, commands, and slice 1 "Done when".
- `prompts/review.md` — the review agent's system prompt.
- `schemas/review.schema.json` — the agent output schema.
- `AGENTS.md` — project conventions, testing rules, and verification commands (CLAUDE.md is a symlink to it).
- `.planning/phases/01-single-review/01-RESEARCH.md` — verified CLI, git, lock, and crate facts, the spike design, and the validation architecture.

</canonical_refs>

<code_context>
## Existing Code Insights

The crate is a scaffold: `src/main.rs` prints the name and version, and `tests/cli.rs` is a smoke test. Tooling (mise tasks, hk hooks, CI on ubuntu and macos, coverage gate, deps policy) is in place from quick task 261020-5pn.

</code_context>

<deferred>
## Deferred Ideas

None.

</deferred>
