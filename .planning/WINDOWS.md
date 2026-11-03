---
schema_version: 1
open_count: 7
waived_count: 0
fixed_count: 1
total_count: 8
last_updated: 2026-11-03T14:30:17.602Z
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
  }
]
````
