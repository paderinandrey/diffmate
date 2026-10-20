# Roadmap: diffmate

## Overview

diffmate is built in five phases, one per SPEC delivery slice, in SPEC order. Phase 1 proves a single honest, isolated review of one PR, starting with a spike on the Claude CLI contract. Phase 2 makes that review safe to repeat and to kill at any point. Phase 3 adds `watch`, so a directly requested PR gets a draft and a notification with no action from me. Phase 4 adds `tui` and the publication state machine, so I can decide on findings and publish exactly one review. Phase 5 puts `watch` under launchd and adds `doctor`. Each phase ends with scenarios I can check by hand; the success criteria below are the slice's "Done when" items from `SPEC.md`, which wins over this file wherever they differ.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Single review** - `diffmate review <url>` and `status` with an isolated agent, validated output, runner lock, bare clone and worktree
- [ ] **Phase 2: Reliability** - idempotent runs, crash recovery, freshness, timeouts, `--force`, signal handling, job attachment
- [ ] **Phase 3: Queue** - `diffmate watch` with tracking, debounce, retries, job handoff, and macOS notifications
- [ ] **Phase 4: Review and publish** - `diffmate tui`, decisions, editing, and the publication state machine with reconciliation and comment sync
- [ ] **Phase 5: Daily use** - launchd agent for `watch`, install instructions, and `diffmate doctor`

## Phase Details

### Phase 1: Single review

**Goal**: I can run `diffmate review <url>` on a PR and get an honest, line-anchored draft from an agent that provably cannot run commands, write files, or load anything from the PR, and a failure is never shown as "no findings".
**Depends on**: Nothing (first phase)
**Requirements**: REV-01, REV-02, REV-03, REV-04, REV-05, REV-06, REV-07, REV-08, REV-09, REV-10, REV-11, REV-12
**Success Criteria** (what must be TRUE):
  1. A PR with an obvious bug produces a finding anchored to the right line. [REV-03]
  2. A clean PR produces `no_findings`. [REV-04]
  3. An agent error with exit code 0 (for example, a turn limit) produces `failed`, not `no_findings`. [REV-05]
  4. Claude isolation, checked with a canary string instead of a real secret, while authentication keeps working:
     - a `CLAUDE.md` in the worktree containing the canary is not loaded; [REV-06]
     - a `CLAUDE.md` with an `@` import of a file outside the worktree containing the canary does not leak it; [REV-07]
     - `.claude/settings.json` with a hook, a plugin, and `.mcp.json` from the PR do not run or load. [REV-08]
  5. Git isolation: a smudge filter and a `core.fsmonitor` command defined in a test global git config, and a `.gitattributes` that selects the filter, do not run during fetch, worktree checkout, or diff. [REV-09]
  6. A PR from a fork is reviewed the same way. [REV-10]
  7. A missing CLI flag fails the run before the agent starts. [REV-11]
  8. A second `review` while the first holds the lock waits, and its job runs after the first; `status` works meanwhile. [REV-12]

**Entry task**: spike first (REV-01). Before the rest of the slice, establish on the target Claude CLI version: the isolation checks for the Claude CLI and git; the result envelope fields (`is_error`, `subtype`, `structured_output`, `total_cost_usd`, `duration_ms`); and whether the full output schema (`$defs`, `anyOf`, `minLength`, `maxItems`, nullable fields) goes through `--json-schema`, with `structured_output` extraction. Local validation with the full schema stays mandatory. If the CLI needs a simplified schema, that form is a separate file with a test that checks it against the full schema. REV-01 and REV-02 are the capability that criteria 1-8 exercise and have no "Done when" line of their own.
**Planning notes**:
  - The SQLite schema and job model must stay compatible with the publication, decision, retry-budget, and automatic/manual job rules that later phases implement (SPEC "States", "Concurrency").
  - Toolchain, crate, and CLI versions are looked up when work starts, never taken from the `cargo init` scaffold or from memory.

**Plans:** 1/12 plans executed

Plans:
**Wave 1**
- [ ] 01-01-PLAN.md — Spike (REV-01): flag contract, envelope matrix, schema form, auth under isolation, fake-endpoint and control feasibility
- [ ] 01-02-PLAN.md — Owner-created fixture PRs (obvious bug, clean, fork), verified and recorded
- [x] 01-03-PLAN.md — Black-box test harness: sandbox, fake claude and gh, git shim, upstream builder

**Wave 2** *(blocked on Wave 1 completion)*
- [ ] 01-04-PLAN.md — Tracer: `diffmate review URL` end to end through every layer

**Wave 3** *(blocked on Wave 2 completion)*
- [ ] 01-05-PLAN.md — Agent contract: version and flag probe (REV-11), D-01 classification (REV-05), schema form, env allowlist
- [ ] 01-06-PLAN.md — Git hardening, git isolation (REV-09) and fork PRs (REV-10)
- [ ] 01-07-PLAN.md — Diff hunk index, anchor normalization (REV-03) and D-04 output

**Wave 4** *(blocked on Wave 3 completion)*
- [ ] 01-08-PLAN.md — Runner lock and waiting (REV-12), recovery, `diffmate status`
- [ ] 01-09-PLAN.md — Full context: caps, previous.json, rules at merge-base, discussion.md, ci.md

**Wave 5** *(blocked on Wave 4 completion)*
- [ ] 01-10-PLAN.md — Minimal SIGINT/SIGTERM handling with agent group stop (D-03)
- [ ] 01-11-PLAN.md — Claude isolation suite on a fake endpoint with positive controls (REV-06 to REV-08)

**Wave 6** *(blocked on Wave 5 completion)*
- [ ] 01-12-PLAN.md — Live acceptance (REV-03, REV-04, REV-05, REV-10) and phase gate

### Phase 2: Reliability

**Goal**: A review can be killed, interrupted, repeated, or overtaken by new pushes at any point without duplicate runs, orphaned worktrees, a lost stop cause, or unbounded retries, and the draft always says whether it still matches the PR head.
**Depends on**: Phase 1
**Requirements**: REL-01, REL-02, REL-03, REL-04, REL-05, REL-06, REL-07, REL-08, REL-09, REL-10, REL-11, REL-12, REL-13
**Success Criteria** (what must be TRUE):
  1. Killing diffmate at any step and starting it again leaves no duplicate runs, no orphaned worktrees, and no stale worktree metadata, and interrupted runs show as `failed (interrupted: crash)`. [REL-01]
  2. Pushing to the PR during a run makes the draft stale. [REL-02]
  3. Advancing the base branch, without a push to the PR, does not make the draft stale. [REL-03]
  4. Running `review` twice on the same head reuses the run unless `--force` is given. [REL-04]
  5. An invalid `previous_finding_id` is nulled with a warning and the run still succeeds. [REL-05]
  6. A PR whose review crashes diffmate every time is retried only within the retry budget, across restarts, and then reported as failed. [REL-06]
  7. Recovery after a stop whose cause was stored but whose cleanup did not finish keeps the cause: `shutdown` re-queues an automatic job without using the budget, `user_cancel` ends the job without a retry. [REL-07]
  8. Stopping `watch` with SIGTERM during an automatic run records `failed (interrupted: shutdown)`, removes the worktree, and re-queues the job without using the retry budget. [REL-08]
  9. Ctrl-C on a `review` that acts as the runner stops and waits for the agent, records `failed (interrupted: user_cancel)` (never `agent_error`), leaves no agent or `rg` processes in its group, removes the worktree, and says so before exiting. [REL-09]
  10. Without `watch`, a `review` that attached to a `tui` re-run executes it, and does not start orphaned or automatic jobs. [REL-10]
  11. Two `review` commands for the same URL produce one run, a `review` started while a `tui` re-run is queued attaches to it, and `review --force` during a running non-forced job produces a second run. [REL-11]

**Planning notes**:
  - Slice scope also names timeouts. The run timeout (REL-12) and the first reads of `config.toml` (REL-13) are planned and tested here although no "Done when" line names them; a short timeout set in `config.toml` is the natural hand check.
  - Several scenarios run the `watch` and `tui` roles before those features exist. The plan needs a minimal `watch` runner loop (executes every job, handles SIGTERM and SIGINT) and a way to enqueue an automatic job and a `tui`-style `review` job as a test seam. Polling, tracking, debounce, and notifications stay in Phase 3; the TUI stays in Phase 4.
  - This phase introduces the automatic/manual job distinction and the retry budget and next-attempt time persisted in SQLite, and `status` gains draft freshness with the time of the last head check.

**Plans**: TBD

### Phase 3: Queue

**Goal**: A PR where my review is directly requested gets a ready draft and a notification without any action from me, and a failed automatic review is never silent.
**Depends on**: Phase 2
**Requirements**: QUE-01, QUE-02, QUE-03, QUE-04, QUE-05, QUE-06, QUE-07, QUE-08, QUE-09, QUE-10, QUE-11
**Success Criteria** (what must be TRUE):
  1. Requesting my review on a PR leads to a ready draft and a notification without any action from me. [QUE-01]
  2. Two pushes within the debounce window produce one run. [QUE-02]
  3. A PR stays tracked after I submit a review and my request disappears. [QUE-03]
  4. A draft PR is not reviewed until it is marked ready. [QUE-04]
  5. A review request to my team does not queue the PR. [QUE-05]
  6. A PR I reviewed manually is not reviewed again on new pushes until my review is requested on it. [QUE-06]
  7. A discarded PR with a still visible request stays untracked; removing and re-adding the request tracks it again. [QUE-07]
  8. An agent failure is retried, and after the last retry a notification names the reason. [QUE-08]
  9. `review` while `watch` runs hands the job to `watch` and prints the findings when it ends. [QUE-09]
  10. Starting `watch` while `review` holds the lock makes `watch` wait, not exit. [QUE-10]

**Planning notes**:
  - The discard rules (QUE-07) are implemented here, but the user-facing discard key in `tui` arrives in Phase 4 (PUB-03). The hand check in this phase sets the discard state through a test seam; the planner chooses the seam.
  - SPEC.md does not fix the macOS notification mechanism. Pick it in planning, with the latest stable version of whatever it uses. Candidate research item.
  - `config.toml` gains the poll interval and `debounce` (QUE-11).

**Plans**: TBD

### Phase 4: Review and publish

**Goal**: I can read a draft in `tui`, decide on every finding, edit what I keep, and publish exactly one GitHub review under my name, or learn precisely why not, with no duplicate review after any crash, double key press, or timeout.
**Depends on**: Phase 3
**Requirements**: PUB-01, PUB-02, PUB-03, PUB-04, PUB-05, PUB-06, PUB-07, PUB-08, PUB-09, PUB-10, PUB-11, PUB-12, PUB-13, PUB-14, PUB-15
**Success Criteria** (what must be TRUE):
  1. Publishing creates one GitHub review on the reviewed commit, with kept and edited findings as inline comments carrying finding markers, and the publication marker in the body. [PUB-04]
  2. Publishing with undecided findings is refused until every finding has a decision. [PUB-05]
  3. Pressing a publish key twice creates one GitHub review, and publishing an already submitted draft is refused. [PUB-06]
  4. Publishing a stale draft asks for confirmation first, and asks again if the head moves before the runner sends it. [PUB-07]
  5. With an existing pending review of mine on the PR, publish stops with a clear message and changes nothing on GitHub. [PUB-08]
  6. Publishing while `watch` is in the middle of an agent run completes without waiting for that run. [PUB-09]
  7. A crash before `sending` is committed resumes safely after restart and publishes exactly once. [PUB-10]
  8. A crash after `sending` is committed leaves the publication `unknown`, with no automatic second POST. [PUB-11]
  9. Declining a `needs_confirmation` publication cancels the job and abandons the publication; redelivering the old job does not revive it. [PUB-12]
  10. While `tui` holds the lock it shows that it acts as the runner and does not run `review` jobs. [PUB-13]
  11. A simulated timeout on the GitHub call leaves the publication `unknown`, and a new attempt stays blocked until reconciliation finds the marker or I abandon it. [PUB-14]
  12. A failed comment sync leaves the publication `submitted`, inline thread statuses `unknown`, and is retried without a new POST. [PUB-15]

**Planning notes**:
  - PUB-01 to PUB-03 (the `tui` views, decisions, editing, browser, re-run, reconcile, abandon, discard) have no "Done when" line of their own; criteria 1-12 exercise them.
  - The publication, job, and recovery rules are fully specified in SPEC.md ("States", "Publication", "Transitions"). The Transitions table there is the test matrix for this phase; do not re-derive or copy it.
  - Failure injection on the GitHub call (timeouts, client and server errors) needs a test seam on the `gh` boundary.
  - Thread statuses are read from GitHub GraphQL and feed `previous.json` for later re-reviews; comment links come from comment sync.

**Plans**: TBD
**UI hint**: yes

### Phase 5: Daily use

**Goal**: diffmate runs unattended day to day: `watch` is started by launchd and survives a reboot, and `doctor` tells me exactly what is wrong when something is.
**Depends on**: Phase 4
**Requirements**: OPS-01, OPS-02, OPS-03, OPS-04, OPS-05
**Success Criteria** (what must be TRUE):
  1. `watch` survives a reboot. [OPS-04]
  2. `doctor` explains every failed check. [OPS-05]

**Planning notes**:
  - The launchd plist lives in my dotfiles, outside this repo; this phase delivers the install instructions (OPS-02) and the `watch` behavior the agent relies on (waits for the lock instead of exiting, exits cleanly on SIGTERM).
  - OPS-01 and OPS-03 have no "Done when" line of their own; the two criteria exercise them. `doctor` checks `gh` auth, Claude CLI version and flag support, storage paths, the runner lock, and the SQLite schema version.
  - The reboot check is manual by nature.

**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Single review | 1/12 | In Progress | - |
| 2. Reliability | 0/TBD | Not started | - |
| 3. Queue | 0/TBD | Not started | - |
| 4. Review and publish | 0/TBD | Not started | - |
| 5. Daily use | 0/TBD | Not started | - |
