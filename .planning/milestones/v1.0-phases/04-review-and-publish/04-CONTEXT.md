# Phase 4: Review and publish - Context

**Gathered:** 2026-11-09
**Status:** Ready for planning

<domain>
## Phase Boundary

`diffmate tui` shows the queue and a draft. The owner decides every finding, edits what they keep in `$EDITOR`, and publishes exactly one GitHub review, or sees exactly why not. A crash, a double key press, or a timeout does not create a second review. SPEC.md slice 4, "States", "Publication", "Commands", and "One model for every process", plus ROADMAP.md Phase 4 (PUB-01 to PUB-15), fix the scope. The publication state machine and the Transitions table stay in SPEC.md. Planning uses that table as the test matrix and does not copy it. launchd and `doctor` stay in Phase 5. Phase 2 UAT and Phase 3 acceptance stay open. This phase does not call the Claude API.

</domain>

<decisions>
## Implementation Decisions

SPEC.md already fixes publication, the events `a` / `r` / `c`, decisions, freshness, and what `tui` runs. The decisions below cover only the screen, the keys, the editor file, and the confirmations. Questions the owner skipped take the recommended option and are marked as such.

### Reading the draft
- **D-01:** The draft screen is a list plus the full text of the selected finding. Each list row shows the anchor (`path:line` when present), severity, title, and decision. The text pane shows the selected finding's body.
- **D-02:** The queue and the draft are two full screens. Enter on a queue row opens its draft. Esc on the draft returns to the queue.
- **D-03:** The list contains findings only. Anchored findings come first, then unanchored findings, matching the order `review` already prints. Limitation and future improvements sit in a separate block under the selected text. They are not rows with a kept or dropped decision. The owner skipped this question. The recommended option was named in the next question, which the owner answered, and the discussion continued.
- **D-04:** The queue lists every target, newest target first. That is the reverse of `status`, which uses `ORDER BY t.id`. Each row shows the label, tracking, latest run, draft freshness with the time of the last head check, and publication state. An untracked target with a publication still appears.

### Keys
- **D-05:** On the selected finding, `k` sets `kept`, `d` sets `dropped`, and `e` opens `$EDITOR`. `a`, `r`, and `c` stay the publish events from SPEC.md: approve, request changes, and comment.
- **D-06:** Arrow keys move the selection. Enter and Esc switch screens, as in D-02. Letter keys are actions.
- **D-07:** The remaining keys are `K` (mark every remaining `undecided` finding `kept`, leaving `kept`, `dropped`, and `edited` as they are), `o` (open the PR in the browser), `n` (enqueue a re-run), `i` (include or exclude the limitation), `x` (discard the target), `y` (reconcile an `unknown` publication), `b` (abandon an `unknown` publication), and `q` (leave `tui`). The owner skipped this question. This is the recommended map. These keys do not replace `k`, `d`, `e`, `a`, `r`, `c`, the arrows, Enter, or Esc.
- **D-08:** `e` sets `edited` only when the saved text differs from the current text. A cancelled editor, or a save with the same text, leaves the decision as it was. The owner skipped this question. This is the recommended rule.

### Editor file
- **D-09:** `e` opens one temporary file for the selected finding, or for the one selected future improvement. Saving writes that piece back. The agent's original output stays unchanged in SQLite. `i` includes or excludes the limitation and does not open it in the editor. The owner skipped this question. This is the recommended shape.

### Confirmations
- **D-10:** On a current draft whose findings are all decided, `a`, `r`, or `c` creates the publication and enqueues the job. A failed pre-check is shown and creates nothing. The owner skipped the confirmation questions. This follows SPEC.md: the extra confirmation is for a stale draft.
- **D-11:** The safe refusal is the highlighted choice. A stale draft and a `needs_confirmation` head ask before sending, and the highlighted choice does not send. Declining `needs_confirmation` cancels the job and abandons the publication. `x` asks before discard, and the highlighted choice keeps the target. `b` asks before abandon, and the highlighted choice leaves the publication `unknown`. `q` leaves a queued publication in place.

### Claude's Discretion
- The temporary finding file is the current title, a blank line, and the body. A future improvement file is that note's text. An empty save stores nothing and leaves the decision unchanged.
- When `EDITOR` is unset, `tui` says so and leaves the finding unchanged. There is no fallback editor.
- The earlier finding's decision is a hint on the finding row. It is not a decision and is not inherited.
- The browser open uses the platform open for the PR URL.
- Comment sync retries on a planner-chosen interval, with paused tokio time in tests, and does not POST again.
- Wording of the queue row, the refusal messages, and the confirmation prompts, as long as D-04 and D-11 hold and a refusal names the SPEC.md reason.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Behavior
- `SPEC.md` — source of truth. Read "Concepts", "States" (including "Publication state", "Job", and "Transitions"), "Publication", "Commands", "One model for every process", and slice 4 "Done when". The Transitions table is the test matrix. Do not copy it into plans or tests as a second source. SPEC.md wins over this file.
- `.planning/REQUIREMENTS.md` — PUB-01 to PUB-15.
- `.planning/ROADMAP.md` — Phase 4 goal, success criteria, and planning notes. Failure injection needs a test seam on the `gh` boundary. Thread statuses come from GitHub GraphQL and feed `previous.json`.

### Prior decisions
- `.planning/phases/03-queue/03-CONTEXT.md` — D-02: the discard store method and hidden `debug discard` already exist. The TUI key is this phase.
- `.planning/phases/02-reliability/02-CONTEXT.md` — D-06 and D-07: freshness wording and the stale line on `review`.
- `.planning/phases/01-single-review/01-CONTEXT.md` — D-04: `review` prints `path:line [severity] title`, then the body, with unanchored findings after the anchored ones.
- `AGENTS.md` — conventions, paused tokio time, typed fakes, no new crate without a concrete need. Ratatui is the SPEC.md stack choice for this phase and is not a dependency yet.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/status.rs` and `src/store/status.rs`: one row per target, `ORDER BY t.id`. The queue shows the same facts and uses the reverse order (D-04). `status` stays read-only and keeps its order.
- `src/render/findings.rs`: anchored blocks, then unanchored findings, in the shape D-01 and D-03 follow.
- `src/store/poll.rs` `discard` and `src/debug.rs`: the discard latch. `x` calls that store method after the D-11 confirmation.
- `src/cli.rs`: `Review`, `Status`, `Watch`, and hidden `Debug`. `tui` is a new command.

### Established Patterns
- `gh`, `git`, and `claude` go through traits and `tokio::process` with an argument vector. Publication failure injection uses a fake at the `gh` boundary.
- Timers in tests use paused tokio time.
- No `ratatui` or `crossterm` dependency yet. Adding Ratatui is the SPEC.md stack requirement for this phase. Take the newest release at least three days old and name the crate in the PR. Do not pin a version in planning docs.

### Integration Points
- `tui` enqueues `publish`, `reconcile`, `sync`, and re-run jobs. While it holds the lock it runs `publish`, `reconcile`, and `sync` only, and it shows that it is the runner. It does not run `review` jobs.
- A re-run with no `watch` is enqueued, and `tui` says that `diffmate review` or `diffmate watch` must run it.

</code_context>

<specifics>
## Specific Ideas

No visual reference beyond the existing terminal text. The draft list uses the same `path:line [severity] title` shape as `review`. Freshness keeps the Phase 2 wording, such as `stale: head 9f3c2a1, checked 3m ago`.

</specifics>

<deferred>
## Deferred Ideas

- Phase 2 UAT stays open. Do not mark Phase 2 accepted from this context.
- Phase 3 has no verification or UAT record. Do not mark Phase 3 accepted from this context.
- Phase 5 owns launchd and `doctor`.

</deferred>

---

*Phase: 04-review-and-publish*
*Context gathered: 2026-11-09*
