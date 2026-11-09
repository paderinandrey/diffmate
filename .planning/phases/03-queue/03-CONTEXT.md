# Phase 3: Queue - Context

**Gathered:** 2026-11-09
**Status:** Ready for planning

<domain>
## Phase Boundary

`diffmate watch` polls GitHub for PRs where the owner's review is requested directly, tracks those targets, debounces new heads, and produces a ready draft plus a macOS notification with no action from the owner. A failed automatic review is retried and, after the last retry, notified with the reason. `review` started while `watch` holds the lock hands its job to `watch` and prints the findings when that job ends. Starting `watch` while `review` holds the lock waits instead of exiting. SPEC.md slice 3, "Queue rules", "Target" tracking, and ROADMAP.md Phase 3 (QUE-01 to QUE-11) fix the scope. The TUI, publication, and the discard key stay in Phase 4. launchd and `doctor` stay in Phase 5. Phase 2 acceptance is still open and the Anthropic HTTP 403 live gate stays blocked; this phase does not call the Claude API to clear that gate.

</domain>

<decisions>
## Implementation Decisions

SPEC.md already fixes what is tracked, when a draft PR is skipped, how debounce and retries work, and that a team review request does not queue a PR. The decisions below cover only what SPEC.md leaves open. They were selected in `--auto` because the owner said to continue past the open Phase 2 UAT.

### macOS notifications
- **D-01:** Notifications go through `osascript -e` with an argument vector and `display notification "…" with title "…"`. No new crate. The title distinguishes a ready draft from an abandoned automatic review. The body is the PR label, plus the failure reason when the automatic review has given up. Text is passed through the existing sanitizer. A failed `osascript` is logged and does not fail the job or change retry state. Only automatic runs notify: a ready draft after success, and a give-up after the last retry. A `shutdown` requeue is not a give-up and does not notify. Manual `review` does not notify.

### Discard before `tui`
- **D-02:** The discard key stays in Phase 4. This phase adds a hidden `diffmate debug discard <url>` next to `debug enqueue`. It is in the release binary, absent from `--help`, and calls one store method. That method sets `tracking` to `untracked` and `request_seen` to seen, so a request that is still visible does not track the target again. Phase 4's TUI calls the same method. `debug enqueue` stays.

### Poll
- **D-03:** Each poll runs `gh search prs` with `--review-requested=@me`, `--state=open`, and `--json` including `isDraft`, `number`, `repository`, and `url`. No `--draft` flag, so drafts are returned and then skipped. The search is paginated until a short page; the default limit of 30 is not the end of the list. The call goes through the `Gh` trait with a typed fake. A failed search changes nothing, including `request_seen`.
- **D-04:** `watch` polls on `poll_interval_seconds` from `config.toml` (default 60, already parsed in Phase 2). The Phase 2 one-second idle wait stays only for the lock retry and for sleeping until the next due job or the next poll, whichever is sooner. Tests use paused tokio time.
- **D-05:** For each already tracked target missing from the search, one read-only `gh pr view` reads `state` and `isDraft`. Closed or merged sets `tracking` to `untracked` and does not by itself clear `request_seen`. A failed view skips that target. A draft is not enqueued. Becoming ready later is an ordinary poll hit.

### Request seen and debounce
- **D-06:** Schema moves to version 3 with the Phase 2 backup pattern. `targets` gains `request_seen` (`0` or `1`, default `0`). There is no third tracking value. A new direct request while `request_seen` is `0` sets `tracked` and `request_seen` to `1`. A request that disappears on a successful poll sets `request_seen` to `0` and leaves a tracked target tracked. Discard and close leave the target `untracked`; it becomes tracked again only after a successful poll with no request and a later successful poll with one. A manual `review` does not change tracking or `request_seen`.
- **D-07:** A new head on a tracked, ready target enqueues one automatic `review` job due at `now + debounce_seconds`, using the existing `next_attempt_at`. The head is resolved when the job starts, so two pushes before the job is due leave one job and reset `next_attempt_at`. A head change during a running job does not cancel it. When that run finishes, the draft is stale as Phase 2 already records, and one new job is queued after the debounce. Retry delays stay on the same column and are not reset by a debounce.

### Handoff
- **D-08:** QUE-09 and QUE-10 use the Phase 2 lock and attach behavior. `watch` remains the runner for every due review job, including one a waiting `review` attached to. The waiting `review` prints the stored findings and does not start the agent. `watch` still does not print findings.

### Claude's Discretion
- Notification title and body wording, as long as a give-up names the failure reason and a ready draft names the PR.
- The `gh` JSON fields and the page size, as long as drafts are visible, team requests are absent, and a list longer than one page is not truncated.
- How `status` shows `request_seen`, if it shows it at all. Tracking is already on the status line.
- The exact SQL for resetting `next_attempt_at` on an already queued automatic job.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Behavior
- `SPEC.md` — "Concepts" (tracking, request seen), "Target" states, "Queue rules", "Commands" (`watch`), "Configuration", slice 3 "Done when". SPEC.md wins over this file.
- `.planning/REQUIREMENTS.md` — QUE-01 to QUE-11.
- `.planning/ROADMAP.md` — Phase 3 success criteria. The discard check needs a seam because the TUI key arrives in Phase 4. The notification mechanism is chosen here.

### Prior decisions
- `.planning/phases/02-reliability/02-CONTEXT.md` — D-01 and D-02 (`watch` is the runner and does not poll; `debug enqueue`), D-08 to D-11 (`debounce_seconds` and `poll_interval_seconds` are parsed and unused).
- `AGENTS.md` — conventions, paused tokio time, typed fakes, no new crate without a concrete need.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/watch.rs`, `src/watch/run.rs`, `src/watch/wait.rs`: `watch` takes the lock, runs recovery, then executes due jobs. `WATCH_IDLE_POLL` is one second. SIGINT and SIGTERM are `shutdown`. It does not poll GitHub.
- `src/config.rs`: `debounce_seconds` and `poll_interval_seconds` default to 60 and are validated. Nothing reads them yet.
- `src/debug.rs` and `src/main.rs`: hidden `debug enqueue`.
- `src/store/schema.sql`: `targets.tracking` is `untracked` or `tracked`. There is no `request_seen`. `jobs.next_attempt_at` already delays automatic retries.
- `src/store/migrate.rs`: `user_version` 2. A v1 database is backed up before the v2 migration.
- `src/gh.rs`: `gh` goes through a trait and `tokio::process` with an argument vector. There is no search call yet.
- `src/review` wait path: a `review` that cannot take the lock waits and can attach. Phase 3 must not start a second runner.

### Established Patterns
- External processes use tokio and an explicit argument vector. No shell.
- Timers in tests use paused tokio time.
- Fakes for `gh`, `git`, and `claude` are hand-written. Live `gh` or `claude` checks are `#[ignore]`.

### Integration Points
- The poller writes tracking and enqueues jobs. The existing runner loop executes them.
- `status` already prints tracking. It stays read-only.

</code_context>

<specifics>
## Specific Ideas

No product reference beyond SPEC.md. Notifications are the macOS Notification Center entries `osascript` already creates. A click action is not required.

</specifics>

<deferred>
## Deferred Ideas

- The discard key in `tui` — Phase 4. The store method and `debug discard` are the seam.
- Choosing a different notifier if `osascript` cannot show a notification on the owner's macOS — only after a hand check fails. Do not add `notify-rust` in this phase.
- Phase 2 UAT (12 pending checks and 2 blocked live checks) — still open. Do not mark Phase 2 accepted from this context.

</deferred>
