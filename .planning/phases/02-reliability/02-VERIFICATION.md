---
phase: 02-reliability
verified: 2026-11-12T10:02:32Z
status: human_needed
score: 24/24 must-haves verified
covered_files: [".github/workflows/ci.yml", ".planning/phases/02-reliability/02-01-PLAN.md", ".planning/phases/02-reliability/02-01-SUMMARY.md", ".planning/phases/02-reliability/02-02-PLAN.md", ".planning/phases/02-reliability/02-02-SUMMARY.md", ".planning/phases/02-reliability/02-03-PLAN.md", ".planning/phases/02-reliability/02-03-SUMMARY.md", ".planning/phases/02-reliability/02-04-PLAN.md", ".planning/phases/02-reliability/02-04-SUMMARY.md", ".planning/phases/02-reliability/02-05-PLAN.md", ".planning/phases/02-reliability/02-05-SUMMARY.md", ".planning/phases/02-reliability/02-06-PLAN.md", ".planning/phases/02-reliability/02-06-SUMMARY.md", ".planning/phases/02-reliability/02-07-PLAN.md", ".planning/phases/02-reliability/02-07-SUMMARY.md", ".planning/phases/02-reliability/02-08-PLAN.md", ".planning/phases/02-reliability/02-08-SUMMARY.md", ".planning/phases/02-reliability/02-09-PLAN.md", ".planning/phases/02-reliability/02-09-SUMMARY.md", ".planning/phases/02-reliability/02-10-PLAN.md", ".planning/phases/02-reliability/02-10-SUMMARY.md", "Cargo.lock", "Cargo.toml", "hk.pkl", "mise.toml", "src/agent.rs", "src/agent/claude.rs", "src/agent/claude/cancel_tests.rs", "src/agent/claude/contract.rs", "src/agent/claude/contract_tests.rs", "src/agent/claude/env_tests.rs", "src/agent/claude/tests.rs", "src/agent/envelope.rs", "src/agent/envelope/fields.rs", "src/agent/envelope/golden_tests.rs", "src/agent/envelope/property_tests.rs", "src/agent/envelope/support.rs", "src/agent/envelope/synthetic_tests.rs", "src/agent/envelope/tests.rs", "src/agent/gitwrap.rs", "src/agent/gitwrap/tests.rs", "src/agent/output.rs", "src/agent/probe.rs", "src/agent/probe/live_tests.rs", "src/agent/probe/run.rs", "src/agent/probe/tests.rs", "src/agent/request.rs", "src/agent/schema.rs", "src/agent/schema/generated_tests.rs", "src/agent/schema/tests.rs", "src/agent/spawn.rs", "src/agent/stop.rs", "src/agent/stop/tests.rs", "src/app.rs", "src/cancel.rs", "src/cancel/tests.rs", "src/cli.rs", "src/config.rs", "src/config/error.rs", "src/config/error/key.rs", "src/config/raw.rs", "src/config/tests.rs", "src/config/tests/errors.rs", "src/config/tests/files.rs", "src/context.rs", "src/context/caps.rs", "src/context/ci.rs", "src/context/discussion.rs", "src/context/files.rs", "src/context/pr.rs", "src/context/previous.rs", "src/context/rules.rs", "src/context/tests.rs", "src/context/tests/caps_cases.rs", "src/context/tests/ci_cases.rs", "src/context/tests/ci_unknown.rs", "src/context/tests/cut_property.rs", "src/context/tests/discussion_cases.rs", "src/context/tests/discussion_limits.rs", "src/context/tests/fake_gh.rs", "src/context/tests/previous_shapes.rs", "src/context/tests/previous_status.rs", "src/crash.rs", "src/debug.rs", "src/diff.rs", "src/diff/hunk.rs", "src/diff/path.rs", "src/diff/tests.rs", "src/diff/tests/model.rs", "src/diff/tests/never_panics.rs", "src/diff/tests/place.rs", "src/diff/tests/roundtrip.rs", "src/doctor.rs", "src/doctor/claude.rs", "src/doctor/gh_auth.rs", "src/doctor/lock_line.rs", "src/doctor/schema.rs", "src/doctor/scratch.rs", "src/doctor/storage.rs", "src/doctor/which.rs", "src/error.rs", "src/gh.rs", "src/gh/checks.rs", "src/gh/discussion.rs", "src/gh/discussion/raw.rs", "src/gh/discussion/raw/shapes.rs", "src/gh/discussion/tests.rs", "src/gh/meta.rs", "src/gh/reviews.rs", "src/gh/reviews/parse.rs", "src/gh/reviews/tests.rs", "src/gh/search.rs", "src/gh/search/parse.rs", "src/gh/search/tests.rs", "src/gh/search/tests/args.rs", "src/gh/search/tests/malformed.rs", "src/gh/search/tests/pages.rs", "src/gh/search/tests/view.rs", "src/gh/threads.rs", "src/gh/threads/parse.rs", "src/git.rs", "src/git/build.rs", "src/git/cli.rs", "src/git/config.rs", "src/git/config_tests.rs", "src/git/fixture.rs", "src/git/fixture/poison.rs", "src/git/isolation_tests.rs", "src/git/refname.rs", "src/git/refname/tests.rs", "src/git/tests.rs", "src/git/tree.rs", "src/git/tree_tests.rs", "src/git/worktree_tests.rs", "src/lock.rs", "src/lock/role.rs", "src/lock/tests.rs", "src/main.rs", "src/normalize.rs", "src/normalize/tests.rs", "src/normalize/tests/fixture.rs", "src/normalize/tests/property.rs", "src/notify.rs", "src/notify/recovery.rs", "src/notify/script.rs", "src/notify/tests.rs", "src/paths.rs", "src/pr_url.rs", "src/pr_url/tests.rs", "src/proc.rs", "src/procenv.rs", "src/publish.rs", "src/publish/body.rs", "src/publish/checks.rs", "src/publish/gate.rs", "src/publish/load.rs", "src/publish/outcome/mod.rs", "src/publish/outcome/settle.rs", "src/publish/post.rs", "src/publish/reconcile/mod.rs", "src/publish/reconcile/write.rs", "src/publish/sync.rs", "src/publish/sync/apply.rs", "src/publish/sync/apply/write.rs", "src/publish/tests/body.rs", "src/publish/tests/confirm.rs", "src/publish/tests/confirm/accept.rs", "src/publish/tests/confirm/boom.rs", "src/publish/tests/confirm/budget.rs", "src/publish/tests/confirm/harness.rs", "src/publish/tests/confirm/moved.rs", "src/publish/tests/confirm/reviews.rs", "src/publish/tests/crash.rs", "src/publish/tests/crash/abandon.rs", "src/publish/tests/crash/before.rs", "src/publish/tests/crash/gh.rs", "src/publish/tests/crash/gh/remote.rs", "src/publish/tests/crash/harness.rs", "src/publish/tests/crash/keys.rs", "src/publish/tests/crash/marker.rs", "src/publish/tests/crash/outcome.rs", "src/publish/tests/flow.rs", "src/publish/tests/gates.rs", "src/publish/tests/mod.rs", "src/publish/tests/once.rs", "src/publish/tests/script.rs", "src/publish/tests/snap.rs", "src/publish/tests/support.rs", "src/publish/tests/sync.rs", "src/publish/tests/sync/harness.rs", "src/publish/tests/sync/harness/remote.rs", "src/publish/tests/sync/link.rs", "src/recovery.rs", "src/recovery/cleanup.rs", "src/recovery/fake.rs", "src/recovery/rules.rs", "src/recovery/rules/failure.rs", "src/recovery/rules/tests.rs", "src/recovery/rules/tests/support.rs", "src/recovery/rules/tests/table.rs", "src/recovery/rules/tests/tokens.rs", "src/recovery/rules/tests/transient.rs", "src/recovery/rules/transient.rs", "src/recovery/scan.rs", "src/recovery/tests.rs", "src/recovery/tests/budget.rs", "src/recovery/tests/causes.rs", "src/recovery/tests/fixture.rs", "src/recovery/tests/orphan.rs", "src/render.rs", "src/render/findings.rs", "src/render/sanitize.rs", "src/render/tests.rs", "src/render/tests/fixture.rs", "src/render/tests/freshness.rs", "src/render/tests/retry.rs", "src/render/tests/sanitize.rs", "src/review.rs", "src/review/acquire.rs", "src/review/follow.rs", "src/review/print.rs", "src/review/signals.rs", "src/review/signals/tests.rs", "src/review/wait.rs", "src/review/wait/tests.rs", "src/runner.rs", "src/runner/agent_run.rs", "src/runner/agent_run/deadline.rs", "src/runner/agent_run/fakes.rs", "src/runner/agent_run/fakes/git.rs", "src/runner/agent_run/tests.rs", "src/runner/agent_run/tests/completion.rs", "src/runner/agent_run/tests/edges.rs", "src/runner/agent_run/tests/fixture.rs", "src/runner/agent_run/tests/persistence.rs", "src/runner/agent_run/tests/persistence_run.rs", "src/runner/agent_run/timing_tests.rs", "src/runner/cancelled.rs", "src/runner/cleanup.rs", "src/runner/outcome.rs", "src/runner/outcome/failed.rs", "src/runner/outcome/succeeded.rs", "src/runner/outcome/tests.rs", "src/runner/outcome/tests/fixture.rs", "src/runner/outcome/tests/later.rs", "src/runner/outcome/tests/preservation.rs", "src/runner/outcome/tests/retry.rs", "src/runner/outcome/tests/terminal.rs", "src/runner/outcome/tests/transient.rs", "src/runner/process.rs", "src/runner/resolve.rs", "src/runner/resolve/tests.rs", "src/runner/resolve/tests/gh.rs", "src/runner/resolve/tests/git.rs", "src/runner/resolve/tests/reuse.rs", "src/runner/resolve/tests/reuse/seed.rs", "src/runner/result.rs", "src/runner/result/stored.rs", "src/runner/run.rs", "src/runner/steps.rs", "src/runner/tests.rs", "src/runner/tests/before.rs", "src/status.rs", "src/status/budget.rs", "src/status/freshness.rs", "src/status/tests.rs", "src/status/tests/budget.rs", "src/status/tests/fixture.rs", "src/status/tests/stored.rs", "src/status/when.rs", "src/store.rs", "src/store/decisions.rs", "src/store/finish.rs", "src/store/jobs.rs", "src/store/jobs/attach.rs", "src/store/jobs/attach/active.rs", "src/store/jobs/due.rs", "src/store/jobs/tests.rs", "src/store/jobs/tests/fixture.rs", "src/store/jobs/tests/held.rs", "src/store/jobs/tests/stop.rs", "src/store/migrate.rs", "src/store/migrate/backup.rs", "src/store/migrate/tests.rs", "src/store/migrate/tests/backups.rs", "src/store/migrate/tests/concurrency.rs", "src/store/migrate/tests/helpers.rs", "src/store/migrate/tests/rollback.rs", "src/store/migrate/tests/v3.rs", "src/store/migrate/tests/v5.rs", "src/store/migrate_v2.sql", "src/store/migrate_v3.sql", "src/store/migrate_v4.sql", "src/store/migrate_v5.sql", "src/store/poll.rs", "src/store/poll/known.rs", "src/store/poll/schedule.rs", "src/store/poll/schedule/rows.rs", "src/store/poll/schedule/write.rs", "src/store/poll/tests.rs", "src/store/poll/tests/absent.rs", "src/store/poll/tests/attempt.rs", "src/store/poll/tests/block.rs", "src/store/poll/tests/closed.rs", "src/store/poll/tests/discard.rs", "src/store/poll/tests/draft.rs", "src/store/poll/tests/enqueue.rs", "src/store/poll/tests/fixture.rs", "src/store/poll/tests/follow.rs", "src/store/poll/tests/heads.rs", "src/store/poll/tests/hold.rs", "src/store/poll/tests/jobs.rs", "src/store/poll/tests/runs.rs", "src/store/poll/tests/schedule.rs", "src/store/poll/tests/seen.rs", "src/store/poll/tracking.rs", "src/store/previous.rs", "src/store/publish.rs", "src/store/publish/cancel.rs", "src/store/publish/confirm.rs", "src/store/publish/insert.rs", "src/store/publish/rows.rs", "src/store/publish/save.rs", "src/store/publish/sync.rs", "src/store/readonly.rs", "src/store/records.rs", "src/store/recovery.rs", "src/store/recovery/resolve.rs", "src/store/recovery/service.rs", "src/store/recovery/service/write.rs", "src/store/result.rs", "src/store/result/read.rs", "src/store/retry.rs", "src/store/retry/queue.rs", "src/store/retry/revision.rs", "src/store/retry/tests.rs", "src/store/runs.rs", "src/store/schema.sql", "src/store/status.rs", "src/store/status/draft.rs", "src/store/status/latest.rs", "src/store/status/queue.rs", "src/store/targets.rs", "src/tests.rs", "src/tests/isolation.rs", "src/tests/isolation/auth.rs", "src/tests/isolation/contract.rs", "src/tests/isolation/endpoint.rs", "src/tests/isolation/endpoint_check.rs", "src/tests/isolation/fixture.rs", "src/tests/isolation/fixture/context.rs", "src/tests/isolation/fixture/files.rs", "src/tests/isolation/fixture/home.rs", "src/tests/isolation/fixture/layout.rs", "src/tests/isolation/fixture/ops.rs", "src/tests/isolation/fixture/scripts.rs", "src/tests/isolation/fixture/worktree.rs", "src/tests/isolation/fixture_check.rs", "src/tests/isolation/harness.rs", "src/tests/isolation/harness_check.rs", "src/tests/isolation/http.rs", "src/tests/isolation/inspect.rs", "src/tests/isolation/rev06.rs", "src/tests/isolation/rev07.rs", "src/tests/isolation/rev08.rs", "src/tests/live.rs", "src/tests/live/agent.rs", "src/tests/live/prs.rs", "src/tests/live/stored.rs", "src/tests/live/support.rs", "src/tui.rs", "src/tui/confirm.rs", "src/tui/draw.rs", "src/tui/draw/draft.rs", "src/tui/edit.rs", "src/tui/keys.rs", "src/tui/model.rs", "src/tui/open.rs", "src/tui/post.rs", "src/tui/runner.rs", "src/tui/tests/actions.rs", "src/tui/tests/cancel.rs", "src/tui/tests/decisions.rs", "src/tui/tests/draft.rs", "src/tui/tests/edit.rs", "src/tui/tests/events.rs", "src/tui/tests/fake.rs", "src/tui/tests/focus.rs", "src/tui/tests/mod.rs", "src/tui/tests/publish.rs", "src/tui/tests/queue.rs", "src/tui/tests/refusals.rs", "src/watch.rs", "src/watch/events.rs", "src/watch/events/date.rs", "src/watch/events/outcomes.rs", "src/watch/events/tests.rs", "src/watch/job.rs", "src/watch/poll.rs", "src/watch/run.rs", "src/watch/service_tests.rs", "src/watch/service_tests/remote.rs", "src/watch/service_tests/rig.rs", "src/watch/timing.rs", "src/watch/timing/tests.rs", "src/watch/wait.rs", "tests/agent_contract.rs", "tests/agent_contract/classification.rs", "tests/attach.rs", "tests/attach/finished.rs", "tests/attach/forced.rs", "tests/attach/helpers.rs", "tests/cli.rs", "tests/concurrency.rs", "tests/concurrency/helpers.rs", "tests/config.rs", "tests/context.rs", "tests/context/helpers.rs", "tests/context_rules.rs", "tests/crash.rs", "tests/crash/budget.rs", "tests/crash/helpers.rs", "tests/crash/manual_stop.rs", "tests/crash/matrix.rs", "tests/crash/matrix/assertions.rs", "tests/crash/process.rs", "tests/crash/restart.rs", "tests/crash/stop_stored.rs", "tests/dedup.rs", "tests/doctor.rs", "tests/doctor/fresh.rs", "tests/doctor/held.rs", "tests/doctor/lock_role.rs", "tests/doctor/pass.rs", "tests/doctor/schema.rs", "tests/doctor/storage.rs", "tests/doctor/tools.rs", "tests/doctor/wal.rs", "tests/fixtures/claude/cases.json", "tests/fixtures/claude/contract.json", "tests/fixtures/claude/error-max-budget.stdout", "tests/fixtures/claude/error-max-turns.stdout", "tests/fixtures/claude/error-not-logged-in.stdout", "tests/fixtures/claude/error-unknown-model.stdout", "tests/fixtures/claude/success-findings.stdout", "tests/fixtures/claude/success-no-findings.stdout", "tests/fixtures/live-prs.json", "tests/fork.rs", "tests/git_isolation.rs", "tests/harness.rs", "tests/migration.rs", "tests/migration/fixture.rs", "tests/migration/fixture.sql", "tests/recovery.rs", "tests/rerun.rs", "tests/rerun/cancel.rs", "tests/rerun/delayed.rs", "tests/review.rs", "tests/review_output.rs", "tests/review_output/helpers.rs", "tests/review_output/previous.rs", "tests/review_output/render.rs", "tests/signals.rs", "tests/signals/helpers.rs", "tests/signals/process.rs", "tests/stale_review.rs", "tests/stale_review/edges.rs", "tests/status.rs", "tests/status/helpers.rs", "tests/status_freshness.rs", "tests/status_freshness/failure.rs", "tests/support/envelope.rs", "tests/support/fake_call.rs", "tests/support/fake_claude.rs", "tests/support/fake_claude.sh", "tests/support/fake_gh.rs", "tests/support/fake_gh.sh", "tests/support/mod.rs", "tests/support/poison.rs", "tests/support/sandbox.rs", "tests/support/upstream.rs", "tests/tracer/agent_call.rs", "tests/tracer/mod.rs", "tests/tracer/stored.rs", "tests/tui.rs", "tests/watch.rs", "tests/watch/attach.rs", "tests/watch/debug.rs", "tests/watch/discard.rs", "tests/watch/helpers.rs", "tests/watch/notify.rs", "tests/watch/notify/attached.rs", "tests/watch/notify/give_up.rs", "tests/watch/notify/ready.rs", "tests/watch/notify/recovery.rs", "tests/watch/notify/review.rs", "tests/watch/notify/support.rs", "tests/watch/persistence.rs", "tests/watch/poll.rs", "tests/watch/poll/absent.rs", "tests/watch/poll/debounce.rs", "tests/watch/poll/draft.rs", "tests/watch/poll/follow.rs", "tests/watch/poll/head.rs", "tests/watch/poll/life.rs", "tests/watch/poll/manual.rs", "tests/watch/poll/ready.rs", "tests/watch/poll/sync.rs", "tests/watch/poll/view.rs", "tests/watch/shutdown.rs", "tests/watch/wait.rs"]
covered_digest: "v4:sha256:5ab4767f41bf21bfca09827ff5377969019288e3753a5040840fac8f8d579e6b"
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: human_needed
  previous_score: 24/24
  gaps_closed: []
  gaps_remaining: []
  regressions: []
decision_coverage:
  honored: 16
  total: 16
  not_honored: []
phase_gate: blocked
external_gate: Anthropic account verification HTTP 403
requirements_covered: 13
artifacts_declared: 34
key_links_verified: 23
human_verification:
  - id: LIVE-01
    source: "WINDOWS-11"
    test: "After the owner separately restores Anthropic access, rerun isolation_flags_keep_authentication."
    expected: "Real Claude runs with the production isolation flags and does not return HTTP 403."
    why_human: "External account-verification barrier. The owner chose to keep the live gate blocked. That is not a waiver and not permission to call the Claude API, live gh, or mise run test:e2e. Local fakes do not prove this check."
  - id: LIVE-02
    source: "WINDOWS-12"
    test: "After account access is restored, run the four remaining model-backed cases with mise run test:e2e."
    expected: "Max-turns is failed, a clean PR is no_findings, and the bug and fork PRs produce the expected finding at calc.py:7."
    why_human: "Those four cases were not run after auth failed fast. Local fakes do not prove the model-backed result."
---

# Phase 2: Reliability Verification Report

**Phase Goal:** A review can be killed, interrupted, repeated, or overtaken by new pushes at any point without duplicate runs, orphaned worktrees, a lost stop cause, or unbounded retries, and the draft always says whether it still matches the PR head.
**Verified:** 2026-11-12T10:02:32Z
**Status:** human_needed
**Re-verification:** Yes — the 2026-11-03 report was replaced because its covered files no longer matched HEAD `cb53dd29f9e7cf1f7671c7c99c5fd157413013b2`. There was no `gaps:` list to close. Owner UAT tests 1–10 and 13–14 are pass. LIVE-01 and LIVE-02 stay blocked. This is not phase acceptance.

## Goal Achievement

The 11 ROADMAP success criteria and the plan must-haves are checked against the current tree, not against SUMMARY.md. `VERIFIED` means the current code is wired and a named behavioral test passed in this pass, or the same transition was exercised by one of those tests. It does not mean the blocked live model gate passed. The phase is not accepted.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| T01 | Killing diffmate at any step and starting it again leaves no duplicate runs, no orphaned worktrees, and no stale worktree metadata, and interrupted runs show as `failed (interrupted: crash)`. | ✓ VERIFIED | `crash_point!` remains after enqueued, locked, recovery-db, recovered, job-started, fetched, revision, run-created, worktree, context, agent-start, agent-running, cleaned, stored, and stop-stored. `reached` still stops on `DIFFMATE_CRASH_POINT`. This pass: `matrix::agent_running` passed (SIGKILL at agent-running, restart, clean). The other 13 matrix cases and `assert_clean` are still in `tests/crash/matrix.rs`. |
| T02 | Pushing to the PR during a run makes the draft stale. | ✓ VERIFIED | `runner.rs` stores the start head; `runner/run.rs` reads the head again after a succeeded agent; `render.rs` appends `stale: PR head is now` when `known_head != head_sha`. This pass: `push_during_run_makes_the_draft_stale` passed. |
| T03 | Advancing the base branch, without a push to the PR, does not make the draft stale. | ✓ VERIFIED | Freshness compares heads only (`render.rs`, `status.rs` via `freshness::describe`). This pass: `base_move_during_run_is_not_stale` and `a_moved_base_branch_keeps_the_draft_current` passed. |
| T04 | Running `review` twice on the same head reuses the run unless `--force` is given. | ✓ VERIFIED | `execute_review_job` calls `succeeded_run_for_head` before inserting a run, and skips it when `started.force` is set. This pass: `same_head_reuses_the_run` and `force_makes_a_new_run` passed. |
| T05 | An invalid `previous_finding_id` is nulled with a warning and the run still succeeds. | ✓ VERIFIED | `normalize.rs` clears an id that is not in `previous.json` and records the warning. This pass: `unknown_previous_finding_id_is_nulled` passed. |
| T06 | A PR whose review crashes diffmate every time is retried only within the retry budget, across restarts, and then reported as failed. | ✓ VERIFIED | `after_failure` requeues automatic crash/timeout only while `retry_count < max`, then fails with `no retries left`. This pass: `crash_every_time_exhausts_the_budget_across_watch_restarts` passed on the current watch loop. |
| T07 | Recovery after a stop whose cause was stored but whose cleanup did not finish keeps the cause: `shutdown` re-queues an automatic job without using the budget, `user_cancel` ends the job without a retry. | ✓ VERIFIED | `request_stop` commits `set_stop_reason` before `send_replace`; `stop-stored` sits between them. `after_failure` requeues shutdown with the same count and fails user_cancel. This pass: `shutdown_cause_survives_a_crash_and_requeues`, `user_cancel_cause_survives_a_crash_and_is_not_retried`, and `stored_causes_survive_recovery_and_a_second_recovery_is_inert` passed. |
| T08 | Stopping `watch` with SIGTERM during an automatic run records `failed (interrupted: shutdown)`, removes the worktree, and re-queues the job without using the retry budget. | ✓ VERIFIED | `watch/run.rs` `alongside` calls `request_stop` with `Shutdown` and returns before the next job. This pass: `sigterm_during_an_automatic_run_requeues_it` passed. |
| T09 | Ctrl-C on a `review` that acts as the runner stops and waits for the agent, records `failed (interrupted: user_cancel)` (never `agent_error`), leaves no agent or `rg` processes in its group, removes the worktree, and says so before exiting. | ✓ VERIFIED | `review.rs` uses `until_done` with `UserCancel`. `outcome.rs` maps a stored user_cancel to `FailureKind::Interrupted`. `supervise` signals `child.id()`, not a row from SQLite. `a_failed_signal_write_does_not_cancel_until_persisted` passed. The process-group assertions remain in `tests/signals.rs`. UAT-01 is already pass. |
| T10 | Without `watch`, a `review` that attached to a `tui` re-run executes it, and does not start orphaned or automatic jobs. | ✓ VERIFIED | `review.rs` runs the job returned by `enqueue_review`. `follow::again` continues only a requeued failure of that same review, not another target's queue. This pass: `review_runs_only_the_rerun_it_attached_to` passed. |
| T11 | Two `review` commands for the same URL produce one run, a `review` started while a `tui` re-run is queued attaches to it, and `review --force` during a running non-forced job produces a second run. | ✓ VERIFIED | `enqueue_review` is still the attach transaction (`src/store/jobs/attach.rs`). This pass: `two_reviews_of_one_url_produce_one_run` passed. Force-while-running remains in `tests/attach/forced.rs`. |
| T12 | The run timeout covers preparation and the agent after the run row exists; the timeout cause is stored before cancel; the outcome is `failed (timeout)`; the group and worktree are cleaned up. | ✓ VERIFIED | `run_agent` wraps `execute` (prepare and agent) in `with_deadline`. `persist_timeout` runs before `send_replace`. This pass: `a_run_over_the_timeout_is_failed_timeout` passed. |
| T13 | Config is read once before the database for review, watch, and tui; a missing or partial file uses the SPEC defaults; invalid keys, types, or ranges are a hard error; status and debug enqueue do not read config. | ✓ VERIFIED | `main.rs` calls `config::load` only for Review, Watch, and Tui, before the runtime. Defaults are 15/60/60/2. `deny_unknown_fields` is on the raw struct. `debug.rs` does not call `config::load`. This pass: `invalid_config_stops_review_before_enqueue` passed. |
| T14 | Opening a Phase 1 database keeps every old row and value and writes a valid private v1 backup before the schema changes. | ✓ VERIFIED | `migrate.rs` still backs up version 1, then applies `migrate_v2.sql`, inside one IMMEDIATE transaction. This pass: `phase_one_database_is_upgraded_and_kept` passed, including preserved rows and `diffmate.db.v1-backup` at user_version 1. The chain now continues to user_version 5; that is later schema work, not a dropped backup. |
| T15 | Schema v2 columns, defaults, and constraints are still applied; a fresh database and an upgraded Phase 1 database converge; reopening the current version is inert; a newer-than-supported version is refused; a read-only open of version 1 does not upgrade it. | ✓ VERIFIED | `SCHEMA_VERSION` is 5, as required. Version 0 runs `schema.sql` then `migrate_v2.sql`; version 1 backs up and runs `migrate_v2.sql`; versions below 5 then run v3–v5. `SCHEMA_VERSION => return Ok(())`. `readonly.rs` returns `SchemaOutdated` for versions in `1..5` and does not migrate. This pass: the upgrade test expects user_version 5 with the v2 job defaults, and `status_on_phase_one_database_asks_for_an_upgrade` passed with user_version left at 1. The plan sentences "stop at 2" and "refuse above 2" are superseded by the later migrations; they are not a Phase 2 gap. |
| T16 | The first stored stop reason wins, including before a run row exists; a finished result is not rewritten by a later signal; a SQLite fault does not cancel until the reason is durable. | ✓ VERIFIED | `set_stop_reason` updates only `stop_reason IS NULL AND state = 'running'`. `cancelled_before_run` records `interrupted before the run started (CAUSE)` for a manual job. `request_stop` loops until the write succeeds or the job is no longer waiting, and only then calls `send_replace`. This pass: `a_failed_signal_write_does_not_cancel_until_persisted` passed. |
| T17 | Status is read-only: one live head read per target that has a draft, a failed check shows the stored head and age, a target without a draft makes no GitHub call, and retry text comes from SQLite. | ✓ VERIFIED | `status.rs` uses `Store::open_read_only` and calls `gh.pr_view` only inside `if let Some(draft)`. `budget::describe` formats the job retry columns. `main.rs` does not load config for Status. `a_moved_base_branch_keeps_the_draft_current` passed. |
| T18 | The retry maximum is fixed at the first retryable failure; zero budget, delays, due order, and idempotent recovery hold; an automatic retry keeps the first revision. | ✓ VERIFIED | `retry_delay` is 60s then 300s. `next_runnable_job` selects the lowest queued review id whose `next_attempt_at` is null or due. This pass: `automatic_retries_keep_the_stored_revision_when_the_pr_head_moves` and the second-recovery assertion in `stored_causes_survive_recovery_and_a_second_recovery_is_inert` passed. |
| T19 | Waiters follow `result_run_id` without the lock; stopping a waiter does not cancel the job; attach keeps origin and tracking; a future retry waits without holding the lock. | ✓ VERIFIED | `review/wait.rs` remains the lock wait. `review/acquire.rs` returns `Finished` from `stored_result` before running, and delays while `job_delayed`. `tests/attach/finished.rs`, `tests/rerun/delayed.rs`, and `tests/rerun/cancel.rs` still hold those cases. `two_reviews_of_one_url_produce_one_run` passed. |
| T20 | A failed post-agent head recheck keeps the successful draft, the start head, and a warning; reused and attached results use the same freshness. | ✓ VERIFIED | `runner/run.rs` pushes `could not re-check the PR head` and does not fail the run. `render.rs` compares the stored `known_head` with the run head for every printed result. `push_during_run_makes_the_draft_stale` passed. |
| T21 | Watch takes the lock, recovers, runs due review jobs in id order, and prints UTC event lines without findings; idle or lock-wait shutdown does not change jobs. | ✓ VERIFIED | `watch/run.rs` still calls `next_runnable_job` and `execute_review_job`. `events.rs` prints `utc(now)` plus a sanitized line. `pub(crate) use date::utc` supplies `fn utc`. `sigterm_during_an_automatic_run_requeues_it` passed on this loop. |
| T22 | `debug enqueue` is hidden from help and only runs the shared enqueue/attach write. It does not read config, call GitHub, take the lock, or start a run. | ✓ VERIFIED | `debug.rs` parses the URL, opens the store, and calls `enqueue_review` only. `main.rs` does not load config for Debug. `tests/watch.rs` still has `debug_is_hidden_from_help`; `tests/watch/debug.rs` still covers the no-lock, no-config path. |
| T23 | All crash points are behind the `crash-points` feature; `mise run build` does not enable it; `mise run test` and `mise run coverage` do; CI ubuntu and macos run coverage. | ✓ VERIFIED | `crash_point!` and `reached` are `cfg(feature = "crash-points")`. `Cargo.toml` has an empty `crash-points` feature and `required-features` on the crash integration target. `mise.toml` build is `cargo build --locked --release`; test and coverage pass `--features crash-points`. `.github/workflows/ci.yml` runs `mise run coverage` on ubuntu-latest and macos-latest. The in-process `code_of` map only names the two publish points; Phase 2 tests use `DIFFMATE_CRASH_POINT`, and `matrix::agent_running` passed that way. The release binary was not rebuilt in this pass. |
| T24 | Recovery removes only its own run directories and its own clone worktree metadata, reports unknown entries, and leaves them. | ✓ VERIFIED | `scan.rs` classifies only numeric directories as runs; other names go to `others`. `cleanup.rs` warns `left {name} in the runs directory: not a run` and does not delete them. `DirEntry::file_type` does not follow a symlink. `src/recovery.rs` and `src/recovery/scan.rs` contain no signal or kill call. |

**Score:** 24/24 truths verified (0 present, behavior-unverified)

Roadmap criteria 1–11 are T01–T11. T12–T24 keep the plan must-haves that are not already those criteria. Plan truths that named a terminal user_version of 2, or a watch that never calls GitHub, are recorded under later-phase extensions. They are not failed Phase 2 truths and they are not new Phase 2 work.

### Later-phase extensions

These are present in the tree. They do not reopen Phase 2 and they are not copied in as new Phase 2 deliverables.

- Schema user_version is 5. Versions 3 and 4 migrate. Only a version above 5 is refused. Phase 1 rows, the v2 columns, and the v1 backup still hold (`phase_one_database_is_upgraded_and_kept`).
- `watch` now polls GitHub and can run publish, reconcile, and sync jobs. Plan 02-09 said polling belongs to Phase 3. REL-08 still holds on the current loop.
- `Failure::TransientAgent` can requeue an automatic `agent_error` inside the same `max_retries` budget. Auth and other agent errors still fail immediately (`recovery/rules/tests/transient.rs`). Crash, timeout, shutdown, and user_cancel rules are unchanged. The crash-every-time budget test passed.

### Advisory (New Scope, Unevidenced)

None. The previous report had no `gaps:` list. No new blocker was found in the Phase 2 paths, and no unevidenced architectural finding is being promoted.

### Required Artifacts

`verify.artifacts` checked 34 declarations. 29 passed their declared pattern. Five pattern misses are the same splits as before; each symbol is compiled through a `mod` and is used.

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `src/store/migrate.rs`, `migrate_v2.sql`, `tests/migration.rs` | v2 step, columns, Phase 1 upgrade | ✓ VERIFIED | `Store::open` calls `migrate::run`. Upgrade test passed through to user_version 5. |
| `src/config.rs`, `src/config/raw.rs`, `tests/config.rs` | Settings, deny unknown fields, hard errors | ✓ VERIFIED | Loaded from `main.rs` before review/watch/tui. Invalid-config test passed. |
| `src/cancel.rs`, `src/review/signals.rs`, `tests/signals.rs` | Stop reasons stored before cancel | ✓ VERIFIED | `set_stop_reason` then `crash_point!("stop-stored")` then `send_replace`. Fault-injection test passed. |
| `src/status/freshness.rs`, `budget.rs`, `when.rs`, `tests/status_freshness.rs` | Live head and retry text | ✓ VERIFIED | `status.rs` calls `pr_view` and `freshness::describe` / `budget::describe`. |
| `src/recovery/rules.rs`, `src/store/retry.rs`, `src/recovery/tests.rs` | Shared job rules and due selection | ✓ VERIFIED | `next_runnable_job` is in `src/store/retry/queue.rs` (`mod queue`). The budget test is in `src/recovery/tests/budget.rs` (`mod budget`). |
| `src/store/jobs/attach.rs`, `src/store/result.rs`, `src/review/wait.rs`, `tests/attach.rs`, `tests/dedup.rs` | Attach, reuse, waiting review | ✓ VERIFIED | Links matched. Dedup and two-review tests passed. |
| `src/store/targets.rs`, `tests/stale_review.rs` | `set_known_head` and stale review output | ✓ VERIFIED | Both runner call sites write `head_sha` / `head_checked_at`. Push and base-move tests passed. |
| `src/runner/agent_run/deadline.rs`, `src/runner/agent_run/tests.rs` | Deadline and paused-time outcome | ✓ VERIFIED | `with_deadline` and `persist_timeout`. Timeout test passed. |
| `src/watch/run.rs`, `src/watch/events.rs`, `src/debug.rs`, `tests/watch.rs`, `tests/rerun.rs` | Watch loop, UTC lines, debug enqueue, REL-10 | ✓ VERIFIED | `fn utc` is in `src/watch/events/date.rs` and re-exported. SIGTERM test is in `tests/watch/shutdown.rs` (`mod shutdown`) and passed. |
| `src/crash.rs`, `tests/crash/matrix.rs`, `tests/crash/budget.rs`, `tests/crash/stop_stored.rs` | Feature-only crash points and kept causes | ✓ VERIFIED | `stop-stored` is the crash point in `review/signals.rs` and the wait string in `tests/crash/process.rs`, which both `stop_stored` tests call. Both passed. |

### Key Link Verification

`verify.key-links` returned 23/23, exit 0 on the positive plans and exit 1 only where an artifact pattern missed. Every declared link was `verified: true` (`Pattern found`).

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `src/store.rs` | `src/store/migrate.rs` | `migrate::run` before open returns | WIRED | Called from `Store::open`. |
| `src/store/readonly.rs` | `src/error.rs` | version 1 is `SchemaOutdated` and is not upgraded | WIRED | Range is `1..SCHEMA_VERSION` (5). Status test passed. |
| `src/main.rs` | `src/config.rs` | `config::load` before review/watch/tui | WIRED | Status and debug are outside the match. |
| `src/app.rs` | `src/config.rs` | `AppConfig.settings` | WIRED | Pattern found. |
| `src/review/signals.rs` | `src/store/jobs.rs` | `set_stop_reason` before `send_replace` | WIRED | Fault test passed. |
| `src/runner/outcome.rs` | `src/store/jobs.rs` and `src/recovery/rules.rs` | stored reason, then `after_failure` | WIRED | Cancelled outcome reads `stop_reason`. |
| `src/store/recovery.rs` | `src/recovery/rules.rs` | recovery uses `after_failure` | WIRED | Pattern found. Cause-survival tests passed. |
| `src/review.rs` | `src/config.rs` and `src/store/jobs/attach.rs` | settings and `enqueue_review` | WIRED | Review runs only the attached job. |
| `src/status.rs` | `src/gh.rs`, `src/store/readonly.rs` | one `pr_view` per draft, read-only store | WIRED | No `pr_view` when `draft_head` is absent. |
| `src/runner.rs` | `src/store/result.rs`, `src/store/targets.rs`, `src/crash.rs` | reuse, `set_known_head`, `crash_point!` | WIRED | Dedup and crash tests passed. |
| `src/review/wait.rs` | `src/store/result.rs`, `src/lock.rs` | stored result without the lock; holder role | WIRED | Patterns found. |
| `src/render.rs` | `src/runner/result.rs` | first line compares `known_head` with `head_sha` | WIRED | Stale and base-move tests passed. |
| `src/runner/agent_run.rs` | `src/runner/agent_run/deadline.rs` | `with_deadline(settings.run_timeout())` | WIRED | Timeout test passed. |
| `src/runner/agent_run/deadline.rs` | `src/store/runs.rs` | `set_stop_reason_for_run` before the merged cancel | WIRED | `persist_timeout` is the `on_timeout` callback. |
| `src/watch/run.rs` | `src/runner.rs` | `execute_review_job` under shutdown | WIRED | SIGTERM requeue test passed. |
| `src/debug.rs` | `src/store/jobs/attach.rs` | same `enqueue_review` | WIRED | No gh, lock, or config load in `enqueue`. |
| `mise.toml` | `Cargo.toml` | test and coverage use `--features crash-points` | WIRED | Build task does not. CI runs coverage on both OS images. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `src/render.rs` | stale suffix | `targets.head_sha` written by `set_known_head` from `gh pr view`, compared with `runs.head_sha` | Yes | ✓ FLOWING |
| `src/status.rs` | freshness cell | latest successful draft head from SQL, plus one live `gh.pr_view`; failure uses stored head and `head_checked_at` | Yes | ✓ FLOWING |
| `src/status/budget.rs` | retry / requeued / exhausted | `jobs.retry_count`, `max_retries`, `next_attempt_at` | Yes | ✓ FLOWING |
| `src/watch/events.rs` | event line | `ReviewResult` / job end plus `utc(now_ms())` | Yes | ✓ FLOWING |
| `src/normalize.rs` | `previous_finding_id` | agent output checked against previous-context ids; unknown becomes NULL plus a warning | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

Three narrow `mise run --force test` filters, not the full suite. Nextest run IDs `38e97efa-5ef2-4fc6-8771-430ce045df59` (5 passed), `5f6c93cb-0c80-4bae-a457-eb2b29d4f3fa` (9 passed), and `d72117c2-4d0a-4372-a997-41263919fbac` (6 passed). Each command exited 0. Skipped counts are the filter, not new `#[ignore]` tests.

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Timeout stores `timeout` and cleans up | `a_run_over_the_timeout_is_failed_timeout` | passed | ✓ PASS |
| Durable stop write before cancel | `a_failed_signal_write_does_not_cancel_until_persisted` | passed | ✓ PASS |
| Stored causes survive a second recovery | `stored_causes_survive_recovery_and_a_second_recovery_is_inert` | passed | ✓ PASS |
| Automatic retry keeps the pinned revision | `automatic_retries_keep_the_stored_revision_when_the_pr_head_moves` | passed | ✓ PASS |
| SIGTERM during watch requeues without the budget | `sigterm_during_an_automatic_run_requeues_it` | passed | ✓ PASS |
| Kill at agent-running leaves one clean restart | `matrix::agent_running` | passed | ✓ PASS |
| Crash budget across watch restarts | `crash_every_time_exhausts_the_budget_across_watch_restarts` | passed | ✓ PASS |
| stop-stored shutdown and user_cancel | both `tests/crash/stop_stored.rs` tests | passed | ✓ PASS |
| Push stales; base move does not | `push_during_run_makes_the_draft_stale`, `base_move_during_run_is_not_stale`, `a_moved_base_branch_keeps_the_draft_current` | passed | ✓ PASS |
| Reuse, force, one run for two reviews, attached re-run only | `same_head_reuses_the_run`, `force_makes_a_new_run`, `two_reviews_of_one_url_produce_one_run`, `review_runs_only_the_rerun_it_attached_to` | passed | ✓ PASS |
| Unknown previous id nulled | `unknown_previous_finding_id_is_nulled` | passed | ✓ PASS |
| Phase 1 upgrade keeps rows; status does not migrate v1 | `phase_one_database_is_upgraded_and_kept`, `status_on_phase_one_database_asks_for_an_upgrade` | passed | ✓ PASS |
| Invalid config stops before enqueue | `invalid_config_stops_review_before_enqueue` | passed | ✓ PASS |

Not run this pass: the full suite, `mise run coverage`, lint, `mise run build`, `mise run test:e2e`, and any live `gh` or Claude call. The release binary was not rebuilt, so crash-point absence was checked from `cfg` and the mise/CI tasks, not from a fresh `strings` pass.

### Probe Execution

N/A. No `scripts/*/tests/probe-*.sh` is declared by the phase plans. The ignored live tests in `src/tests/live/` are the blocked external gate, not a substitute probe.

### Requirements Coverage

Every REL-01–REL-13 id appears in at least one plan `requirements:` field. REQUIREMENTS.md maps those same 13 ids to Phase 2 and no others. Orphaned ids: 0. `SATISFIED` here means the implementation and the named tests. It does not mark the requirement or the phase Complete. REQUIREMENTS.md still shows REL-01, REL-06, and REL-07 unchecked; that tracker was not edited.

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| REL-01 | 02-10 | Kill at any step, no duplicates or orphan worktrees, `interrupted: crash` | ✓ SATISFIED | T01, T23, T24. `matrix::agent_running` passed. UAT-09 is pass. |
| REL-02 | 02-04, 02-07 | Push during a run makes the draft stale | ✓ SATISFIED | T02, T17, T20. Push test passed. UAT-04 is pass. |
| REL-03 | 02-04, 02-07 | Base move alone does not stale the draft | ✓ SATISFIED | T03. Both base-move tests passed. UAT-02 is pass. |
| REL-04 | 02-06 | Same head reuses the run unless `--force` | ✓ SATISFIED | T04. Reuse and force tests passed. UAT-03 is pass. |
| REL-05 | 02-06 | Invalid `previous_finding_id` is nulled; the run succeeds | ✓ SATISFIED | T05. Test passed. |
| REL-06 | 02-01, 02-04, 02-05, 02-10 | Crash retries stay inside the budget across restarts | ✓ SATISFIED | T06, T18. Budget test passed. UAT-08 is pass. |
| REL-07 | 02-01, 02-03, 02-05, 02-10 | Stored shutdown and user_cancel survive unfinished cleanup | ✓ SATISFIED | T07, T14, T16. stop-stored and recovery tests passed. |
| REL-08 | 02-09 | SIGTERM to watch requeues an automatic job without the budget | ✓ SATISFIED | T08, T21. SIGTERM test passed. UAT-06 is pass. |
| REL-09 | 02-03 | Ctrl-C records `user_cancel`, not `agent_error`, and stops the group | ✓ SATISFIED | T09, T16. UAT-01 is pass. |
| REL-10 | 02-09 | A review runs only the re-run it attached to | ✓ SATISFIED | T10, T19, T22. Rerun test passed. UAT-07 is pass. |
| REL-11 | 02-06, 02-09 | Two reviews share one run; force can create a second | ✓ SATISFIED | T11, T19. Two-review test passed. UAT-03 is pass. |
| REL-12 | 02-08 | Run timeout is `failed (timeout)` and stops the agent group | ✓ SATISFIED | T12. Timeout test passed. UAT-05 is pass. |
| REL-13 | 02-02, 02-08, 02-09 | config.toml defaults, hard errors, timeout and `max_retries` | ✓ SATISFIED | T13. Invalid-config test passed. Defaults 15 and 2 are in `config.rs`. |

### Decision Coverage

`check.decision-coverage-verify` exited 0: 16/16 trackable CONTEXT.md decisions honored, `not_honored: []`, `blocking: false`. Message: "All trackable CONTEXT.md decisions are honored by shipped artifacts." This does not close the live gate.

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| `tests/migration.rs` | REL-07 prerequisite | Active | 0 | No; snapshot is taken before upgrade | Value: old rows, v2 defaults, backup user_version 1, final user_version 5 | Sufficient |
| `tests/config.rs` plus config unit tests | REL-13 | Active | 0 | No | Value: defaults, one-line errors, no enqueue | Sufficient |
| `tests/crash/*` | REL-01, REL-06, REL-07 | Active under `crash-points` | 0 `#[ignore]` | No | Behavioral: SIGKILL, SQL counts, worktree metadata | Sufficient for the points exercised; one matrix point re-run, the other cases still present |
| `tests/signals.rs`, `tests/watch/shutdown.rs` | REL-08, REL-09 | Active | 0 | No | Behavioral: process group, cause, requeue | Sufficient |
| `tests/stale_review.rs`, `tests/dedup.rs`, `tests/attach.rs`, `tests/rerun.rs`, `tests/review_output.rs` | REL-02–REL-05, REL-10, REL-11 | Active | 0 | No | Behavioral: CLI output, call counts, stored rows | Sufficient |
| `src/tests/live/agent.rs`, `src/tests/live/prs.rs` | Final live gate, not the only REL proof | 4 `#[ignore]` model/auth cases | Ignored in `mise run test` | No | External model result | Not proof. LIVE-01 and LIVE-02 stay blocked |

**Disabled tests on requirements:** the four ignored live tests are not the only tests for REL-01–REL-13. They remain an open external gate, not an implementation blocker.
**Circular patterns detected:** 0 in the Phase 2 oracles inspected. Expected heads, causes, and deadlines are fixtures or clock pauses, not output rewritten by the same run.
**Insufficient assertions:** 0 for the requirements above.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `src/`, `tests/` | — | `TBD`, `FIXME`, `XXX`, `todo!`, `unimplemented!` | — | None found |

Empty defaults in config and recovery are overwritten by parsed settings or by `after_failure`. They are not stubs.

### Prohibitions

Thirteen test-tier prohibitions still have negative tests in the tree (migration preservation, invalid config, stale versus current, read-only status, user_cancel not retried, bounded budget, transparent reuse, attach not mutating tracking, stale warning, timeout not success, debug enqueue not running, release feature cfg, cleanup staying on owned paths). The two judgment-tier prohibitions are no longer pending:

| Plan | Statement | Status | Evidence |
| --- | --- | --- | --- |
| 02-03 | Do not signal any PID or PGID read from SQLite | Resolved by owner | UAT test 13 pass. `supervise` uses `child.id()`. Recovery scan and cleanup do not signal. |
| 02-09 | Watch must not poll or create jobs on its own in this phase | Resolved by owner for the Phase 2 boundary | UAT test 14 pass. The prohibition names Phase 3 as the owner of polling. Current `watch/poll.rs` is that later work. REL-08 was re-tested on the current loop. |

### Human Verification Required

UAT tests 1–10 and 13–14 are pass in `02-UAT.md` and are not repeated here. Two live checks remain blocked. They are why status is not `passed`. No Claude API call, live `gh` call, or `mise run test:e2e` was run.

### 1. LIVE-01

**Test:** After the owner separately restores Anthropic access, rerun `isolation_flags_keep_authentication`.
**Expected:** Real Claude runs with the production isolation flags and does not return HTTP 403.
**Why human:** External account-verification barrier. The owner chose to keep the live gate blocked. Local fakes do not prove it.

### 2. LIVE-02

**Test:** After account access is restored, run the four remaining model-backed cases with `mise run test:e2e`.
**Expected:** Max-turns is failed, a clean PR is `no_findings`, and the bug and fork PRs produce the expected finding at `calc.py:7`.
**Why human:** Those four cases were not run after auth failed fast. Local fakes do not prove the model result.

### Gaps Summary

No implementation gap. The Phase 2 goal holds in the current tree: a review can be killed, interrupted, repeated, or overtaken by a push without a duplicate run, an orphaned worktree, a lost stop cause, or an unbounded retry, and the draft reports whether it still matches the PR head.

The phase is not accepted. `status` is `human_needed` because LIVE-01 and LIVE-02 are still blocked on Anthropic HTTP 403. That block is not a waiver. `passed` is not valid while those checks are unmet. No code gap-closure plan is required.

---

_Verified: 2026-11-12T10:02:32Z_
_Verifier: Claude (gsd-verifier)_
