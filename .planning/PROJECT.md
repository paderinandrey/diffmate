# diffmate

## What This Is

diffmate is a personal pull-request review agent written in Rust. For every PR where my review is directly requested, an agent reviews the change in an isolated git worktree and diffmate prepares a draft: findings anchored to diff lines, an optional limitation, and up to two future improvements. I read the draft, decide on every finding, edit what I keep, and submit the review myself. It runs on my own macOS machine with my own `gh` credentials. Private repo: paderinandrey/diffmate.

In this project "I", "my", and "me" mean the single user and owner, the same voice SPEC.md uses.

## Core Value

Every PR where my review is directly requested gets an honest, ready-to-read draft without manual steps, and nothing reaches GitHub under my name until I act.

## Source of Truth

**`SPEC.md` (repo root) is the source of truth. If SPEC.md and anything in `.planning/` differ, SPEC.md wins.** That covers this file, REQUIREMENTS.md, ROADMAP.md, STATE.md, and the digests under `.planning/intel/`.

- `SPEC.md`: the design, states, required properties, delivery slices, and stack (ingest precedence 0).
- `prompts/review.md`: the review agent's system prompt, passed to the Claude CLI (ingest precedence 1). Its finding-writing rules are agent behavior, not project requirements.
- `schemas/review.schema.json`: the review output schema that SPEC.md designates (JSON Schema draft 2020-12).
- `.planning/intel/`: a derived digest of the above. Use it as an index, never as an override.
- The job x publication transitions table lives only in SPEC.md ("Transitions"). Planning artifacts reference it and never copy it.

## Success Metric

- The five SPEC delivery slices each pass their "Done when" scenarios by hand.
- Day to day: every PR where my review is directly requested gets a ready draft without manual steps, and nothing reaches GitHub without my action.

## Requirements

### Validated

(None yet - ship to validate)

### Active

Five groups, one per SPEC delivery slice, in SPEC order. Full text and IDs are in `.planning/REQUIREMENTS.md`.

- [ ] Single review (REV-01 to REV-12): `diffmate review <url>` and `diffmate status`, Claude CLI and git isolation, output validation, runner lock, bare clone and worktree. Starts with a spike on the agent contract.
- [ ] Reliability (REL-01 to REL-13): idempotent runs, crash recovery, draft freshness, timeouts, `--force`, signal handling, job attachment.
- [ ] Queue (QUE-01 to QUE-11): `diffmate watch`, tracking, debounce, retries, macOS notifications, job handoff.
- [ ] Review and publish (PUB-01 to PUB-15): `diffmate tui`, decisions, editing, publication state machine, reconciliation, comment sync.
- [ ] Daily use (OPS-01 to OPS-05): launchd agent for `watch`, install instructions, `diffmate doctor`.

### Out of Scope

These are the SPEC non-goals.

- Automatically reviewing PRs nobody asked me to review - a manual run on any PR I can read is allowed, but it never starts tracking.
- Review requests made to a team I belong to - the MVP handles direct requests only.
- Running the code under review (tests, builds, scripts, hooks) - the agent is read-only, which is part of the isolation guarantee.
- Submitting, approving, or requesting changes without my explicit action - publication always needs my action and a decision on every finding.
- A team bot - diffmate acts as me, on my machine, with my `gh` credentials.
- Editing a diff inside the terminal - editing happens in `$EDITOR`.
- Choosing the review language per PR - findings are written in English.

## Context

- Runtime: macOS only (launchd, `~/Library` paths), single user, my own `gh` auth. No server, no team features.
- Repo state at roadmap time: SPEC.md, `prompts/review.md`, `schemas/review.schema.json`, and a `cargo init` scaffold (`Cargo.toml` with no dependencies, a hello-world `src/main.rs`). No product code yet.
- How it works, in brief (SPEC.md governs every detail):
  - Every process (`review`, `watch`, `tui`) enqueues a job in SQLite (WAL mode). The one process holding the exclusive `flock` on `~/Library/Application Support/diffmate/runner.lock` is the runner: it performs recovery first, then executes the jobs it is responsible for.
  - A review run fetches the PR head and base into a diffmate-owned bare clone, makes a detached worktree under `~/Library/Caches/diffmate/runs/<run-id>/`, writes the agent's input files, and runs `claude -p` with isolation flags and a scrubbed environment. Output is validated against the full schema, normalized, and stored as the run and its draft.
  - Git is run by diffmate with a controlled environment: no global or system config, no inherited `GIT_*` variables, diffs with `--no-ext-diff --no-textconv`.
  - The agent never calls GitHub. Publication is a `publish` job with a stable publication ID, an immutable snapshot, markers in the review, a `sending` commit before the POST, reconciliation for `unknown` outcomes, and never an automatic second POST.
- Accepted MVP limitations (from SPEC.md): a review request that is removed and re-added between two polls is missed; processes that create their own process group are not guaranteed to be stopped; managed (policy) settings of the Claude CLI still apply, so isolation is proven by the slice 1 checks and not assumed.

## Constraints

- **Tech stack (locked, DEC-stack)**: Rust; Ratatui for the TUI; tokio for the queue and subprocesses; rusqlite with bundled SQLite for state; the `jsonschema` crate for output validation; GitHub access only through the `gh` CLI - fixed by SPEC.md "Stack" and the project owner.
- **Latest stable everywhere**: always the latest stable version of every crate, bundled SQLite, the Rust toolchain and edition, and all other tooling (`gh`, the Claude CLI, CI actions if any are ever added). Look the version up at the time of use. Never write a version from memory, and do not pin a version in planning documents - the scaffold `Cargo.toml` is only a `cargo init` placeholder to be re-checked when real work starts.
- **Platform**: macOS only, one user, launchd for the daily-use agent.
- **Required properties 1-9**: SPEC.md "Required properties" are hard invariants: revision per run, freshness check before publication, crash safety without duplicates, failures never reported as "no findings", publication only by my action, agent isolation, an existing pending review is never touched, a single mutating process, bounded retries after crashes. `.planning/REQUIREMENTS.md` maps each to the phase that first proves it.
- **Isolation**: the agent has Read, Grep, and Glob only, no network, no MCP servers, no instructions, imports, hooks, or plugins from the PR, no GitHub credentials. Whether the Claude CLI flags actually hold is established by checks in Phase 1.
- **Storage**: diffmate data lives only in its own directories; my working copies are never used (SPEC.md "Storage").
- **Output contract**: `schemas/review.schema.json` is validated locally with the full schema and JSON Schema draft 2020-12 whether or not the CLI accepts it; the `$schema` key is removed before the schema goes to `--json-schema`.
- **Configuration**: `~/.config/diffmate/config.toml` (poll interval, debounce, run timeout, `max_retries`).

<decisions>
## Locked Decisions

- **DEC-stack (LOCKED):** Rust. Ratatui for the TUI, tokio for the queue and subprocesses, rusqlite for state (bundled SQLite), `jsonschema` for output validation. GitHub access goes through the `gh` CLI. Source: SPEC.md "Stack". Scope: implementation language, TUI, async runtime and subprocesses, state storage, output validation, GitHub access. Not open to revision during planning or execution; a change needs an explicit decision from the owner and an update to SPEC.md first.
</decisions>

## Working Conventions

- **GSD in lightweight mode**: research and plan-checker only where a phase actually needs them; small items go through `gsd-quick`; TDD, diagnosis, and review are applied inside tasks; there is no second planning cycle.
- **Pull requests**: create with `gh pr create --assignee @me`. Right after creation, post a PR comment `@codex review` to request Codex review (for example `gh pr comment <number> --body "@codex review"`).
- **No agent attribution**: commit messages carry no `Co-Authored-By` trailer naming an agent, and PR bodies carry no "Generated with" footer, link, or badge.
- **Code comments**: none by default. A comment is allowed only for an external constraint the code cannot express (a third-party API quirk, a CLI or OS limitation). Design rationale goes to the PR description, not the code.
- **Language**: English for code, docs, and the agent's findings.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Stack: Rust, Ratatui, tokio, rusqlite (bundled SQLite), `jsonschema`, `gh` CLI (DEC-stack) | Fixed by SPEC.md "Stack"; locked | Locked |
| SPEC.md is the source of truth; planning artifacts follow it | The SPEC carries the design and the exact "Done when" scenarios | Pending |
| Exactly five phases, one per SPEC delivery slice, in SPEC order; slices are not split or merged | Each slice ends with a scenario I can check by hand (owner instruction) | Pending |
| Phase success criteria are the slice's "Done when" items | Keeps verification identical to the SPEC (owner instruction) | Pending |
| Phase 1 opens with a spike on the agent contract | Isolation flags, the result envelope, and full-schema support through `--json-schema` are unproven until checked, and the agent contract depends on them | Pending |
| Requirement IDs are per-slice categories (REV, REL, QUE, PUB, OPS), decomposed from the five intel `REQ-*` entries | Planning tools track `CAT-NN` IDs; the decomposition adds no new scope beyond the SPEC | Pending |
| Latest stable versions of all libraries and tooling, looked up at the time of use | Owner instruction; avoids stale pins and memory-based versions | Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition**:
1. Requirements invalidated? Move to Out of Scope with the reason.
2. Requirements validated? Move to Validated with the phase reference.
3. New requirements emerged? Add to Active.
4. Decisions to log? Add to Key Decisions.
5. "What This Is" still accurate? Update if it drifted.

**After each milestone**:
1. Full review of all sections.
2. Core Value check - still the right priority?
3. Audit Out of Scope - reasons still valid?
4. Update Context with the current state.

---
*Last updated: 2026-10-20 after roadmap creation from the SPEC.md ingest*
