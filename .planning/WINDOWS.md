---
schema_version: 1
open_count: 1
waived_count: 0
fixed_count: 0
total_count: 1
last_updated: 2026-11-01T00:15:09.889Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 02 | unrun-verify | .planning/phases/02-reliability/02-03-PLAN.md |  | Real Claude Ctrl-C hand check is deferred to end-of-phase UAT; isolated fake-process signal tests pass. | open |  | 2026-11-01T00:15:09.889Z |  |

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
  }
]
````
