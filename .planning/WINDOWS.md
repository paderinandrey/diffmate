---
schema_version: 1
open_count: 4
waived_count: 0
fixed_count: 0
total_count: 4
last_updated: 2026-11-03T13:41:54.559Z
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
  }
]
````
