# Requirements: diffmate

**Defined:** 2026-10-20
**Core Value:** Every PR where my review is directly requested gets an honest, ready-to-read draft without manual steps, and nothing reaches GitHub under my name until I act.

**Source of truth:** `SPEC.md` wins over this file wherever they differ. Requirements below come from SPEC.md "Delivery slices" (via `.planning/intel/requirements.md`). "I", "my", and "me" are the single user and owner, as in SPEC.md. Each item is the slice's "Done when" wording kept close to verbatim, so verification stays identical to the SPEC.

Intel crosswalk (the five intel entries are decomposed into checkable IDs; no scope is added except where an item is marked "scope line"):

| Intel entry | SPEC slice | Category | IDs |
|-------------|------------|----------|-----|
| REQ-single-review | 1. Single review | REV | REV-01 to REV-12 |
| REQ-reliability | 2. Reliability | REL | REL-01 to REL-13 |
| REQ-queue | 3. Queue | QUE | QUE-01 to QUE-11 |
| REQ-review-and-publish | 4. Review and publish | PUB | PUB-01 to PUB-15 |
| REQ-daily-use | 5. Daily use | OPS | OPS-01 to OPS-05 |

## v1 Requirements

### Single review (REV) - Phase 1

Slice description: the first task is a spike, since the agent contract depends on it. Then `diffmate review <url>` and `diffmate status`.

- [ ] **REV-01**: Spike, done first: establish on the target Claude CLI version (a) the isolation checks for the Claude CLI and git; (b) the CLI result envelope fields (`is_error`, `subtype`, `structured_output`, `total_cost_usd`, `duration_ms`); (c) the full output schema through `--json-schema`: `$defs`, `anyOf`, `minLength`, `maxItems`, nullable fields, and extraction of `structured_output`. Local validation with the full schema stays mandatory either way. If the CLI needs a simplified schema, the CLI form becomes a separate file with a test that checks it against the full schema. (Slice "first task"; no separate "Done when" line. It enables REV-05 to REV-09 and REV-11.)
- [x] **REV-02**: `diffmate review <url>` and `diffmate status` work end to end: enqueue a `review` job, take the runner lock, perform recovery, resolve the revision, fetch into the bare clone, create the worktree and context files, check CLI flag support, run the agent with the isolation flags, validate and normalize the output, store the run and its draft, clean up, and print the findings. (Slice description; it is the capability that REV-03 to REV-12 exercise.)
- [ ] **REV-03**: A PR with an obvious bug produces a finding anchored to the right line.
- [ ] **REV-04**: A clean PR produces `no_findings`.
- [ ] **REV-05**: An agent error with exit code 0 (for example, a turn limit) produces `failed`, not `no_findings`.
- [ ] **REV-06**: Claude isolation, checked with a canary string instead of a real secret, while authentication keeps working: a `CLAUDE.md` in the worktree containing the canary is not loaded.
- [ ] **REV-07**: Claude isolation, same canary method: a `CLAUDE.md` with an `@` import of a file outside the worktree containing the canary does not leak it.
- [ ] **REV-08**: Claude isolation, same canary method: `.claude/settings.json` with a hook, a plugin, and `.mcp.json` from the PR do not run or load.
- [x] **REV-09**: Git isolation: a smudge filter and a `core.fsmonitor` command defined in a test global git config, and a `.gitattributes` that selects the filter, do not run during fetch, worktree checkout, or diff.
- [ ] **REV-10**: A PR from a fork is reviewed the same way.
- [x] **REV-11**: A missing CLI flag fails the run before the agent starts.
- [x] **REV-12**: A second `review` while the first holds the lock waits, and its job runs after the first; `status` works meanwhile.

### Reliability (REL) - Phase 2

Slice description: idempotent runs, crash recovery, freshness, timeouts, `--force`.

- [ ] **REL-01**: Killing diffmate at any step and starting it again leaves no duplicate runs, no orphaned worktrees, and no stale worktree metadata, and interrupted runs show as `failed (interrupted: crash)`.
- [ ] **REL-02**: Pushing to the PR during a run makes the draft stale.
- [ ] **REL-03**: Advancing the base branch, without a push to the PR, does not make the draft stale.
- [ ] **REL-04**: Running `review` twice on the same head reuses the run unless `--force` is given.
- [ ] **REL-05**: An invalid `previous_finding_id` is nulled with a warning and the run still succeeds.
- [ ] **REL-06**: A PR whose review crashes diffmate every time is retried only within the retry budget, across restarts, and then reported as failed.
- [ ] **REL-07**: Recovery after a stop whose cause was stored but whose cleanup did not finish keeps the cause: `shutdown` re-queues an automatic job without using the budget, `user_cancel` ends the job without a retry.
- [ ] **REL-08**: Stopping `watch` with SIGTERM during an automatic run records `failed (interrupted: shutdown)`, removes the worktree, and re-queues the job without using the retry budget.
- [ ] **REL-09**: Ctrl-C on a `review` that acts as the runner stops and waits for the agent, records `failed (interrupted: user_cancel)` (never `agent_error`), leaves no agent or `rg` processes in its group, removes the worktree, and says so before exiting.
- [ ] **REL-10**: Without `watch`, a `review` that attached to a `tui` re-run executes it, and does not start orphaned or automatic jobs.
- [ ] **REL-11**: Two `review` commands for the same URL produce one run, a `review` started while a `tui` re-run is queued attaches to it, and `review --force` during a running non-forced job produces a second run.
- [ ] **REL-12**: Run timeout (slice scope line "timeouts"; SPEC "Agent contract"): a run that exceeds the timeout, 15 minutes by default, is recorded as `failed (timeout)`, and its agent process group is stopped with SIGTERM then SIGKILL after a bounded wait.
- [ ] **REL-13**: Configuration (SPEC "Configuration", needed from this slice on): `~/.config/diffmate/config.toml` is read with the SPEC defaults when absent or partial; this slice honors the run timeout and `max_retries` (default 2).

### Queue (QUE) - Phase 3

Slice description: `diffmate watch` with the queue rules, jobs handed over from `review`, and macOS notifications for ready drafts and abandoned automatic reviews.

- [ ] **QUE-01**: Requesting my review on a PR leads to a ready draft and a notification without any action from me.
- [ ] **QUE-02**: Two pushes within the debounce window produce one run.
- [ ] **QUE-03**: A PR stays tracked after I submit a review and my request disappears.
- [ ] **QUE-04**: A draft PR is not reviewed until it is marked ready.
- [ ] **QUE-05**: A review request to my team does not queue the PR.
- [ ] **QUE-06**: A PR I reviewed manually is not reviewed again on new pushes until my review is requested on it.
- [ ] **QUE-07**: A discarded PR with a still visible request stays untracked; removing and re-adding the request tracks it again.
- [ ] **QUE-08**: An agent failure is retried, and after the last retry a notification names the reason.
- [ ] **QUE-09**: `review` while `watch` runs hands the job to `watch` and prints the findings when it ends.
- [ ] **QUE-10**: Starting `watch` while `review` holds the lock makes `watch` wait, not exit.
- [ ] **QUE-11**: Configuration (SPEC "Configuration"): `config.toml` also honors the poll interval and `debounce` (default 60 seconds).

### Review and publish (PUB) - Phase 4

Slice description: `diffmate tui` shows targets and drafts. I can set decisions, edit findings and future improvements, include the limitation, open the PR in the browser, re-run, reconcile, abandon an `unknown` publication, and publish with `a` (approve), `r` (request changes), `c` (comment).

- [ ] **PUB-01**: `diffmate tui` shows the queue view (targets with tracking, latest run, draft freshness with the time of the last head check, and publication state) and the draft view (findings, limitation, future improvements). (Slice description.)
- [ ] **PUB-02**: In `tui` I can set a decision on each finding (`undecided`, `kept`, `edited`, `dropped`), edit findings and future improvements in `$EDITOR`, and include the limitation; a shortcut marks all remaining undecided findings `kept`. (Slice description and SPEC "Publication" step 2.)
- [ ] **PUB-03**: In `tui` I can open the PR in the browser, re-run a review, discard a target, reconcile an `unknown` publication, and abandon an `unknown` publication after checking. A re-run requested with no `watch` running is enqueued, and `tui` explains that `diffmate review` or `diffmate watch` must run it. (Slice description and SPEC "Commands".)
- [ ] **PUB-04**: Publishing creates one GitHub review on the reviewed commit, with kept and edited findings as inline comments carrying finding markers, and the publication marker in the body.
- [ ] **PUB-05**: Publishing with undecided findings is refused until every finding has a decision.
- [ ] **PUB-06**: Pressing a publish key twice creates one GitHub review, and publishing an already submitted draft is refused.
- [ ] **PUB-07**: Publishing a stale draft asks for confirmation first, and asks again if the head moves before the runner sends it.
- [ ] **PUB-08**: With an existing pending review of mine on the PR, publish stops with a clear message and changes nothing on GitHub.
- [ ] **PUB-09**: Publishing while `watch` is in the middle of an agent run completes without waiting for that run.
- [ ] **PUB-10**: A crash before `sending` is committed resumes safely after restart and publishes exactly once.
- [ ] **PUB-11**: A crash after `sending` is committed leaves the publication `unknown`, with no automatic second POST.
- [ ] **PUB-12**: Declining a `needs_confirmation` publication cancels the job and abandons the publication; redelivering the old job does not revive it.
- [ ] **PUB-13**: While `tui` holds the lock it shows that it acts as the runner and does not run `review` jobs.
- [ ] **PUB-14**: A simulated timeout on the GitHub call leaves the publication `unknown`, and a new attempt stays blocked until reconciliation finds the marker or I abandon it.
- [ ] **PUB-15**: A failed comment sync leaves the publication `submitted`, inline thread statuses `unknown`, and is retried without a new POST.

### Daily use (OPS) - Phase 5

Slice description: launchd agent for `watch` (plist lives in dotfiles), install instructions, and `diffmate doctor`.

- [ ] **OPS-01**: A launchd agent runs `watch`. Its plist lives in my dotfiles, outside this repo; this repo documents what it needs and how to install it. (Slice description.)
- [ ] **OPS-02**: Install instructions exist and are enough to set diffmate up on a fresh macOS machine. (Slice description.)
- [ ] **OPS-03**: `diffmate doctor` checks `gh` auth, Claude CLI version and flag support, storage paths, the runner lock, and the SQLite schema version. (SPEC "Commands".)
- [ ] **OPS-04**: `watch` survives a reboot.
- [ ] **OPS-05**: `doctor` explains every failed check.

## v2 Requirements

SPEC.md defines no post-MVP features, so there are no v2 items. Known limitations accepted for the MVP, not planned:

- A review request that is removed and re-added between two polls is missed (SPEC "Target").
- diffmate makes no guarantee for agent descendants that create their own process group (SPEC "Agent contract").
- There is no atomic transaction across SQLite and GitHub; diffmate relies on the publication ID, markers, reconciliation, and my confirmation (SPEC "Required properties", 3).

## Out of Scope

Explicitly excluded (SPEC.md "Non-goals").

| Feature | Reason |
|---------|--------|
| Automatically reviewing PRs nobody asked me to review | Only direct requests start tracking; a manual run on any PR I can read is allowed and never changes tracking |
| Review requests made to a team I belong to | The MVP handles direct requests only (`user-review-requested:@me`) |
| Running the code under review (tests, builds, scripts, hooks) | The agent is read-only; running PR code would defeat the isolation guarantee |
| Submitting, approving, or requesting changes without my explicit action | Publication requires my action and a decision on every finding |
| A team bot | diffmate acts as me, on my machine, with my `gh` credentials |
| Editing a diff inside the terminal | Editing happens in `$EDITOR` |
| Choosing the review language per PR | Findings are written in English |

## Cross-cutting invariants

SPEC.md "Required properties" 1-9 are hard invariants for every phase, not separate checklist items. The table maps each to where the "Done when" scenarios first prove it. The mapping is the roadmapper's derivation; the property text in SPEC.md is authoritative.

| Property | Where proven |
|----------|--------------|
| 1. Every run records its revision; every finding belongs to one run | REV-02 (Phase 1) |
| 2. Head is fetched before publication; a stale draft needs confirmation or a re-run | REL-02, REL-03 (Phase 2, freshness); PUB-07 (Phase 4) |
| 3. A crash creates no duplicate runs, drafts, worktrees, or GitHub reviews | REL-01 (Phase 2); PUB-06, PUB-10, PUB-11 (Phase 4) |
| 4. Failures are never "no findings"; a failed automatic review is never silent | REV-05 (Phase 1); QUE-08 (Phase 3) |
| 5. Publication needs my action and a decision on every finding | PUB-04, PUB-05 (Phase 4) |
| 6. Agent isolation | REV-01, REV-06 to REV-09, REV-11 (Phase 1) |
| 7. An existing pending review of mine is untouched | PUB-08 (Phase 4) |
| 8. One mutating process at a time | REV-12 (Phase 1); REL-10, REL-11 (Phase 2); QUE-10 (Phase 3); PUB-13 (Phase 4) |
| 9. Retries after crashes are bounded | REL-06 (Phase 2); QUE-08 (Phase 3) |

## Traceability

Which phases cover which requirements. Each requirement maps to exactly one phase.

| Requirement | Phase | Status |
|-------------|-------|--------|
| REV-01 | Phase 1 | Pending |
| REV-02 | Phase 1 | Complete |
| REV-03 | Phase 1 | Pending |
| REV-04 | Phase 1 | Pending |
| REV-05 | Phase 1 | Pending |
| REV-06 | Phase 1 | Pending |
| REV-07 | Phase 1 | Pending |
| REV-08 | Phase 1 | Pending |
| REV-09 | Phase 1 | Complete |
| REV-10 | Phase 1 | Pending |
| REV-11 | Phase 1 | Complete |
| REV-12 | Phase 1 | Complete |
| REL-01 | Phase 2 | Pending |
| REL-02 | Phase 2 | Pending |
| REL-03 | Phase 2 | Pending |
| REL-04 | Phase 2 | Pending |
| REL-05 | Phase 2 | Pending |
| REL-06 | Phase 2 | Pending |
| REL-07 | Phase 2 | Pending |
| REL-08 | Phase 2 | Pending |
| REL-09 | Phase 2 | Pending |
| REL-10 | Phase 2 | Pending |
| REL-11 | Phase 2 | Pending |
| REL-12 | Phase 2 | Pending |
| REL-13 | Phase 2 | Pending |
| QUE-01 | Phase 3 | Pending |
| QUE-02 | Phase 3 | Pending |
| QUE-03 | Phase 3 | Pending |
| QUE-04 | Phase 3 | Pending |
| QUE-05 | Phase 3 | Pending |
| QUE-06 | Phase 3 | Pending |
| QUE-07 | Phase 3 | Pending |
| QUE-08 | Phase 3 | Pending |
| QUE-09 | Phase 3 | Pending |
| QUE-10 | Phase 3 | Pending |
| QUE-11 | Phase 3 | Pending |
| PUB-01 | Phase 4 | Pending |
| PUB-02 | Phase 4 | Pending |
| PUB-03 | Phase 4 | Pending |
| PUB-04 | Phase 4 | Pending |
| PUB-05 | Phase 4 | Pending |
| PUB-06 | Phase 4 | Pending |
| PUB-07 | Phase 4 | Pending |
| PUB-08 | Phase 4 | Pending |
| PUB-09 | Phase 4 | Pending |
| PUB-10 | Phase 4 | Pending |
| PUB-11 | Phase 4 | Pending |
| PUB-12 | Phase 4 | Pending |
| PUB-13 | Phase 4 | Pending |
| PUB-14 | Phase 4 | Pending |
| PUB-15 | Phase 4 | Pending |
| OPS-01 | Phase 5 | Pending |
| OPS-02 | Phase 5 | Pending |
| OPS-03 | Phase 5 | Pending |
| OPS-04 | Phase 5 | Pending |
| OPS-05 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 56 total
- Mapped to phases: 56
- Unmapped: 0

---
*Requirements defined: 2026-10-20*
*Last updated: 2026-10-20 after initial definition*
