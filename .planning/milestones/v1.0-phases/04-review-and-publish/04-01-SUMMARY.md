---
phase: 04-review-and-publish
plan: 01
subsystem: tui
tags: [ratatui, sqlite, queue, draft, schema]
requires:
  - phase: 03-queue
    provides: "Persisted targets, succeeded drafts, findings, and status rows ordered by target id"
provides:
  - "user_version 5 with a version-4 snapshot before migrate_v5.sql"
  - "diffmate tui queue, newest target first, that quits without writing"
  - "Draft screen in review print order, opened with Enter and closed with Esc"
  - "diffmate status keeps ascending target id and ends each line with the publication state"
affects: [04-review-and-publish, publish, decisions]
actuals:
  tokens: 17169
  tasks: 2
  commits: 2
plan_head_before: a15a62ce4fc9e5d0e404965acac61b6628953179
plan_head_after: 6c58f32c2ca739c425e3060dd36e41eb3135e5f4
commits: 2
tech-stack:
  added: ["ratatui 0.30.2 (default features off, crossterm feature only)"]
  patterns:
    - "Queue rows use the status facts in reverse target-id order and stored freshness, with no gh call"
    - "Draft findings are partitioned like render::findings: anchored first, then unanchored, ordinal order inside each group"
    - "Finding and queue text pass through render::sanitize before the terminal backend"
key-files:
  created:
    - src/store/migrate_v5.sql
    - src/store/migrate/tests/v5.rs
    - src/store/status/queue.rs
    - src/store/status/draft.rs
    - src/tui.rs
    - src/tui/model.rs
    - src/tui/keys.rs
    - src/tui/draw.rs
    - src/tui/draw/draft.rs
    - src/tui/tests/queue.rs
    - src/tui/tests/draft.rs
    - src/tui/tests/focus.rs
    - tests/tui.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/cli.rs
    - src/main.rs
    - src/lock/role.rs
    - src/store/migrate.rs
    - src/store/status.rs
    - src/status.rs
    - src/status/freshness.rs
key-decisions:
  - "ratatui 0.30.2 is installed with default features off and only the crossterm feature. crossterm is not a direct dependency. toml stays 1.2.1."
patterns-established:
  - "Scripted TUI tests drive TestBackend through run_scripted and never open a real terminal"
  - "Key reads on the real terminal go through spawn_blocking"
requirements-completed: [PUB-01]
coverage:
  - id: D1
    description: "Opening a version 4 database ends at user_version 5 and leaves diffmate.db.v4-backup at user_version 4. A read-only open of version 4 returns SchemaOutdated and creates no backup. Version 6 is refused and not rewritten."
    requirement: PUB-01
    verification:
      - kind: unit
        ref: "src/store/migrate/tests/v5.rs#version_four_is_snapshotted_then_upgraded"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests/v5.rs#read_only_version_four_is_outdated_without_a_backup"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests.rs#newer_version_is_refused_and_left_alone"
        status: pass
    human_judgment: false
  - id: D2
    description: "diffmate tui draws a newest-first queue and q writes neither a publication nor a job. Non-terminal stdout exits 1 without creating a database."
    requirement: PUB-01
    verification:
      - kind: unit
        ref: "src/tui/tests/queue.rs#queue_shows_the_target_and_quit_writes_nothing"
        status: pass
      - kind: unit
        ref: "src/tui/tests/queue.rs#queue_is_newest_first_and_arrows_move_the_selection"
        status: pass
      - kind: integration
        ref: "tests/tui.rs#tui_without_a_terminal_exits_before_creating_a_database"
        status: pass
    human_judgment: false
  - id: D3
    description: "Enter opens the latest succeeded draft with anchored findings before unanchored findings, the selected body, the limitation, and both future notes. Esc restores the queue selection. Down past the last finding focuses future notes. k, d, and e do nothing. q on the draft writes no publication. An untracked target with a publication stays on the queue."
    requirement: PUB-01
    verification:
      - kind: unit
        ref: "src/tui/tests/draft.rs#enter_opens_the_draft_and_esc_restores_the_queue"
        status: pass
      - kind: unit
        ref: "src/tui/tests/focus.rs#future_notes_can_be_focused_and_quit_writes_nothing"
        status: pass
    human_judgment: false
  - id: D4
    description: "diffmate status still prints the lower target id first and ends each line with the publication state."
    requirement: PUB-01
    verification:
      - kind: integration
        ref: "tests/status.rs#status_lists_latest_run_per_target"
        status: pass
      - kind: integration
        ref: "tests/concurrency.rs#second_review_waits_for_the_lock_and_status_works"
        status: pass
    human_judgment: false
duration: 44min
completed: 2026-11-10
status: complete
---

# Phase 4 Plan 1: Queue and draft screen Summary

**Schema version 5 with a version-4 snapshot, a newest-first `diffmate tui` queue, and a draft screen in review print order, on ratatui 0.30.2.**

## Performance

- **Duration:** 44 min
- **Started:** 2026-11-10T13:53:59Z
- **Completed:** 2026-11-10T14:38:13Z
- **Tasks:** 2
- **Files modified:** 35

## Accomplishments

- `Store::open` on user_version 4 snapshots `diffmate.db.v4-backup` and migrates to user_version 5. A read-only open of version 4 does not migrate. Version 6 and newer is refused.
- `diffmate tui` draws the queue newest target first (label, tracking, latest run, stored freshness, publication state) and `q` exits without writing. Non-terminal stdout exits 1 before the database is created.
- Enter opens that target's latest succeeded draft. Anchored findings come before unanchored findings. The limitation and future improvements sit under the text pane and are not decision rows. Esc returns to the same queue selection.
- `diffmate status` still uses `ORDER BY t.id` and appends the publication state as the last field. ratatui 0.30.2 is the TUI crate. crossterm was not added as a direct dependency.

## Task Commits

Each task was committed atomically:

1. **Task 1: Schema version 5 and the queue screen** - `f3c92da` (feat)
2. **Task 2: Draft screen in review print order** - `6c58f32` (feat)

## Files Created/Modified

- `src/store/migrate_v5.sql` - publications, limitation_included, thread columns, jobs.publication_id
- `src/store/migrate.rs` - SCHEMA_VERSION 5 and the version-4 snapshot
- `src/store/status/queue.rs` - `Store::queue_rows`, `ORDER BY t.id DESC`
- `src/store/status/draft.rs` - latest succeeded draft in review print order
- `src/tui.rs` - terminal gate, event loop, scripted seam
- `src/tui/draw/draft.rs` - finding list, text pane, limitation, future notes
- `src/status.rs` - publication state as the last status field
- `Cargo.toml` - ratatui 0.30.2, default features off, crossterm feature only

## Decisions Made

ratatui 0.30.2 (published 2026-06-19, inside `minimum_release_age`) is installed with `default-features = false` and `features = ["crossterm"]`. Default features would have pulled extra backends plus time and calendar. Key events come from `ratatui::crossterm`. crossterm is not a direct crate. `toml` stays 1.2.1. `deny.toml` was not changed. `mise run vuln` reported advisories, bans, licenses, and sources ok.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Status golden strings omitted the new publication field**
- **Found during:** Task 1 (schema version 5 and the queue screen)
- **Issue:** `diffmate status` now ends every line with the publication state. `tests/concurrency.rs` compared the full line, and `tests/recovery.rs` required a newline immediately after the failure text. The hook's full suite failed on the concurrency assertion.
- **Fix:** Expect `  none` as the last field. The status order stays `ORDER BY t.id`.
- **Files modified:** `tests/concurrency.rs`, `tests/recovery.rs`, `tests/status.rs`, `tests/status_freshness.rs`, `tests/status_freshness/failure.rs`
- **Commit:** `f3c92da`

**2. [Rule 3 - Blocking] The draft test would exceed the 100-line hook in queue.rs**
- **Found during:** Task 2 (draft screen)
- **Issue:** hk rejects a staged source file with more than 100 insertions. Putting the draft scripted test in `src/tui/tests/queue.rs` added more than 100 lines.
- **Fix:** Keep the queue helpers in `queue.rs`. Put the draft screen test in `src/tui/tests/draft.rs` and the focus and quit checks in `src/tui/tests/focus.rs`.
- **Files modified:** `src/tui/tests/draft.rs`, `src/tui/tests/focus.rs`, `src/tui/tests/mod.rs`, `src/tui/tests/queue.rs`
- **Commit:** `6c58f32`

## Verification

- Tracer re-run of `mise run --force test -- -E 'test(store::migrate) | test(tui::) | binary(tui) | binary(migration) | binary(status)' --no-tests=fail`: 28 passed.
- Task 2 `mise run --force test -- -E 'test(tui::)' --no-tests=fail` equivalent: 4 passed. The task 2 hook then ran the full suite: 416 passed, 11 skipped.
- `mise run lint` inside both task hooks: rustfmt, clippy `-D warnings`, and cargo-machete passed.
- `mise run vuln`: advisories ok, bans ok, licenses ok, sources ok. cargo-deny printed duplicate-crate warnings for hashbrown and syn.
- Coverage (`mise run coverage`) and `mise run test:e2e` were not run.

## Auth Gates

None.

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: src/store/migrate_v5.sql
- FOUND: src/tui/draw/draft.rs
- FOUND: src/store/status/draft.rs
- FOUND: .planning/phases/04-review-and-publish/04-01-SUMMARY.md
- FOUND: f3c92da
- FOUND: 6c58f32
