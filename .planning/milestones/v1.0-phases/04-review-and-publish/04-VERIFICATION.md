---
phase: 04-review-and-publish
verified: 2026-11-11T09:05:23Z
status: passed
score: 29/29 must-haves verified
behavior_unverified: 0
overrides_applied: 0
decision_coverage:
  honored: 11
  total: 11
  not_honored: []
covered_files:
  - ".planning/phases/04-review-and-publish/04-01-PLAN.md"
  - ".planning/phases/04-review-and-publish/04-01-SUMMARY.md"
  - ".planning/phases/04-review-and-publish/04-02-PLAN.md"
  - ".planning/phases/04-review-and-publish/04-02-SUMMARY.md"
  - ".planning/phases/04-review-and-publish/04-03-PLAN.md"
  - ".planning/phases/04-review-and-publish/04-03-SUMMARY.md"
  - ".planning/phases/04-review-and-publish/04-04-PLAN.md"
  - ".planning/phases/04-review-and-publish/04-04-SUMMARY.md"
  - ".planning/phases/04-review-and-publish/04-05-PLAN.md"
  - ".planning/phases/04-review-and-publish/04-05-SUMMARY.md"
  - ".planning/phases/04-review-and-publish/04-06-PLAN.md"
  - ".planning/phases/04-review-and-publish/04-06-SUMMARY.md"
  - ".planning/phases/04-review-and-publish/04-07-PLAN.md"
  - ".planning/phases/04-review-and-publish/04-07-SUMMARY.md"
  - ".planning/phases/04-review-and-publish/04-08-PLAN.md"
  - ".planning/phases/04-review-and-publish/04-08-SUMMARY.md"
  - ".planning/phases/04-review-and-publish/04-09-PLAN.md"
  - ".planning/phases/04-review-and-publish/04-09-SUMMARY.md"
  - ".planning/phases/04-review-and-publish/04-10-PLAN.md"
  - ".planning/phases/04-review-and-publish/04-10-SUMMARY.md"
  - "src/context/previous.rs"
  - "src/gh.rs"
  - "src/gh/reviews.rs"
  - "src/gh/threads.rs"
  - "src/main.rs"
  - "src/publish/body.rs"
  - "src/publish/checks.rs"
  - "src/publish/gate.rs"
  - "src/publish/outcome/mod.rs"
  - "src/publish/post.rs"
  - "src/publish/reconcile/mod.rs"
  - "src/publish/reconcile/write.rs"
  - "src/publish/sync.rs"
  - "src/publish/sync/apply.rs"
  - "src/publish/sync/apply/write.rs"
  - "src/publish/tests/confirm/budget.rs"
  - "src/publish/tests/crash.rs"
  - "src/publish/tests/crash/gh.rs"
  - "src/publish/tests/crash/keys.rs"
  - "src/publish/tests/crash/marker.rs"
  - "src/publish/tests/crash/outcome.rs"
  - "src/publish/tests/sync.rs"
  - "src/status.rs"
  - "src/status/tests/stored.rs"
  - "src/store/decisions.rs"
  - "src/store/migrate.rs"
  - "src/store/migrate_v5.sql"
  - "src/store/publish.rs"
  - "src/store/publish/sync.rs"
  - "src/store/recovery.rs"
  - "src/store/recovery/service.rs"
  - "src/store/recovery/service/write.rs"
  - "src/store/status.rs"
  - "src/store/status/latest.rs"
  - "src/store/status/queue.rs"
  - "src/tui.rs"
  - "src/tui/confirm.rs"
  - "src/tui/draw.rs"
  - "src/tui/draw/draft.rs"
  - "src/tui/edit.rs"
  - "src/tui/keys.rs"
  - "src/tui/model.rs"
  - "src/tui/open.rs"
  - "src/tui/post.rs"
  - "src/tui/runner.rs"
  - "src/tui/tests/publish.rs"
  - "src/tui/tests/queue.rs"
  - "src/watch/run.rs"
  - "tests/status.rs"
  - "tests/tui.rs"
covered_digest: "v4:sha256:8fd5cafb44536513a3f7d01b2667eb000c1ea40b2eabd9d58f438972a75014f3"
re_verification:
  previous_status: gaps_found
  previous_score: 25/28
  gaps_closed:
    - "While tui holds the lock, serve runs a due sync with execute_sync and then later reconcile and publish jobs, without a second create_review."
    - "y runs reconcile when a sync job is already the oldest due service job, and a missing marker stays unknown."
    - "Queue and diffmate status join the latest review job, so a later publish or sync does not replace succeeded (findings: N)."
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Accept or reject the judgment that the tui sync arm never calls create_review, and that a failed sync retries the same job while the publication stays submitted."
    expected: "serve matches sync, calls execute_sync, and loops. A failed read requeues that job with next_attempt_at and does not POST. Tests y_runs_reconcile_after_a_due_sync and serve_runs_a_due_sync_then_a_newer_publish already passed (posts 0 on the y path, posts 1 only for the later publish)."
    why_human: "unverified-prohibition — human review recommended. Plan 04-09 marks this MUST NOT as verification judgment and status unresolved. The automated verdict is non-authoritative. It holds in the code and in the two tests, but a judgment-tier prohibition is not a silent pass."
  - test: "Accept or reject the judgment that a missing publication marker does not move unknown to rejected."
    expected: "execute_reconcile on Ok(None) calls write::succeed and leaves the publication unknown. y_missing_marker_stays_unknown and reconcile_missing_marker_stays_unknown passed with state unknown, reconcile succeeded, and posts 0."
    why_human: "unverified-prohibition — human review recommended. Plan 04-09 marks this MUST NOT as verification judgment and status unresolved. Non-authoritative verdict: it holds. SPEC says a missing marker never moves unknown to rejected."
  - test: "Accept or reject the judgment that a later publish, reconcile, or sync job is not the latest review run on the queue or on diffmate status."
    expected: "The shared jobs subquery is max(id) where kind = 'review'. later_publish_and_sync_keep_the_review_run and queue_frame_keeps_the_review_run_after_sync passed with succeeded (findings: 1) after a later publish and sync. Publication, freshness, and newest-target order were unchanged."
    why_human: "unverified-prohibition — human review recommended. Plan 04-10 marks this MUST NOT as verification judgment and status unresolved. Non-authoritative verdict: it holds. The queue test does not insert a reconcile row; reconcile is excluded by the same kind predicate."
---

# Phase 4: Review and publish Verification Report

**Phase Goal:** I can read a draft in `tui`, decide on every finding, edit what I keep, and publish exactly one GitHub review under my name, or learn precisely why not, with no duplicate review after any crash, double key press, or timeout.
**Verified:** 2026-11-11T09:05:23Z
**Status:** human_needed
**Re-verification:** Yes — after gap closure

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | ------- | ---------- | -------------- |
| 1 | Publishing creates one GitHub review on the reviewed commit, with kept and edited findings as inline comments carrying finding markers, and the publication marker in the body. | ✓ VERIFIED | `execute_publish` sets `commit_id` from the run head after `mark_sending`. `body.rs` appends `<!-- diffmate:finding=... -->` and ends the body with `<!-- diffmate:publication=... -->`. Unchanged since the prior pass. `comment_key_posts_one_review` passed again (1 POST). |
| 2 | Publishing with undecided findings is refused until every finding has a decision. | ✓ VERIFIED | `insert::write` returns `Outcome::Undecided` and does not commit a job. `tui/post.rs` renders `undecided findings:`. Zero findings remain decidable. File unchanged. |
| 3 | Pressing a publish key twice creates one GitHub review, and publishing an already submitted draft is refused. | ✓ VERIFIED | Blocking states and `DraftSubmitted` are still returned from `confirm_publication`. Tests `second_c_while_pending_posts_once` and `submitted_draft_does_not_post_again` are still present. Those files were not modified after the prior verification. |
| 4 | Publishing a stale draft asks for confirmation first, and asks again if the head moves before the runner sends it. | ✓ VERIFIED | `Confirm` still highlights `Don't send`. `gate::before_send` calls `mark_needs_confirmation` when the fetched head differs, before `mark_sending`. Unchanged. |
| 5 | With an existing pending review of mine on the PR, publish stops with a clear message and changes nothing on GitHub. | ✓ VERIFIED | `pending_review_of_mine` still requires exact login and exact state `PENDING`. `stop` abandons and returns before `create_review`. Test `pending_review_of_mine_posts_nothing` still present. Unchanged. |
| 6 | Publishing while `watch` is in the middle of an agent run completes without waiting for that run. | ✓ VERIFIED | `watch/run.rs` `alongside` still polls `WATCH_LOCK_POLL` and calls `serve` while the review future is pending. `serve` still runs publish. File unchanged. Test `publish_finishes_during_an_agent_run` still present. |
| 7 | A crash before `sending` is committed resumes safely after restart and publishes exactly once. | ✓ VERIFIED | `crash_point!("publish-before-sending")` is still before `mark_sending`. Test `crash_before_sending_posts_once` still present. `post.rs` unchanged. |
| 8 | A crash after `sending` is committed leaves the publication `unknown`, with no automatic second POST. | ✓ VERIFIED | `recovery::resolve_running` calls `service::settle`. `sending` becomes `unknown` with reason `outcome unknown` and no `gh` call. Test `crash_after_sending_is_unknown` still present. Recovery files unchanged. |
| 9 | Declining a `needs_confirmation` publication cancels the job and abandons the publication; redelivering the old job does not revive it. | ✓ VERIFIED | `confirm::decline` still sets the job `cancelled`. `gate::before_send` returns before `create_review` for an abandoned publication. Test `decline_cancels_and_abandons` still present. Unchanged. |
| 10 | While `tui` holds the lock it shows that it acts as the runner and does not run `review` jobs. | ✓ VERIFIED | `draw.rs` pushes `runner: tui` only while `session.running()`. `next_service` selects only `publish`, `reconcile`, and `sync`. No `execute_review_job` under `src/tui`. `comment_key_posts_one_review` passed again and still expects `runner: tui`. |
| 11 | A simulated timeout on the GitHub call leaves the publication `unknown`, and a new attempt stays blocked until reconciliation finds the marker or the owner abandons it. A missing marker stays `unknown`. | ✓ VERIFIED | `outcome::finish` maps timeout and server errors to `unknown`, and a non-transient client error to `rejected`. `timeout_is_unknown` and `server_error_is_unknown` passed this pass. `execute_reconcile` on no marker calls `write::succeed` and does not update the publication. `reconcile_missing_marker_stays_unknown` and `y_missing_marker_stays_unknown` passed: state `unknown`, not `rejected`, posts 0. |
| 12 | A failed comment sync leaves the publication `submitted`, inline thread statuses `unknown`, and is retried without a new POST, and while `tui` holds the lock it serves that sync job. | ✓ VERIFIED | `execute_sync` on a failed comment read calls `later`, which requeues the same job 60s out and returns `Ok`. `sync_failure_stays_submitted` passed (submitted, `unknown`, no comment id, posts 0, gap 60000). `tui/runner.rs` now matches `sync`, calls `execute_sync`, and loops. `y_runs_reconcile_after_a_due_sync` passed: sync `next_attempt_at` set, publication still `submitted`, posts 0. |
| 13 | `diffmate tui` opens a queue of every target, newest target id first, and `q` leaves without writing a publication or a job. | ✓ VERIFIED | `queue_rows` uses `ORDER BY t.id DESC`. `queue_is_newest_first_and_arrows_move_the_selection` and `queue_shows_the_target_and_quit_writes_nothing` passed this pass. |
| 14 | Each queue row shows the label, tracking, the latest review run, draft freshness from the stored head and `head_checked_at`, and the latest publication state. A later publish, reconcile, or sync job does not replace that review run. | ✓ VERIFIED | Jobs join is `max(id) ... AND kind = 'review'`. `latest()` still formats `succeeded (findings: N)`. `draw.rs` prints `row.latest`, freshness, and publication. `later_publish_and_sync_keep_the_review_run` and `queue_frame_keeps_the_review_run_after_sync` passed. Reconcile is excluded by the same predicate. |
| 15 | Enter opens that target's draft and Esc returns to the queue. The list is findings only, anchored before unanchored, ordinal inside each group. Limitation and future improvements are not decision rows. | ✓ VERIFIED | `draw/draft.rs` `screen` is called from `draw.rs`. Test `enter_opens_the_draft_and_esc_restores_the_queue` still present. Draft files unchanged. The plan locator `fn draft` is the function `screen`. |
| 16 | `diffmate status` stays read-only, keeps `ORDER BY t.id`, and appends the publication state and the same latest review run, including `succeeded (findings: N)` after a publish. A non-terminal stdout makes `tui` exit 1 before creating a database. | ✓ VERIFIED | `target_statuses` orders by `t.id` and uses the same SQL. `status.rs` appends `target.publication` and `latest()`. `later_publish_and_sync_keep_the_review_run` passed for `target_statuses`. `tui::run` returns on `!stdout().is_terminal()` before `Store::open`. `status_does_not_write` and `tui_without_a_terminal_exits_before_creating_a_database` still present. |
| 17 | `k` sets the selected finding to kept, `d` sets it to dropped, and `K` sets only the remaining undecided findings to kept. | ✓ VERIFIED | `set_decision` and `keep_undecided` (`decision = 'undecided'` only) in `decisions.rs`. Tests still present. File unchanged. |
| 18 | `e` sets edited only when the editor exits 0 and the saved text differs. A cancelled editor, an unchanged save, an empty save, or an unset `EDITOR` leaves the decision. | ✓ VERIFIED | `tui/edit.rs` `run` is the editor seam (plan locator `fn edit_finding` is this function). Test `editor_saves_only_a_real_change` still present. Unchanged. |
| 19 | The agent's `original_output` is not updated. `i` toggles `limitation_included` and does not open an editor. A previous finding's decision is a hint and is not copied into `decision`. | ✓ VERIFIED | `decisions.rs` updates `findings` title, body, and decision, `future_improvements`, and `limitation_included`. It does not mention `original_output`. The key-link pattern miss is the absence. Tests still assert the stored original. |
| 20 | Two findings with the same path and line stay two rows. Anchored rows stay before unanchored rows. | ✓ VERIFIED | Snapshot and draft queries still order by anchor then ordinal and do not collapse ids. Unchanged. |
| 21 | A draft with no findings still shows its limitation and future improvements. `k`, `d`, and `e` do not invent a finding. | ✓ VERIFIED | `i_includes_the_limitation_without_starting_an_editor` and `empty_draft_body_is_only_the_marker` still present. Unchanged. |
| 22 | The confirm transaction creates a pending publication and a queued publish job, or neither. `sending` is committed before `create_review`. | ✓ VERIFIED | `confirm_publication` still commits only `Outcome::Job`. `post.rs` calls `mark_sending` before `outcome::finish`. The crash remote still asserts the publication is `sending` when `create_review` runs. `comment_key_posts_one_review` passed. |
| 23 | Finding order in the body and in a refusal is anchored findings, then unanchored findings, ordinal ascending inside each group. | ✓ VERIFIED | Body and undecided-id ordering unchanged. |
| 24 | `o` opens the PR URL as one argument. `n` enqueues a manual review, does not change tracking, and explains that `review` or `watch` must run it when watch does not hold the lock. `x` asks first and the highlighted choice keeps the target. | ✓ VERIFIED | `open_pr`, `enqueue_review`, and `Confirm` with `highlighted_safe: true` still wired. Tests `o_spawns_open_with_only_the_pr_url`, `n_without_watch_enqueues_a_manual_review_and_explains`, and `enter_on_keep_leaves_tracking_and_request_seen` still present. Unchanged. |
| 25 | `u`'s highlighted choice leaves a queued publication pending. The other choice cancels it. `q` does not cancel it. | ✓ VERIFIED | `Confirm::cancel_queued` still highlights `Keep queued`. Tests `keep_queued_is_highlighted_and_enter_writes_nothing`, `the_other_choice_cancels_the_queued_publication`, and `quit_leaves_a_queued_publication_pending` still present. Unchanged. |
| 26 | `y` enqueues and runs reconcile while `tui` holds the lock, including when a sync job is already the oldest due service job. | ✓ VERIFIED | `post::reconcile` calls `enqueue_reconcile` then `runner::drive`. `serve` no longer returns on `sync`. `y_runs_reconcile_after_a_due_sync` passed: reconcile `succeeded`, sync requeued with `next_attempt_at`, posts 0, unknown publication not `rejected`. |
| 27 | After that sync is served, a newer queued publish still runs, and the sync itself does not call `create_review`. | ✓ VERIFIED | `serve` loops after `execute_sync`. `execute_sync` only reads comments and threads. `serve_runs_a_due_sync_then_a_newer_publish` passed: posts 1, sync job queued with `next_attempt_at`, synced publication still `submitted` with the same review id, newer publish job `succeeded`. |
| 28 | `b`'s highlighted choice leaves an `unknown` publication unknown. Accepting abandon sets reason `confirmed by me` and does not POST. | ✓ VERIFIED | `abandon_unknown` updates only `state = 'unknown'`. Tests `b_safe_choice_leaves_unknown`, `b_abandon_sets_confirmed_by_me`, and `abandon_unknown_sets_confirmed_by_me` still present. `abandon_unknown_sets_confirmed_by_me` is also in `marker.rs` and asserts posts 0. Unchanged. |
| 29 | `previous.json` gets thread status from a fresh GraphQL read. A failed read writes `unknown` and does not fail the review. Body findings stay `none`. | ✓ VERIFIED | `Gh::review_comment_threads` calls `threads::comments`. `context/previous.rs` still maps a failed read to `unknown` and body findings to `none`. Test `linked_thread_is_resolved_and_a_body_finding_stays_none` still present. Unchanged. The plan locator `fn review_comment_threads` lives on the `Gh` trait in `src/gh.rs`. |

**Score:** 29/29 truths verified (0 present, behavior-unverified)

### Advisory (New Scope, Unevidenced)

New-scope findings from Step 7 with no deterministic evidence — reported, not blocking.

None. The only files changed since the prior verification (`2026-11-11T01:03:37Z`) are `src/tui/runner.rs`, `src/store/status.rs`, `src/publish/tests/crash/keys.rs`, `src/publish/tests/crash/gh.rs`, `src/status/tests/stored.rs`, and `src/tui/tests/queue.rs`. No debt markers. The `gh` fake change only selects the `sending` publication when more than one row exists.

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `src/store/migrate_v5.sql` | publications table and decision columns | ✓ VERIFIED | `CREATE TABLE publications` and `thread_status`. `SCHEMA_VERSION` is 5. No `migrate_v6.sql`. |
| `src/tui.rs` | `tui` entry | ✓ VERIFIED | `run` refuses a non-terminal, then draws. `main.rs` calls `tui::run`. |
| `src/tui/draw/draft.rs` | draft list and text pane | ✓ VERIFIED | Locator `fn draft` is absent. The screen function is `screen`, called from `draw.rs`. Substantive. |
| `src/tui/edit.rs` | editor seam | ✓ VERIFIED | Locator `fn edit_finding` is absent. The function is `run`, called from the model. |
| `src/store/decisions.rs` | decision writes | ✓ VERIFIED | Updates findings and the limitation flag, never `original_output`. |
| `src/store/publish.rs` | confirm and publication states | ✓ VERIFIED | `confirm_publication` delegates to the insert transaction. `enqueue_sync` inserts a queued sync job with no `next_attempt_at`. |
| `src/publish/post.rs` | sending, then one create | ✓ VERIFIED | `mark_sending` then `outcome::finish`. |
| `src/gh/reviews.rs` | `create_review` | ✓ VERIFIED | Used by `GhCli::create_review`. |
| `src/tui/confirm.rs` | safe-default modal | ✓ VERIFIED | Constructors set `highlighted_safe: true`. |
| `src/tui/open.rs` | browser open | ✓ VERIFIED | `open_pr` spawns one program and the URL. |
| `src/tui/runner.rs` | tui service loop, including sync | ✓ VERIFIED | Publish, reconcile, and sync each call their executor, then the loop continues. `_` returns only for a kind `next_service` does not select. |
| `src/publish/checks.rs` | stale head, pending review, and terminal 401/403 | ✓ VERIFIED | `transient_gh` retries 408, 429, and 500..=599 only. 401 and 403 are not transient. |
| `src/publish/reconcile/mod.rs` | reconcile without a POST | ✓ VERIFIED | Declared path `src/publish/reconcile.rs` is this module. A missing marker does not set `rejected`. |
| `src/publish/sync.rs` | comment sync | ✓ VERIFIED | Failure requeues the same job. No `create_review` in the sync module. Wired from `tui/runner.rs` and `watch/run.rs`. |
| `src/gh/threads.rs` | GraphQL thread reads | ✓ VERIFIED | Locator `fn review_comment_threads` is `comments` here, called from the `Gh` trait. |
| `src/watch/run.rs` | publish during an agent run | ✓ VERIFIED | `alongside` serves publish, reconcile, and sync. |
| `src/context/previous.rs` | `previous.json` thread status | ✓ VERIFIED | Called with the `Gh` handle. |
| `src/store/recovery/service.rs` | sending becomes unknown | ✓ VERIFIED | Not in `resolve.rs`. `resolve_running` calls `service::settle`. |
| `src/store/status.rs` | latest review job | ✓ VERIFIED | `kind = 'review'` on the shared subquery. Queue and status both use it. |
| `src/publish/tests/crash/keys.rs` | due sync does not block y or a newer publish | ✓ VERIFIED | Contains `y_runs_reconcile_after_a_due_sync` and `serve_runs_a_due_sync_then_a_newer_publish`. Both passed. |
| `src/status/tests/stored.rs` | later publish and sync keep the review run | ✓ VERIFIED | `later_publish_and_sync_keep_the_review_run` passed. |
| `src/tui/tests/queue.rs` | drawn queue keeps the review run | ✓ VERIFIED | `queue_frame_keeps_the_review_run_after_sync` passed. |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `src/main.rs` | `src/tui.rs` | `Command::Tui` calls `tui::run` | WIRED | `main.rs` line 71. |
| `src/tui/model.rs` | `src/store/status.rs` | `queue_rows` ordered by target id descending | WIRED | `queue.rs` appends `ORDER BY t.id DESC`. |
| `src/tui/keys.rs` | `src/store/decisions.rs` | k, d, K, i, and a saved edit | WIRED | Model calls `set_decision`, `keep_undecided`, `save_finding_text`, `toggle_limitation`. |
| `src/tui/edit.rs` | `drafts.original_output` | edits must not write it | WIRED | The pattern is absent, which is the link. `decisions.rs` has no `original_output` write. |
| `src/publish/post.rs` | `src/gh/reviews.rs` | `create_review` only after `sending` | WIRED | `mark_sending` then `outcome::finish`. |
| `src/store/publish.rs` | `publications.snapshot` | snapshot frozen at confirm | WIRED | Confirm transaction. |
| `src/tui/keys.rs` | `src/store/poll.rs` | discard only after the dangerous choice | WIRED | Pattern `discard` found. |
| `src/tui/keys.rs` | `src/store/jobs/attach.rs` | `n` calls `enqueue_review` | WIRED | No `execute_review_job` under `src/tui`. |
| `src/tui/keys.rs` | `src/store/publish.rs` | a, r, c call `confirm_publication` | WIRED | `post::send` checks first. |
| `src/tui/runner.rs` | `src/publish/post.rs` | tui runs publish and never a review | WIRED | Publish arm calls `execute_publish`. |
| `src/tui/runner.rs` | `src/publish/sync.rs` | tui serves every due sync job, then loops | WIRED | Sync arm calls `execute_sync`. Previously NOT_WIRED. |
| `src/tui/post.rs` | `src/tui/runner.rs` | y enqueues reconcile then drive | WIRED | `enqueue_reconcile` then `runner::drive`. A due sync no longer returns early. |
| `src/publish/post.rs` | `src/gh/reviews.rs` | `list_my_reviews` before sending | WIRED | `gate::before_send`. |
| `src/tui/confirm.rs` | `src/store/publish.rs` | decline does not requeue | WIRED | `fn decline` cancels. |
| `src/store/recovery.rs` | `publications.state` | sending becomes unknown with no gh call | WIRED | The plan pointed at `resolve.rs`. `resolve_running` calls `service::settle`, which runs `mark_unknown`. |
| `src/publish/reconcile/mod.rs` | `src/gh/reviews.rs` | list reviews, no `create_review` | WIRED | `review_id` uses `list_my_reviews` only. The declared path `reconcile.rs` is the module directory. |
| `src/watch/run.rs` | `src/publish/post.rs` | service job during an agent run | WIRED | `WATCH_LOCK_POLL` branch. |
| `src/context/previous.rs` | `src/gh/threads.rs` | fresh `thread_resolved` | WIRED | Pattern found. |
| `src/store/status.rs` | `src/store/status/latest.rs` | review job run is what `latest()` formats | WIRED | `kind = 'review'` in the join. `latest()` unchanged. |
| `src/store/status/queue.rs` | `src/tui/draw.rs` | frame prints `row.latest` | WIRED | `queue_rows` uses the shared SQL. `row_text` prints `row.latest`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `src/store/status.rs` | `latest` | Latest `kind = 'review'` job and its run, then `latest()` | Yes | ✓ FLOWING |
| `src/store/status.rs` | `publication` | `publications.state` for the target, newest `created_at` | Yes | ✓ FLOWING |
| `src/store/status/queue.rs` | `freshness` | Stored draft head and `head_checked_at` | Yes | ✓ FLOWING |
| `src/tui/draw.rs` | queue line | `QueueRow` from `queue_rows` | Yes | ✓ FLOWING |
| `src/status.rs` | status line | `target_statuses` | Yes | ✓ FLOWING |
| `src/publish/post.rs` | review body and comments | Publication snapshot frozen at confirm | Yes | ✓ FLOWING |
| `src/publish/sync.rs` | comment links | `review_comment_threads`, or a requeue when that read fails | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

Command: `mise exec -- cargo nextest run --locked --features crash-points -E 'test(/y_runs_reconcile_after_a_due_sync|serve_runs_a_due_sync_then_a_newer_publish|y_missing_marker_stays_unknown|reconcile_missing_marker_stays_unknown|later_publish_and_sync_keep_the_review_run|queue_frame_keeps_the_review_run_after_sync|queue_shows_the_target_and_quit_writes_nothing|queue_is_newest_first_and_arrows_move_the_selection|a_queued_crash_retry_appends_its_persisted_budget_to_latest|auth_status_abandons_without_sleep|http_status_classifies_retries|client_error_rejects|server_error_is_unknown|sync_failure_stays_submitted|comment_key_posts_one_review|timeout_is_unknown/)'`

Result: 16 passed, 0 failed, 490 skipped. Run ID `3cfe4e3f-812b-4ff7-af56-c915f9de5bb8`.

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Due sync then reconcile, no POST, marker missing stays unknown | `y_runs_reconcile_after_a_due_sync` | pass | ✓ PASS |
| Due sync then a newer publish, one POST | `serve_runs_a_due_sync_then_a_newer_publish` | pass | ✓ PASS |
| y with no marker stays unknown | `y_missing_marker_stays_unknown` | pass | ✓ PASS |
| reconcile with no marker stays unknown | `reconcile_missing_marker_stays_unknown` | pass | ✓ PASS |
| Later publish and sync keep `succeeded (findings: 1)` | `later_publish_and_sync_keep_the_review_run` | pass | ✓ PASS |
| Drawn queue keeps the review run and newest-first order | `queue_frame_keeps_the_review_run_after_sync` | pass | ✓ PASS |
| Queue quit writes nothing | `queue_shows_the_target_and_quit_writes_nothing` | pass | ✓ PASS |
| 401 and 403 on pre-checks abandon with no sleep | `auth_status_abandons_without_sleep` | pass | ✓ PASS |
| 401 and 403 are not transient | `http_status_classifies_retries` | pass | ✓ PASS |
| Non-transient `create_review` rejects once | `client_error_rejects` (422, same arm as 401/403) | pass | ✓ PASS |
| Server error is unknown, one POST | `server_error_is_unknown` | pass | ✓ PASS |
| Failed sync stays submitted and retries | `sync_failure_stays_submitted` | pass | ✓ PASS |
| Comment key posts one review and shows `runner: tui` | `comment_key_posts_one_review` | pass | ✓ PASS |
| Timeout is unknown and blocks another confirm | `timeout_is_unknown` | pass | ✓ PASS |
| Review retry text still follows the review job | `a_queued_crash_retry_appends_its_persisted_budget_to_latest` | pass | ✓ PASS |

`SCHEMA_VERSION` is 5. `user_version` is written only as that constant. HTTP 401 and 403 stay terminal: `transient_gh` is false, pre-checks abandon on the first failure, and `create_review` takes the non-transient `rejected` arm (one POST, not `unknown`, not retried).

### Probe Execution

No `scripts/*/tests/probe-*.sh` and no probe path declared in the phase plans. Not applicable.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| PUB-01 | 04-01, 04-10 | Queue and draft views, latest review run, freshness, publication | ✓ SATISFIED | Truths 13-16. Review-job join and both new tests passed. |
| PUB-02 | 04-02 | Decisions, edit, limitation, keep-rest | ✓ SATISFIED | Truths 17-21. `decisions.rs` unchanged and does not copy a previous decision. |
| PUB-03 | 04-04, 04-07, 04-09 | Open, re-run, discard, reconcile, abandon | ✓ SATISFIED | Truths 24, 26, 28. `y` runs after a due sync. Missing marker stays unknown. |
| PUB-04 | 04-03, 04-05 | One review, markers, kept and edited findings | ✓ SATISFIED | Truths 1 and 22. |
| PUB-05 | 04-03, 04-05 | Undecided findings refuse publish | ✓ SATISFIED | Truth 2. |
| PUB-06 | 04-03, 04-05 | Double key posts once; submitted draft refused | ✓ SATISFIED | Truth 3. |
| PUB-07 | 04-06 | Stale draft confirmation, and again if the head moves | ✓ SATISFIED | Truth 4. |
| PUB-08 | 04-06 | Pending review of mine stops the publish | ✓ SATISFIED | Truth 5. |
| PUB-09 | 04-08 | Publish during an agent run | ✓ SATISFIED | Truth 6. |
| PUB-10 | 04-07 | Crash before sending publishes once | ✓ SATISFIED | Truth 7. |
| PUB-11 | 04-07 | Crash after sending is unknown, no second POST | ✓ SATISFIED | Truth 8. |
| PUB-12 | 04-06 | Decline cancels and does not revive | ✓ SATISFIED | Truth 9. |
| PUB-13 | 04-05, 04-08 | tui is the runner and does not run review jobs | ✓ SATISFIED | Truth 10. `comment_key_posts_one_review` passed after the runner change. |
| PUB-14 | 04-07 | Timeout is unknown until reconcile or abandon | ✓ SATISFIED | Truth 11. `timeout_is_unknown` passed. |
| PUB-15 | 04-08, 04-09 | Failed sync stays submitted and retries without a POST, including under tui | ✓ SATISFIED | Truths 12 and 27. |

Every phase-4 requirement ID in REQUIREMENTS.md is claimed by at least one plan. No orphan IDs. No plan claims an ID outside PUB-01 through PUB-15.

### Test-tier prohibitions

These are `verification: test` and `status: resolved` on plans 04-02 through 04-08. Each has a named test. The implementing files were not modified after the prior verification, except where a spot-check above re-ran the behavior.

| Prohibition | Enforcement |
| ----------- | ----------- |
| MUST NOT copy a previous decision into the current decision | `k_keeps_one_finding_and_shift_k_keeps_only_the_rest` |
| MUST NOT put a dropped finding, or an excluded limitation, into the review | `body_omits_dropped_and_excluded_limitation` |
| MUST NOT highlight the choice that discards a target or sends a review | `Confirm` sets `highlighted_safe: true`; `enter_on_keep_leaves_tracking_and_request_seen`, `stale` highlight tests |
| MUST NOT create or change an existing pending review of mine | `pending_review_of_mine_posts_nothing` |
| MUST NOT call `create_review` again for sending, unknown, submitted, or abandoned | `crash_after_sending_is_unknown`; reconcile lists reviews only |
| MUST NOT POST from comment sync or a thread-status read | `sync_failure_stays_submitted` passed this pass |

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
|-----------|-----------|--------|---------|----------|-----------------|---------|
| `src/publish/tests/crash/keys.rs` | PUB-15, PUB-03 | yes | no `#[ignore]` | no | Behavioral: states, `next_attempt_at`, post count | Sufficient |
| `src/status/tests/stored.rs` | PUB-01 | yes | no | no | Value: `succeeded (findings: 1)` and unchanged columns | Sufficient |
| `src/tui/tests/queue.rs` | PUB-01 | yes | no | no | Behavioral frame text and job count | Sufficient |
| `src/publish/tests/confirm/budget.rs` | PUB-07, 401/403 | yes | no | no | Value: elapsed zero, one call, abandoned | Sufficient |
| `src/publish/tests/crash/outcome.rs` | PUB-14 | yes | no | no | Behavioral: unknown, blocked confirm, one POST | Sufficient |
| `src/publish/tests/sync.rs` | PUB-15 | yes | no | no | Behavioral: submitted, unknown, 60s, posts 0 | Sufficient |

**Disabled tests on requirements:** 0
**Circular patterns detected:** 0
**Insufficient assertions:** 0

The `thread_status = unknown` assertion in the tui sync tests matches the column default for an inline finding. It does not by itself prove a write. `next_attempt_at IS NOT NULL` does prove `later()` ran, because `enqueue_sync` does not set that column. The post count proves sync did not call `create_review`.

`create_review` with HTTP 401 is not its own test. `http_status_classifies_retries` asserts 401 and 403 are not transient, and `client_error_rejects` exercises the same `Ok(Err(_)) => rejected` arm with 422. Pre-check 401 and 403 are tested directly.

### Decision Coverage

All trackable CONTEXT.md decisions are honored by shipped artifacts. 11 honored, 11 total, none unhonored. Non-blocking.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No `TBD`, `FIXME`, or `XXX` in the files changed since the prior verification | — | None |

`serve`'s `_ => return Ok(())` is not a stub. `next_service` only selects `publish`, `reconcile`, and `sync`.

### Human Verification Required

**unverified-prohibition — human review recommended.** Three judgment-tier prohibitions on plans 04-09 and 04-10 are `status: unresolved`. Automated checks passed. The verdicts below are non-authoritative. They must not be treated as a silent pass.

### 1. Sync arm does not post

**Test:** Accept or reject the judgment that the tui sync arm never calls `create_review`, and that a failed sync retries the same job while the publication stays submitted.
**Expected:** `serve` calls `execute_sync` and continues. A failed read requeues that job. Tests `y_runs_reconcile_after_a_due_sync` and `serve_runs_a_due_sync_then_a_newer_publish` passed.
**Why human:** Plan 04-09 marks the prohibition `verification: judgment`. A judgment-tier item stays flagged even when a test passed.

### 2. Missing marker stays unknown

**Test:** Accept or reject the judgment that a missing publication marker does not move `unknown` to `rejected`.
**Expected:** Reconcile succeeds and the publication stays `unknown`, with no POST. `y_missing_marker_stays_unknown` and `reconcile_missing_marker_stays_unknown` passed.
**Why human:** Plan 04-09 marks this prohibition `verification: judgment` and unresolved.

### 3. Latest run is the review job

**Test:** Accept or reject the judgment that a later publish, reconcile, or sync job is not shown as the latest review run.
**Expected:** Queue and `diffmate status` still show `succeeded (findings: N)` from the latest review job. The two PUB-01 tests passed for publish and sync. Reconcile is excluded by `kind = 'review'`.
**Why human:** Plan 04-10 marks this prohibition `verification: judgment` and unresolved.

### Gaps Summary

The three gaps from the prior report are closed in the current tree. `tui` serves a due sync, then a later reconcile or publish, without a second `create_review`. A missing marker stays `unknown`. Queue and status join `kind = 'review'`. Schema `user_version` stays 5. HTTP 401 and 403 stay terminal.

No truth, artifact, or key link failed. Status is `human_needed` only because the three judgment-tier prohibitions above are unresolved in the plans and cannot be a silent pass.

---

_Verified: 2026-11-11T09:05:23Z_
_Verifier: Claude (gsd-verifier)_
