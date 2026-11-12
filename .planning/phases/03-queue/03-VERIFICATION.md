---
phase: 03-queue
verified: 2026-11-12T10:55:42Z
status: human_needed
score: 16/16 must-haves verified
covered_files: [".planning/phases/03-queue/03-01-PLAN.md", ".planning/phases/03-queue/03-01-SUMMARY.md", ".planning/phases/03-queue/03-02-PLAN.md", ".planning/phases/03-queue/03-02-SUMMARY.md", ".planning/phases/03-queue/03-03-PLAN.md", ".planning/phases/03-queue/03-03-SUMMARY.md", ".planning/phases/03-queue/03-04-PLAN.md", ".planning/phases/03-queue/03-04-SUMMARY.md", ".planning/phases/03-queue/03-05-PLAN.md", ".planning/phases/03-queue/03-05-SUMMARY.md", ".planning/phases/03-queue/03-06-PLAN.md", ".planning/phases/03-queue/03-06-SUMMARY.md", ".planning/phases/03-queue/03-07-PLAN.md", ".planning/phases/03-queue/03-07-SUMMARY.md", ".planning/phases/03-queue/03-08-PLAN.md", ".planning/phases/03-queue/03-08-SUMMARY.md", "src/cli.rs", "src/config.rs", "src/debug.rs", "src/gh.rs", "src/gh/meta.rs", "src/gh/search.rs", "src/gh/search/parse.rs", "src/gh/search/tests/args.rs", "src/gh/search/tests/malformed.rs", "src/gh/search/tests/pages.rs", "src/main.rs", "src/notify.rs", "src/notify/recovery.rs", "src/notify/script.rs", "src/notify/tests.rs", "src/recovery/rules.rs", "src/recovery/rules/tests/transient.rs", "src/recovery/rules/transient.rs", "src/render.rs", "src/review.rs", "src/review/print.rs", "src/runner/outcome/failed.rs", "src/runner/run.rs", "src/store/migrate.rs", "src/store/migrate/backup.rs", "src/store/migrate/tests.rs", "src/store/migrate/tests/backups.rs", "src/store/migrate/tests/v3.rs", "src/store/migrate_v3.sql", "src/store/migrate_v4.sql", "src/store/poll.rs", "src/store/poll/schedule.rs", "src/store/poll/schedule/rows.rs", "src/store/poll/schedule/write.rs", "src/store/poll/tests/discard.rs", "src/store/poll/tests/enqueue.rs", "src/store/poll/tests/schedule.rs", "src/store/poll/tests/seen.rs", "src/store/poll/tracking.rs", "src/store/retry/queue.rs", "src/store/schema.sql", "src/watch.rs", "src/watch/job.rs", "src/watch/poll.rs", "src/watch/run.rs", "src/watch/timing.rs", "src/watch/wait.rs", "tests/watch/attach.rs", "tests/watch/discard.rs", "tests/watch/notify/give_up.rs", "tests/watch/notify/ready.rs", "tests/watch/notify/recovery.rs", "tests/watch/poll/absent.rs", "tests/watch/poll/debounce.rs", "tests/watch/poll/draft.rs", "tests/watch/poll/follow.rs", "tests/watch/poll/manual.rs", "tests/watch/poll/ready.rs", "tests/watch/wait.rs"]
covered_digest: "v4:sha256:6e8caf7ed2f69eb9aa304716392b8329f910f187bb65c99caa4e2836730a8a30"
behavior_unverified: 0
overrides_applied: 0
decision_coverage:
  honored: 9
  total: 9
  not_honored: []
human_verification:
  - test: "With diffmate watch running, request your review directly on a real GitHub pull request and do nothing else."
    expected: "The PR becomes a tracked target, one automatic review finishes as a ready draft, and Notification Center shows a notification titled diffmate: draft ready whose body is the PR label."
    why_human: "Process tests drive a fake gh and a fake osascript that records argv. They do not call GitHub and they do not display a macOS notification."
  - test: "Let an automatic review exhaust its retry budget (or fail terminally) while watch is running. Separately, stop watch with SIGTERM during an automatic run."
    expected: "The give-up shows a notification titled diffmate: review failed whose body is the PR label plus the failure reason. The SIGTERM shutdown requeues the job and does not notify."
    why_human: "Tests assert the AppleScript text and that a shutdown writes no script. They do not show Notification Center."
---

# Phase 3: Queue Verification Report

**Phase Goal:** A PR where my review is directly requested gets a ready draft and a notification without any action from me, and a failed automatic review is never silent.
**Verified:** 2026-11-12T10:55:42Z
**Status:** human_needed
**Re-verification:** No — initial verification

This verifier did not re-run the suite and did not call live `gh` or the Claude API. The orchestrator reported `mise run --force test` as 509 passed and 11 skipped on this same tree. Named tests below were read and assert the transitions; that report is the pass evidence.

The phase goal holds in the current tree. Later phases moved schema, search, close, draft hold, and retry-head rules forward. Those moves match current SPEC.md and the locked later behavior. They are not phase 3 gaps and were not treated as work to revert. SPEC.md "Transitions" was not copied in as new phase 3 work.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | ------- | ---------- | -------------- |
| 1 | Requesting my review on a PR leads to a ready draft and a notification without any action from me. | ✓ VERIFIED | `watch` polls after the lock, `apply_poll` arms one automatic job, and success calls `notify::deliver`. `tests/watch/poll/ready.rs` `a_ready_request_becomes_one_automatic_run` and `tests/watch/notify/ready.rs` `a_ready_automatic_draft_runs_osascript` assert one automatic success and one script titled `diffmate: draft ready` with body `o/r#1`. A manual job records no script. |
| 2 | Two pushes within the debounce window produce one run. | ✓ VERIFIED | An unstarted automatic job with a new head is reset in place (`schedule/rows.rs` `consider` → `write::reset`), not duplicated. `tests/watch/poll/debounce.rs` `two_heads_before_the_job_starts_leave_one_later_run` asserts one review job and one succeeded run of the later head. A running job is not cancelled; `tests/watch/poll/follow.rs` leaves one follow-up after success. |
| 3 | A PR stays tracked after I submit a review and my request disappears. | ✓ VERIFIED | `tracking.rs` `miss` on `Absent::Open` sets `request_seen = 0` and does not clear `tracked`. `tests/watch/poll/absent.rs` `an_open_absence_keeps_one_delayed_job_for_the_new_head` asserts `tracking = tracked`, `request_seen = 0`, and one delayed automatic job. |
| 4 | A draft PR is not reviewed until it is marked ready. | ✓ VERIFIED | `schedule.rs` `arm` holds a draft instead of considering a head. `next_runnable_job` requires `held = 0`. `tests/watch/poll/draft.rs` `a_draft_is_tracked_without_a_job_until_it_is_ready` asserts a tracked draft with zero jobs, then one automatic run after `isDraft` becomes false. |
| 5 | A review request to my team does not queue the PR. | ✓ VERIFIED | Search argv is `user-review-requested:@me` with `--state=open` and no `--draft` or `--team` (`src/gh/search.rs` `search_args`). `src/gh/search/tests/args.rs` and the ready-request process test assert that argv. Team requests are never a search hit. |
| 6 | A PR I reviewed manually is not reviewed again on new pushes until my review is requested on it. | ✓ VERIFIED | `enqueue_review` does not change tracking or `request_seen` (`src/store/poll/tests/enqueue.rs`). `see` sets `tracked` only `WHERE request_seen = 0`. `tests/watch/poll/manual.rs` asserts new heads create no automatic job until a later search hit, which then tracks and arms one delayed job. |
| 7 | A discarded PR with a still visible request stays untracked; removing and re-adding the request tracks it again. | ✓ VERIFIED | `discard` sets `tracking = untracked`, `request_seen = 1`. A still-visible hit does not pass the `request_seen = 0` guard. Absence sets `request_seen = 0` and leaves it untracked; the next hit tracks and arms. `tests/watch/discard.rs` `a_still_visible_request_stays_untracked_until_it_returns` and `src/store/poll/tests/discard.rs` assert the sequence. |
| 8 | An agent failure is retried, and after the last retry a notification names the reason. | ✓ VERIFIED | `is_transient` accepts 408, 429, and 500–599, or a fixed token when status is absent. 401 and 403 stay terminal (`src/recovery/rules/tests/transient.rs`). `after_failure` uses the 60s then 300s budget and, on exhaustion, a fail reason that names retries used. `notify::decide` notifies only an automatic terminal result, with the message as the body. `tests/watch/notify/give_up.rs` asserts the script contains `diffmate: review failed`, the label, and the envelope reason. Shutdown requeues with `retry_count` unchanged and writes no script. |
| 9 | `review` while `watch` runs hands the job to `watch` and prints the findings when it ends. | ✓ VERIFIED | A finished wait calls `print::stored`, which renders the stored result through `render::review` (findings included). `tests/watch/attach.rs` `review_while_watch_runs_prints_the_result` routes search to `[]`, asserts watch runs the job, review stdout reports success, watch stderr has the job line and not the findings report, and the agent runs once. |
| 10 | Starting `watch` while `review` holds the lock makes `watch` wait, not exit. | ✓ VERIFIED | `watch::execute` acquires the lock before recovery and before `poll::once`. `tests/watch/wait.rs` `watch_waits_for_the_lock_and_stops_on_sigterm` asserts the waiting line, that the process is still running, that `gh search prs` has not been called, and that SIGTERM exits 0. |
| 11 | `config.toml` honors `poll_interval_seconds` and `debounce_seconds`, both defaulting to 60. | ✓ VERIFIED | `Settings::default` and `src/config/tests.rs` `empty_and_comment_only_inputs_use_spec_defaults`. A missing file returns defaults (`config::load`). `watch/run.rs` sleeps via `timing::idle_delay` (sooner of 1s, next poll, next attempt) and passes `settings.debounce()` into `poll::once`. The first poll is immediate (`next_poll_at` starts at 0) after the lock and recovery. |
| 12 | Opening user_version 0, 1, or 2 migrates forward through `request_seen` and `queued_head` to the current schema. Version 1 and 2 databases are backed up first. A fresh database writes neither backup. A newer-than-current version is refused unchanged. Tracking stays `untracked` or `tracked`. | ✓ VERIFIED | `SCHEMA_VERSION` is 5. Versions below 3 run `migrate_v3.sql` (`request_seen` 0/1 default 0, nullable `queued_head`). `schema.sql` does not add those columns. `backup::create` writes `diffmate.db.v{version}-backup`. Tests: `version_two_keeps_tracking_and_snapshots_that_version` (ends at 5, v2 backup, `request_seen = 0`, tracking kept, `head_sha` null), `backup_preserves_version_one_rows_and_is_not_rewritten`, `fresh_database_has_no_backup`, `newer_version_is_refused_and_left_alone` (version 6 byte-for-byte). |
| 13 | A failed search does not change tracking state. | ✓ VERIFIED | `poll::once` returns before `apply_poll` when search fails. `tests/watch/poll/ready.rs` `a_failed_search_does_not_change_request_seen` asserts one `poll failed:` line, unchanged `request_seen`, and no `gh pr view`. |
| 14 | `diffmate debug discard URL` is hidden, calls only `Store::discard`, prints `discarded OWNER/REPO#N`, and exits 0. An invalid URL exits 2 with the enqueue error and does not open the store. It does not take the lock, require a valid config, or call gh. | ✓ VERIFIED | `Command::Debug` is `hide = true`. `main` loads `config.toml` only for review, watch, and tui. `debug::discard` parses, opens the store, and calls `discard`. `tests/watch/discard.rs` covers stdout, exit 2, a broken config, a held lock, no gh calls, and hidden help. |
| 15 | Shutdown requeues an automatic job without spending the budget and does not notify. Recovery of an automatic give-up does notify, and names the reason. A manual crash does not. | ✓ VERIFIED | `after_failure` returns `Requeue` for `Failure::Shutdown` with the same `retry_count`. `notify::decide` ignores a requeued cancel. `notify/recovery.rs` only notifies job texts that start with `failed` for automatic jobs. `tests/watch/notify/recovery.rs` asserts one script for `o/r#1` containing `interrupted: crash` and none for the manual job. |
| 16 | Search pages by raising `--limit` from 100, keeps drafts, skips one bad element, and fails the whole call on a non-array or a failed page. `gh pr view` includes `isDraft`, defaulting to false when absent. | ✓ VERIFIED | `search_review_requested` stops when the page is shorter than the limit or at 1000. `parse_page` requires an array (`{}` and invalid JSON are `Error::Json` in `src/gh/search/tests.rs`) and keeps an element only when URL, number, owner/repo, and open state agree. `src/gh/meta.rs` `is_draft` is `#[serde(default)]`. `GhCli::search_review_requested` uses `command()` and `run_tool`, not a shell. |

**Score:** 16/16 truths verified (0 present, behavior-unverified)

### Evolved plan constraints

These plan sentences are no longer literal. Current SPEC.md and later locked behavior replace them. They are not gaps.

| Plan sentence | Current behavior | Why it is not a gap |
| --- | --- | --- |
| Opening 0, 1, or 2 ends at user_version 3, and version 4 or newer is refused. | Open ends at user_version 5. Version 4 migrates. Version 6 or newer is refused unchanged. | Later migrations. The v3 columns and the v1 and v2 backups remain. |
| Search qualifier `--review-requested=@me`. | `user-review-requested:@me`. | SPEC queue source. This is what keeps a team request out of the queue. |
| Close or merge does not change `request_seen`. | Close or merge sets `tracking = untracked`, `request_seen = 0` only `WHERE tracking = 'tracked'`. | SPEC: tracking stops when the PR is merged or closed. A discarded row is not updated. |
| A draft gets no job. | A new draft still gets no job. An existing queued automatic job is held with `jobs.held = 1` (`migrate_v4.sql`) and is not runnable. | The draft is still not reviewed until it is ready. |
| A different head on a started retry resets that same job. | A retry stays on the first run's head. A different head cancels that job and inserts a new one. An unstarted debounce job is still reset in place. | Later retry rule. QUE-02 still produces one run for two pushes before the job starts. |
| The poll does not write `targets.head_sha`. | `schedule/write.rs` `note` records the observed head and `head_checked_at`. The runner still calls `set_known_head` from the post-agent `pr view` before `schedule_after_run`. | Last-known head is what draft freshness uses. Poll and absence tests assert the write. Reverting it would undo later behavior. |

### Decision Coverage

All trackable CONTEXT.md decisions are honored by shipped artifacts (9/9). The gate is non-blocking. D-03's old `--review-requested=@me` wording is implemented as the SPEC qualifier `user-review-requested:@me`. D-06's "end at version 3" is implemented as the v3 step inside the current migration to version 5.

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `src/store/migrate_v3.sql` | `request_seen` and `queued_head` | ✓ VERIFIED | Both columns. Wired from `migrate.rs` when `version < 3`. |
| `src/store/migrate/backup.rs` | Parameterized backup | ✓ VERIFIED | `create(db, version)` writes `diffmate.db.v{version}-backup`. Called for versions 1 through 4. |
| `src/gh/search.rs` | Paged direct-request search | ✓ VERIFIED | Used by `GhCli::search_review_requested`. |
| `src/gh/search/parse.rs` | Typed page parse | ✓ VERIFIED | `pr_url::parse` plus number, owner/repo, and open state. |
| `src/gh/meta.rs` | `is_draft` on `PrMeta` | ✓ VERIFIED | Field requested in `FIELDS` and defaulted. |
| `src/store/poll.rs` | `apply_poll` and `discard` | ✓ VERIFIED | One immediate transaction for tracking and jobs. |
| `src/store/poll/schedule.rs` | Debounce arming | ✓ VERIFIED | Called from `apply_poll` and `schedule_after_run`. |
| `src/watch/poll.rs` | Search, views, `apply_poll` | ✓ VERIFIED | Called from `watch/run.rs` only when the poll is due. |
| `src/watch/timing.rs` | Idle delay capped at 1s | ✓ VERIFIED | Used by the watch loop. |
| `src/config.rs` | Poll and debounce accessors | ✓ VERIFIED | Defaults 60. Watch reads them. |
| `src/notify/script.rs` | AppleScript from sanitized text | ✓ VERIFIED | `display notification` with escaped title and body. |
| `src/recovery/rules/transient.rs` | Transient classification | ✓ VERIFIED | `runner/outcome/failed.rs` maps a transient `AgentError` to `Failure::TransientAgent` before `after_failure`. |
| `src/debug.rs` | `debug discard` | ✓ VERIFIED | CLI calls `store.discard` and no other store write. |
| `tests/watch/notify/ready.rs` | Fake osascript | ✓ VERIFIED | Records argv. Wired by spawning `watch`. |
| `tests/watch/attach.rs` | Handoff with an empty poll | ✓ VERIFIED | Empty search, one agent run, review prints the result. |
| `tests/watch/wait.rs` | Watch waits and does not poll first | ✓ VERIFIED | Asserts no `search prs` while the lock is held. |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `src/store/migrate.rs` | `src/store/migrate/backup.rs` | `backup::create` before the migration that changes a v1 or v2 file | ✓ WIRED | Versions 1 and 2 snapshot, then v3+ SQL runs. |
| `src/gh.rs` | `src/gh/search.rs` | `command()` and `run_tool` | ✓ WIRED | Same process path as other gh calls. No shell. |
| `src/gh/search/parse.rs` | `src/pr_url.rs` | `pr_url::parse` | ✓ WIRED | Mismatched number or owner/repo is dropped. |
| `src/store/poll.rs` | `src/store/poll/schedule.rs` | One immediate transaction | ✓ WIRED | `tracking::apply` then `schedule::apply`, then commit. |
| `src/runner/run.rs` | `src/store/poll/schedule.rs` | `set_known_head` then the watch loop's `schedule_after_run` | ✓ WIRED | Post-agent `pr view` stores the head before `watch/job.rs` `follow` reads it. |
| `src/watch/run.rs` | `src/watch/poll.rs` | `poll::once` when `next_poll_at` is due, after the lock | ✓ WIRED | A failed poll is logged and does not apply. |
| `src/watch/poll.rs` | `src/store/poll.rs` | `apply_poll` only after a successful search | ✓ WIRED | Views that fail become `Absent::Skip`. |
| `src/watch/run.rs` | `src/notify.rs` | `watch/job.rs` `announce` after the outcome line | ✓ WIRED | Only automatic terminal results. A failed `osascript` is logged and does not change the job. |
| `src/runner/outcome/failed.rs` | `src/recovery/rules/transient.rs` | `is_transient` | ✓ WIRED | Non-transient agent errors stay terminal. |
| `src/debug.rs` | `src/store/poll.rs` | `store.discard` | ✓ WIRED | No other write. |
| `src/review.rs` | `src/review/print.rs` | `print::stored` when the waited job finishes | ✓ WIRED | Watch does not print findings. |
| `src/watch.rs` | `src/watch/run.rs` | `announce` then recovery, then `jobs` | ✓ WIRED | Poll stays inside the locked loop. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `src/gh/search.rs` | search hits | `gh search prs` stdout, parsed | Yes, or `Err` on a bad page | ✓ FLOWING |
| `src/watch/poll.rs` | seen and absent | search hits plus `gh pr view` | Yes. A failed view is `Skip`, not a fake open state | ✓ FLOWING |
| `src/store/poll.rs` | tracking, jobs | those rows, in one transaction | Yes. No static job list | ✓ FLOWING |
| `src/notify/script.rs` | title and body | automatic `ReviewResult` label and failure text | Yes. Titles are fixed; body is the label or `{label}: {reason}` | ✓ FLOWING |
| `src/review/print.rs` | findings | `store.stored_result` | Yes, through `render::review` | ✓ FLOWING |

### Behavioral Spot-Checks

This verifier did not execute tests. The orchestrator observed `mise run --force test` on this tree: 509 passed, 11 skipped. The rows below are the tests whose assertions were read.

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Ready request becomes one automatic run and uses the direct-user qualifier | `mise run --force test` (orchestrator), test `a_ready_request_becomes_one_automatic_run` | suite 509 passed, 11 skipped; assertion read | ✓ PASS |
| Two heads before start leave one run | same suite, `two_heads_before_the_job_starts_leave_one_later_run` | assertion read | ✓ PASS |
| Open absence stays tracked with `request_seen = 0` | same suite, `an_open_absence_keeps_one_delayed_job_for_the_new_head` | assertion read | ✓ PASS |
| Draft is not run until ready | same suite, `a_draft_is_tracked_without_a_job_until_it_is_ready` | assertion read | ✓ PASS |
| Manual review does not track | same suite, `a_manual_review_stays_untracked_until_search_contains_it` | assertion read | ✓ PASS |
| Discard latch and re-request | same suite, `a_still_visible_request_stays_untracked_until_it_returns` | assertion read | ✓ PASS |
| Give-up notification names the reason; SIGTERM does not notify | same suite, `an_automatic_give_up_names_the_reason`, `sigterm_during_an_automatic_run_does_not_notify` | assertion read | ✓ PASS |
| Review attaches to watch; watch waits for the lock | same suite, `review_while_watch_runs_prints_the_result`, `watch_waits_for_the_lock_and_stops_on_sigterm` | assertion read | ✓ PASS |

### Probe Execution

No phase probe is declared. Step 7c is not applicable.

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| none | — | no `probe-*.sh` declared by the phase plans | n/a |

### Requirements Coverage

Every QUE id in REQUIREMENTS.md for Phase 3 appears in at least one plan. `03-01` declares `requirements: []` because it is the schema step those ids depend on. No phase 3 id is orphaned.

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| QUE-01 | 03-04 | Direct request leads to a ready draft and a notification with no action | ✓ SATISFIED | Truth 1 |
| QUE-02 | 03-03 | Two pushes inside the debounce window produce one run | ✓ SATISFIED | Truth 2 |
| QUE-03 | 03-03, 03-07 | Stays tracked after the request disappears | ✓ SATISFIED | Truth 3 |
| QUE-04 | 03-02, 03-03 | A draft is not reviewed until ready | ✓ SATISFIED | Truths 4 and 16 |
| QUE-05 | 03-02 | A team request does not queue the PR | ✓ SATISFIED | Truth 5 |
| QUE-06 | 03-03, 03-07 | A manual review does not track or auto-review later heads | ✓ SATISFIED | Truth 6 |
| QUE-07 | 03-05, 03-07 | Discard stays untracked while the request is visible; a real re-request tracks again | ✓ SATISFIED | Truths 7 and 14 |
| QUE-08 | 03-04, 03-08 | Transient failure is retried; the last failure notifies with the reason | ✓ SATISFIED | Truths 8 and 15 |
| QUE-09 | 03-06 | `review` during `watch` hands off and prints the result | ✓ SATISFIED | Truth 9 |
| QUE-10 | 03-06 | `watch` waits for a lock held by `review` | ✓ SATISFIED | Truth 10 |
| QUE-11 | 03-03 | Poll interval and debounce default to 60 and are read from config | ✓ SATISFIED | Truth 11 |

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
|-----------|-----------|--------|---------|----------|-----------------|---------|
| `tests/watch/poll/ready.rs` | QUE-01, QUE-05, QUE-11 | 2 | 0 | no | behavioral | sufficient |
| `tests/watch/notify/ready.rs` | QUE-01 | 3 | 0 | no | value (script text, job state) | sufficient |
| `tests/watch/poll/debounce.rs` | QUE-02 | 1 | 0 | no | behavioral | sufficient |
| `tests/watch/poll/absent.rs` | QUE-03 | 2 | 0 | no | value (tracking, seen, head) | sufficient |
| `tests/watch/poll/draft.rs` | QUE-04 | 1 | 0 | no | behavioral | sufficient |
| `src/gh/search/tests/args.rs` | QUE-05 | 1 | 0 | no | value (argv) | sufficient |
| `tests/watch/poll/manual.rs` | QUE-06 | 1 | 0 | no | behavioral | sufficient |
| `tests/watch/discard.rs` | QUE-07 | 4 | 0 | no | behavioral | sufficient |
| `src/recovery/rules/tests/transient.rs` | QUE-08 | 4 | 0 | no | value (budget and delays) | sufficient |
| `tests/watch/notify/give_up.rs` | QUE-08 | 2 | 0 | no | value (reason in the script) | sufficient |
| `tests/watch/attach.rs` | QUE-09 | 2 | 0 | no | behavioral | sufficient |
| `tests/watch/wait.rs` | QUE-10 | 1 | 0 | no | behavioral | sufficient |
| `src/config/tests.rs` | QUE-11 | defaults test | 0 | no | value | sufficient |

**Disabled tests on requirements:** 0. No `#[ignore]` on these files.
**Circular patterns detected:** 0. Fakes supply gh, claude, and osascript output; expected rows are asserted, not written by the system under test.
**Insufficient assertions:** 0 blocking. The handoff test prints a success summary rather than a seeded finding line. The printer it calls is `render::review`, which writes findings when the stored result has them. The give-up process test uses `max_retries = 0`; the multi-attempt budget is the unit test `a_rate_limit_uses_the_same_budget_as_a_timeout`, on the same `after_failure` path the runner calls.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `src/watch/**`, `src/notify/**`, `src/store/poll/**`, `src/gh/search/**`, `src/recovery/rules/**`, `src/debug.rs` | — | No `TBD`, `FIXME`, or `XXX` | ℹ️ Info | None |

No stub returns, empty handlers, or disconnected fetches were found on the queue path. `hold` updates an existing queued job and does not invent a runnable one. A failed notification is logged and does not rewrite retry columns.

### Human Verification Required

### 1. Live direct request

**Test:** With `diffmate watch` running, request your review directly on a real GitHub pull request. Do not run `diffmate review`.
**Expected:** The PR is tracked, one automatic review finishes as a ready draft, and Notification Center shows `diffmate: draft ready` with the PR label. No other action is required.
**Why human:** Fake `gh` and a fake `osascript` prove the code path. They do not call GitHub or Notification Center.

### 2. Live give-up notification

**Test:** Let an automatic review use up its retries, or fail terminally, while `watch` is running. Separately, send SIGTERM during an automatic run.
**Expected:** The give-up shows `diffmate: review failed` with the PR label and the failure reason. SIGTERM requeues the job and does not notify.
**Why human:** Tests check the AppleScript text and an empty script log. They do not display a notification.

### Gaps Summary

No code gap blocks the phase goal. The ten roadmap success criteria and QUE-11 are implemented and covered by fakes. Two checks still need a person: a real GitHub review request that becomes a draft with no local action, and a real macOS notification for a ready draft and for a give-up. Those are external delivery checks, not missing code.

The phase was not marked accepted.

---

_Verified: 2026-11-12T10:55:42Z_
_Verifier: Claude (gsd-verifier)_
