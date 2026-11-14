---
schema_version: 1
open_count: 15
waived_count: 0
fixed_count: 3
total_count: 18
last_updated: 2026-11-14T18:27:19.000Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 02 | unrun-verify | .planning/phases/02-reliability/02-03-PLAN.md |  | Real Claude Ctrl-C hand check is deferred to end-of-phase UAT; isolated fake-process signal tests pass. | open |  | 2026-11-01T00:15:09.889Z |  |
| 2 | 02 | unrun-verify | .planning/phases/02-reliability/02-04-PLAN.md |  | Release-binary real GitHub base-branch advance hand check is deferred to end-of-phase UAT; isolated base-only push and live-head freshness tests pass. | open |  | 2026-11-03T12:39:21.370Z |  |
| 3 | 02 | unrun-verify | .planning/phases/02-reliability/02-06-PLAN.md |  | Release-binary real CLI reuse, force and concurrent attachment hand check is deferred to end-of-phase UAT; isolated integration and property tests pass. | open |  | 2026-11-03 |  |
| 4 | 02 | unrun-verify | .planning/phases/02-reliability/02-07-PLAN.md |  | Release-binary real GitHub push-during-review hand check is deferred to end-of-phase UAT; isolated FIFO push, stored-result and status freshness tests pass. | open |  | 2026-11-03T13:41:54.559Z |  |
| 5 | 02 | unrun-verify | .planning/phases/02-reliability/02-08-PLAN.md |  | Release-binary real Claude one-minute timeout hand check is deferred to end-of-phase UAT; paused-time runner, durable-reason and shared process-group signal checks pass. | open |  | 2026-11-03T14:03:39.652Z |  |
| 6 | 02 | deviation | src/review/signals.rs | 26 | 02-09 must fix signal cancellation after failed stop-reason persistence; the existing path warns and cancels without a durable reason, contrary to SPEC Agent contract. Timeout persistence in 02-08 retries before cancellation. | fixed |  | 2026-11-03T14:03:40.124Z | 2026-11-03T14:25:24.172Z |
| 7 | 02 | unrun-verify | .planning/phases/02-reliability/02-09-PLAN.md |  | Release-binary real Claude watch SIGTERM, status and restart hand check is deferred to end-of-phase UAT; isolated signal, requeue, process-group cleanup and restart checks pass. | open |  | 2026-11-03T14:30:17.120Z |  |
| 8 | 02 | unrun-verify | .planning/phases/02-reliability/02-09-PLAN.md |  | Release-binary real CLI attach-to-rerun and unrelated automatic backlog hand check is deferred to end-of-phase UAT; isolated attachment, backoff, cancellation and backlog checks pass. | open |  | 2026-11-03T14:30:17.602Z |  |
| 9 | 02 | unrun-verify | .planning/phases/02-reliability/02-10-PLAN.md |  | Release-binary live SIGKILL retry-budget hand check is deferred to end-of-phase UAT; the feature-build crash-every-time test proves three attempts across watch restarts and persisted status budgets. | open |  | 2026-11-03T15:01:08.016Z |  |
| 10 | 02 | unrun-verify | .planning/phases/02-reliability/02-10-PLAN.md |  | Release-binary live SIGKILL during preparation and agent execution is deferred to end-of-phase UAT; all named crash points pass isolated restart, database, directory and worktree-metadata checks. | open |  | 2026-11-03T15:01:08.489Z |  |
| 11 | 02 | unrun-verify | src/tests/isolation/auth.rs |  | Grok live check: untrusted_worktree_canary_is_not_loaded. grok inspect on an untrusted worktree must omit its AGENTS.md and CLAUDE.md, report Project trusted: no, and show claude and cursor rules OFF via the production env. The production run must still reach the model. Passed 2026-11-14 in 31.285s. | fixed |  | 2026-11-14T16:00:00.000Z | 2026-11-14T18:27:19.000Z |
| 12 | 02 | unrun-verify | src/tests/live |  | Grok live check: live_max_turns_one_is_failed_not_no_findings is failed agent_error with subtype error_max_turns, live_clean_pr_has_no_findings is no_findings, and the bug and fork PR tests find the expected calc.py line. Passed 2026-11-14. | fixed |  | 2026-11-14T16:00:00.000Z | 2026-11-14T18:27:19.000Z |
| 13 | 04 | deviation | src/tui/tests/edit.rs |  | Editor tests live in src/tui/tests/edit.rs because decisions.rs exceeded the 100-insertion hook. | open |  | 2026-11-10T15:46:56.770Z |  |
| 14 | 04 | deviation | src/tui/model.rs |  | Accepting Discard reloads queue rows after Store::discard so the latched target leaves the list. | open |  | 2026-11-10T17:10:18.894Z |  |
| 15 | 04 | deviation | src/tui/tests/actions.rs |  | Task 2 has no separate test(04-04) RED commit. hk runs the full suite, so the failing tests and the implementation landed together in 9f1a62a. | open |  | 2026-11-10T17:10:19.312Z |  |
| 16 | 04 | deviation | src/publish/tests/script.rs |  | Script answers pr_view, current_login, and list_my_reviews so the publish gate does not treat the trait default as a transient gh error | open |  | 2026-11-10T18:38:59.087Z |  |
| 17 | 04 | deviation | src/publish/tests/crash/gh.rs | 59 | Crash remote create_review prefers a sending publication when more than one row exists | open |  | 2026-11-11T08:48:39.857Z |  |
| 18 | 04 | deviation | src/publish/tests/crash/keys.rs | 160 | Continuation test calls crate::tui::serve because the runner module is private | open |  | 2026-11-11T08:48:40.537Z |  |

````json
[
  {
    "id": 1,
    "kind": "unrun-verify",
    "phase": "02",
    "file": ".planning/phases/02-reliability/02-03-PLAN.md",
    "line": null,
    "description": "Real Claude Ctrl-C hand check is deferred to end-of-phase UAT; isolated fake-process signal tests pass.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-01T00:15:09.889Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 2,
    "kind": "unrun-verify",
    "phase": "02",
    "file": ".planning/phases/02-reliability/02-04-PLAN.md",
    "line": null,
    "description": "Release-binary real GitHub base-branch advance hand check is deferred to end-of-phase UAT; isolated base-only push and live-head freshness tests pass.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-03T12:39:21.370Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 3,
    "kind": "unrun-verify",
    "phase": "02",
    "file": ".planning/phases/02-reliability/02-06-PLAN.md",
    "line": null,
    "description": "Release-binary real CLI reuse, force and concurrent attachment hand check is deferred to end-of-phase UAT; isolated integration and property tests pass.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-03",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 4,
    "kind": "unrun-verify",
    "phase": "02",
    "file": ".planning/phases/02-reliability/02-07-PLAN.md",
    "line": null,
    "description": "Release-binary real GitHub push-during-review hand check is deferred to end-of-phase UAT; isolated FIFO push, stored-result and status freshness tests pass.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-03T13:41:54.559Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 5,
    "kind": "unrun-verify",
    "phase": "02",
    "file": ".planning/phases/02-reliability/02-08-PLAN.md",
    "line": null,
    "description": "Release-binary real Claude one-minute timeout hand check is deferred to end-of-phase UAT; paused-time runner, durable-reason and shared process-group signal checks pass.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-03T14:03:39.652Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 6,
    "kind": "deviation",
    "phase": "02",
    "file": "src/review/signals.rs",
    "line": 26,
    "description": "02-09 must fix signal cancellation after failed stop-reason persistence; the existing path warns and cancels without a durable reason, contrary to SPEC Agent contract. Timeout persistence in 02-08 retries before cancellation.",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-11-03T14:03:40.124Z",
    "resolved_at": "2026-11-03T14:25:24.172Z",
    "milestone": null
  },
  {
    "id": 7,
    "kind": "unrun-verify",
    "phase": "02",
    "file": ".planning/phases/02-reliability/02-09-PLAN.md",
    "line": null,
    "description": "Release-binary real Claude watch SIGTERM, status and restart hand check is deferred to end-of-phase UAT; isolated signal, requeue, process-group cleanup and restart checks pass.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-03T14:30:17.120Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 8,
    "kind": "unrun-verify",
    "phase": "02",
    "file": ".planning/phases/02-reliability/02-09-PLAN.md",
    "line": null,
    "description": "Release-binary real CLI attach-to-rerun and unrelated automatic backlog hand check is deferred to end-of-phase UAT; isolated attachment, backoff, cancellation and backlog checks pass.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-03T14:30:17.602Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 9,
    "kind": "unrun-verify",
    "phase": "02",
    "file": ".planning/phases/02-reliability/02-10-PLAN.md",
    "line": null,
    "description": "Release-binary live SIGKILL retry-budget hand check is deferred to end-of-phase UAT; the feature-build crash-every-time test proves three attempts across watch restarts and persisted status budgets.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-03T15:01:08.016Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 10,
    "kind": "unrun-verify",
    "phase": "02",
    "file": ".planning/phases/02-reliability/02-10-PLAN.md",
    "line": null,
    "description": "Release-binary live SIGKILL during preparation and agent execution is deferred to end-of-phase UAT; all named crash points pass isolated restart, database, directory and worktree-metadata checks.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-03T15:01:08.489Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 11,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/tests/isolation/auth.rs",
    "line": null,
    "description": "Grok live check: untrusted_worktree_canary_is_not_loaded. grok inspect on an untrusted worktree must omit its AGENTS.md and CLAUDE.md, report Project trusted: no, and show claude and cursor rules OFF via the production env. The production run must still reach the model. Passed 2026-11-14 in 31.285s.",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-11-14T16:00:00.000Z",
    "resolved_at": "2026-11-14T18:27:19.000Z",
    "milestone": null
  },
  {
    "id": 12,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/tests/live",
    "line": null,
    "description": "Grok live check: live_max_turns_one_is_failed_not_no_findings is failed agent_error with subtype error_max_turns, live_clean_pr_has_no_findings is no_findings, and the bug and fork PR tests find the expected calc.py line. Passed 2026-11-14.",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-11-14T16:00:00.000Z",
    "resolved_at": "2026-11-14T18:27:19.000Z",
    "milestone": null
  },
  {
    "id": 13,
    "kind": "deviation",
    "phase": "04",
    "file": "src/tui/tests/edit.rs",
    "line": null,
    "description": "Editor tests live in src/tui/tests/edit.rs because decisions.rs exceeded the 100-insertion hook.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-10T15:46:56.770Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 14,
    "kind": "deviation",
    "phase": "04",
    "file": "src/tui/model.rs",
    "line": null,
    "description": "Accepting Discard reloads queue rows after Store::discard so the latched target leaves the list.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-10T17:10:18.894Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 15,
    "kind": "deviation",
    "phase": "04",
    "file": "src/tui/tests/actions.rs",
    "line": null,
    "description": "Task 2 has no separate test(04-04) RED commit. hk runs the full suite, so the failing tests and the implementation landed together in 9f1a62a.",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-10T17:10:19.312Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 16,
    "kind": "deviation",
    "phase": "04",
    "file": "src/publish/tests/script.rs",
    "line": null,
    "description": "Script answers pr_view, current_login, and list_my_reviews so the publish gate does not treat the trait default as a transient gh error",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-10T18:38:59.087Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 17,
    "kind": "deviation",
    "phase": "04",
    "file": "src/publish/tests/crash/gh.rs",
    "line": 59,
    "description": "Crash remote create_review prefers a sending publication when more than one row exists",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-11T08:48:39.857Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 18,
    "kind": "deviation",
    "phase": "04",
    "file": "src/publish/tests/crash/keys.rs",
    "line": 160,
    "description": "Continuation test calls crate::tui::serve because the runner module is private",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-11T08:48:40.537Z",
    "resolved_at": null,
    "milestone": null
  }
]
````
