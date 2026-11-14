# Phase 2: Reliability - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-27
**Phase:** 2-Reliability
**Areas discussed:** watch/tui stand-ins, head check and freshness, config.toml format, kill-at-any-step proof

---

## watch/tui stand-ins

| Option | Description | Selected |
|--------|-------------|----------|
| Real command, runner only | `diffmate watch` waits for the lock, runs recovery, executes every job, handles SIGTERM/SIGINT; Phase 3 adds polling | ✓ |
| Hidden command | The same loop under a hidden name; `watch` appears in Phase 3 | |
| Tests only | The loop exists only as a tested function | |

| Option | Description | Selected |
|--------|-------------|----------|
| Hidden enqueue command | `diffmate debug enqueue <url>` with automatic and tui-re-run modes through the real enqueue code | ✓ |
| Direct SQL in the instructions | `sqlite3` inserts in the hand check | |
| Automated tests only | No hand check for these scenarios | |

| Option | Description | Selected |
|--------|-------------|----------|
| Timestamped lines on stderr | One line per event, no findings | ✓ |
| JSON lines | One JSON object per event | |
| Planner decides | | |

| Option | Description | Selected |
|--------|-------------|----------|
| Release binary, hidden from --help | Hand checks run on the working binary | ✓ |
| Behind a cargo feature | Separate build for hand checks | |

**User's choice:** the recommended option each time.

---

## Head check and freshness

| Option | Description | Selected |
|--------|-------------|----------|
| End of run + live `status` | Runner checks the head at start and after the agent; `status` checks live per target | ✓ |
| Runner only | `status` offline | |
| End of run + `status --refresh` | Live check only on a flag | |

| Option | Description | Selected |
|--------|-------------|----------|
| No, display only | `status` stays read-only; only the runner stores the head | ✓ |
| Yes, store it | `status` writes the head and check time | |

| Option | Description | Selected |
|--------|-------------|----------|
| Relative time | `checked 3m ago`, `checked just now` | ✓ |
| Absolute local time | `checked 2026-10-27 14:05` | |
| Planner decides | | |

| Option | Description | Selected |
|--------|-------------|----------|
| Warning on the first line, exit 0 | Findings printed as usual | ✓ |
| Separate exit code | Exit 3 for stale | |
| Suggest re-run command | Warning plus a hint | |

**User's choice:** the recommended option each time.

---

## config.toml format

| Option | Description | Selected |
|--------|-------------|----------|
| Integers with the unit in the key name | `run_timeout_minutes = 15`, `debounce_seconds = 60` | ✓ |
| Suffixed strings | `run_timeout = "15m"` | |
| All seconds | `run_timeout = 900` | |

| Option | Description | Selected |
|--------|-------------|----------|
| Hard error | File, line and key named; nothing enqueued | ✓ |
| Warning and default | | |
| Error for values, warning for keys | | |

| Option | Description | Selected |
|--------|-------------|----------|
| Once at process start | `status` does not read it | ✓ |
| Before every job | `watch` re-reads without restart | |

| Option | Description | Selected |
|--------|-------------|----------|
| All four keys, validated | Full SPEC schema now | ✓ |
| Only the two used | | |

| Option | Description | Selected |
|--------|-------------|----------|
| 60 seconds | Default poll interval, recorded in SPEC.md | ✓ |
| 120 seconds | | |
| Decide in Phase 3 | | |

**User's choice:** the recommended option each time. SPEC.md "Configuration" updated with the 60-second default.

---

## Kill-at-any-step proof

| Option | Description | Selected |
|--------|-------------|----------|
| Test crash points + manual kill -9 | Named point per runner step, real SIGKILL, restart and check | ✓ |
| Manual kill -9 only | | |
| Fakes only | In-process simulation, no real SIGKILL | |

| Option | Description | Selected |
|--------|-------------|----------|
| Hook in a test build only | Cargo feature; env var exists only in that build | ✓ |
| External fakes on PATH only | Cannot reach internal steps | |
| Fakes plus hook for internal steps | Two mechanisms | |

| Option | Description | Selected |
|--------|-------------|----------|
| Manual kill -9 on each attempt | Release binary, `debug enqueue` and `watch`, three times | ✓ |
| Test build with the hook | | |
| Automated test only | | |

| Option | Description | Selected |
|--------|-------------|----------|
| Normal `mise run test` and CI | No network or auth needed | ✓ |
| Separate mise task | | |

**User's choice:** the recommended option each time.

---

## Claude's Discretion

- Incremental schema migration v1 to v2 that keeps existing data.
- Wording of the retry budget and next attempt in `status`.
- The wait between SIGTERM and SIGKILL, and the lock retry pause in `watch`.
- How `watch` waits for new jobs when idle.
- The exact list and names of crash points.

## Deferred Ideas

None.
