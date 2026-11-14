---
phase: 03-queue
plan: 05
subsystem: queue
tags: [debug, discard, watch, sqlite, rust]

requires:
  - phase: 03-queue
    provides: "Store::discard sets tracking untracked and request_seen seen, and watch applies the re-request rule"
provides:
  - "Hidden diffmate debug discard URL calls Store::discard, prints discarded OWNER/REPO#N, and exits 0"
  - "A discarded target stays untracked while search still returns it, and becomes tracked with one delayed automatic job only after the request leaves and comes back"
affects: [03-queue, tui]
actuals:
  tokens: 1792
  tasks: 2
  commits: 2
plan_head_before: 00d2af84ebcab2dd6c86baa47e09549b33f450c3
plan_head_after: a21eb54a300a9ec86d919f76091720275f07ed5f
commits: 2
tech-stack:
  added: []
  patterns:
    - "debug discard parses with pr_url::parse, opens the store, and calls Store::discard once"
    - "Invalid URLs use the same stderr line and exit 2 as debug enqueue"
key-files:
  created:
    - tests/watch/discard.rs
  modified:
    - src/cli.rs
    - src/debug.rs
    - src/main.rs
    - src/store/poll.rs
    - tests/watch.rs
key-decisions:
  - "debug discard stays in src/debug.rs and calls Store::discard; discard was not reimplemented"
  - "The not(test) dead_code allow on Store::discard was removed because the binary calls it"
  - "KnownTarget.queued_head keeps its allow; this plan does not read that field"
patterns-established:
  - "Hidden debug commands do not load config.toml, take the runner lock, or call gh"
requirements-completed: [QUE-07]
coverage:
  - id: D1
    description: "diffmate debug discard URL is absent from --help, calls Store::discard, prints discarded o/r#1, and exits 0. An invalid URL exits 2 with the enqueue error text. The command does not call gh, read config.toml, or need the runner lock."
    requirement: QUE-07
    verification:
      - kind: integration
        ref: "tests/watch/discard.rs#discard_marks_the_row_untracked_without_gh_config_or_the_lock"
        status: pass
      - kind: integration
        ref: "tests/watch/discard.rs#discard_rejects_the_same_invalid_url_as_enqueue"
        status: pass
      - kind: integration
        ref: "tests/watch/discard.rs#debug_stays_hidden_when_discard_exists"
        status: pass
    human_judgment: false
  - id: D2
    description: "A target discarded while search still returns it stays untracked with request_seen 1 and no queued automatic job. After an empty search with the PR still open, a later search hit tracks it and arms one delayed automatic job."
    requirement: QUE-07
    verification:
      - kind: integration
        ref: "tests/watch/discard.rs#a_still_visible_request_stays_untracked_until_it_returns"
        status: pass
    human_judgment: false

duration: 10min
completed: 2026-11-09
status: complete
---

# Phase 3 Plan 05: Hidden debug discard Summary

**Hidden `diffmate debug discard` calls `Store::discard`, and a running watch tracks the PR again only after the request leaves and returns**

## Performance

- **Duration:** 10 min
- **Started:** 2026-11-09T12:43:26Z
- **Completed:** 2026-11-09T12:53:05Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- `diffmate debug discard URL` is hidden with `debug enqueue`, prints `discarded OWNER/REPO#N`, and exits 0
- An invalid URL exits 2 with the same stderr as `debug enqueue`, and the command makes no gh call
- A still-visible request stays untracked; an empty poll then a later hit arms one delayed automatic job

## Task Commits

Each task was committed atomically:

1. **Task 1: Hidden debug discard calls Store::discard** - `5a93690` (feat)
2. **Task 2: A still-visible request does not return, and a real re-request does** - `a21eb54` (test)

## Files Created/Modified

- `src/cli.rs` - `DebugCommand::Discard { url }` under the hidden `debug` command
- `src/main.rs` - Dispatch discard without loading `config.toml`
- `src/debug.rs` - Parse the URL, call `Store::discard` once, print the label
- `src/store/poll.rs` - Drop the unused `dead_code` allow on `Store::discard`
- `tests/watch.rs` - Declare `tests/watch/discard.rs`
- `tests/watch/discard.rs` - Command behavior and the watch re-request sequence

## Decisions Made

- Discard stays a call to `Store::discard`. The store method was not rewritten, and no third tracking value was added.
- The function fits in `src/debug.rs` (25 lines added), so it was not moved to `src/debug/discard.rs`.
- `KnownTarget.queued_head` is still unread outside tests, so its `dead_code` allow stays.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 4 can bind the TUI discard key to `Store::discard`. `debug enqueue` is unchanged and still hidden. `.planning/STATE.md`, `.planning/ROADMAP.md`, and `.planning/REQUIREMENTS.md` were not updated; Phase 2 UAT stays open.

---
*Phase: 03-queue*
*Completed: 2026-11-09*

## Self-Check: PASSED

- FOUND: src/cli.rs, src/debug.rs, src/main.rs, src/store/poll.rs, tests/watch.rs, tests/watch/discard.rs
- FOUND: 5a93690, a21eb54 as ancestors of HEAD
- commits measured from 00d2af84ebcab2dd6c86baa47e09549b33f450c3: 2
- plan_head_after: a21eb54a300a9ec86d919f76091720275f07ed5f
