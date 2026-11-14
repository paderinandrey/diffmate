# Phase 3: Queue - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-11-09
**Phase:** 3-Queue
**Areas discussed:** Notifications, Discard seam, Poll, Request seen and debounce

The owner said to continue. Phase 2 UAT was still open, so these choices were auto-selected from the recommended option. SPEC.md requirements were not re-opened.

---

## Notifications

| Option | Description | Selected |
|--------|-------------|----------|
| osascript | `display notification` through tokio, no new crate | ✓ |
| notify-rust | A crate that wraps UNUserNotificationCenter | |

**User's choice:** auto-selected osascript
**Notes:** SPEC.md requires a title and a body, including the failure reason. It does not require a click action. A new crate needs a concrete need this does not have.

## Discard seam

| Option | Description | Selected |
|--------|-------------|----------|
| debug discard | Hidden command calling one store method, same shape as debug enqueue | ✓ |
| Public discard command | A supported CLI command before the TUI exists | |

**User's choice:** auto-selected debug discard
**Notes:** ROADMAP.md says the discard key arrives in Phase 4 and this phase needs a test seam.

## Poll

| Option | Description | Selected |
|--------|-------------|----------|
| gh search prs --review-requested=@me | Direct requests only, drafts included and then skipped, paginated | ✓ |
| Search with --draft=false | Drop drafts from the result set | |

**User's choice:** auto-selected the inclusive search
**Notes:** A tracked PR can become a draft later, so draft state is read per target, not only by filtering the search.

## Request seen and debounce

| Option | Description | Selected |
|--------|-------------|----------|
| request_seen column plus next_attempt_at | Schema v3; one delayed automatic job; head resolved at start | ✓ |
| Third tracking value discarded | A new targets.tracking state | |

**User's choice:** auto-selected request_seen
**Notes:** SPEC.md tracking has two values. The re-request gap is request_seen, not a new state.
