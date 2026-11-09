---
phase: 03-queue
plan: 01
subsystem: database
tags: [sqlite, migration, backup, request_seen, queued_head, rust]
requires:
  - phase: 02-reliability
    provides: "Schema version 2, the version 1 backup helper, and read-only outdated-schema errors"
provides:
  - "Schema version 3 with targets.request_seen and targets.queued_head"
  - "A version 2 snapshot at diffmate.db.v2-backup before the version 3 ALTER"
affects: [03-queue, tracking, debounce]
actuals:
  tokens: 3019
  tasks: 1
  commits: 1
plan_head_before: a8c1bdf18cc4cdd862229f315c07ae5dcb9d177f
plan_head_after: ea9de3b34c2d77c0ca4fc2767b0f6afd7938586f
commits: 1
tech-stack:
  added: []
  patterns:
    - "backup::create(db, version) publishes diffmate.db.v{version}-backup and requires that user_version"
    - "Version 0 applies schema.sql, migrate_v2.sql, then migrate_v3.sql with no snapshot"
    - "Version 1 snapshots only the version 1 file, then applies v2 and v3 SQL"
    - "Version 2 snapshots only the version 2 file, then applies migrate_v3.sql"
key-files:
  created:
    - src/store/migrate_v3.sql
    - src/store/migrate/tests/v3.rs
  modified:
    - src/store/migrate.rs
    - src/store/migrate/backup.rs
    - src/store/readonly.rs
    - src/store/migrate/tests.rs
    - src/store/migrate/tests/backups.rs
    - src/store/migrate/tests/concurrency.rs
    - tests/migration.rs
key-decisions:
  - "schema.sql stays the version 1 base; request_seen and queued_head exist only in migrate_v3.sql"
  - "A version 1 database is copied to diffmate.db.v1-backup and is not also copied as a version 2 backup"
  - "open_read_only treats user_version 1 and 2 as SchemaOutdated and does not migrate or snapshot"
patterns-established:
  - "Each schema version that is about to change is snapshotted under its own user_version before later SQL runs"
requirements-completed: []
coverage:
  - id: D1
    description: "Opening user_version 0, 1, or 2 ends at user_version 3, with a version 1 or version 2 snapshot only for the file that is about to change"
    verification:
      - kind: unit
        ref: "src/store/migrate/tests/v3.rs#version_two_keeps_tracking_and_snapshots_that_version"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests/v3.rs#fresh_database_has_no_backup"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests.rs#reopening_migrated_database_changes_nothing"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests.rs#fresh_and_upgraded_schemas_match"
        status: pass
      - kind: integration
        ref: "tests/migration.rs#phase_one_database_is_upgraded_and_kept"
        status: pass
    human_judgment: false
  - id: D2
    description: "targets.request_seen is 0 or 1 with default 0, queued_head is nullable text, tracking is unchanged, and head_sha is not written"
    verification:
      - kind: unit
        ref: "src/store/migrate/tests/v3.rs#version_two_keeps_tracking_and_snapshots_that_version"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests.rs#check_constraints_hold"
        status: pass
      - kind: integration
        ref: "tests/migration.rs#phase_one_database_is_upgraded_and_kept"
        status: pass
    human_judgment: false
  - id: D3
    description: "Read-only version 2 is SchemaOutdated and creates no backup; user_version 4 or newer is refused without a write"
    verification:
      - kind: unit
        ref: "src/store/migrate/tests/v3.rs#read_only_version_two_is_outdated_without_a_backup"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests.rs#newer_version_is_refused_and_left_alone"
        status: pass
      - kind: integration
        ref: "tests/migration.rs#status_on_phase_one_database_asks_for_an_upgrade"
        status: pass
    human_judgment: false
duration: 7min
completed: 2026-11-09
status: complete
---

# Phase 3 Plan 01: Schema version 3 with the version 2 backup

**Schema version 3 adds `request_seen` and `queued_head`, and a version 2 database is snapshotted to `diffmate.db.v2-backup` before that ALTER.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-11-09T09:38:38Z
- **Completed:** 2026-11-09T09:45:52Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- `SCHEMA_VERSION` is 3. A fresh database applies `schema.sql`, `migrate_v2.sql`, and `migrate_v3.sql` and writes no backup. A version 1 database is copied to `diffmate.db.v1-backup`, then both later scripts run. A version 2 database is copied to `diffmate.db.v2-backup`, then only `migrate_v3.sql` runs. Version 3 returns before any SQL. Version 4 or newer returns `Error::SchemaVersion` without a commit.
- `targets.request_seen` is `INTEGER NOT NULL DEFAULT 0` with a check that it is 0 or 1. `targets.queued_head` is nullable text. No third `tracking` value was added, and the migration does not write `targets.head_sha`. `schema.sql` is still the version 1 base.
- `backup::create(db, version)` keeps the existing publish rules and names the snapshot `diffmate.db.v{version}-backup`. `open_read_only` treats versions 1 and 2 as `SchemaOutdated` and does not migrate or snapshot.

## Task Commits

Each task was committed atomically:

1. **Task 1: Schema version 3 with the version 2 backup** - `ea9de3b` (feat)

## Files Created/Modified

- `src/store/migrate_v3.sql` - Adds `request_seen` and `queued_head`
- `src/store/migrate.rs` - Routes versions 0, 1, and 2 to version 3
- `src/store/migrate/backup.rs` - Parameterizes the snapshot file by `user_version`
- `src/store/readonly.rs` - Rejects versions 1 and 2 as outdated
- `src/store/migrate/tests/v3.rs` - Version 2 upgrade, fresh open, and read-only refusal
- `src/store/migrate/tests.rs` - Current version is 3; version 4 is refused; `request_seen = 2` fails its check
- `src/store/migrate/tests/backups.rs` - A version 1 upgrade ends at version 3 and writes no v2 backup
- `src/store/migrate/tests/concurrency.rs` - Concurrent version 1 opens end at version 3
- `tests/migration.rs` - The phase 1 database upgrades to version 3 with `request_seen` 0 and a version 1 backup

## Decisions Made

- `schema.sql` stays the version 1 base so a version 1 database does not already have `request_seen` when the v3 `ALTER` runs.
- A version 1 source is snapshotted only as version 1. The v2 backup is taken only when the file about to change is already version 2.
- `open_read_only` does not create either backup.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Post-migration tests still expected user_version 2**
- **Found during:** Task 1 (Schema version 3 with the version 2 backup)
- **Issue:** `backups.rs` and `concurrency.rs` opened a version 1 database and asserted `user_version` 2. That assertion is false once the current schema is 3, and those files were not in the plan's file list.
- **Fix:** Expect `user_version` 3 and assert that a version 1 source does not write `diffmate.db.v2-backup`.
- **Files modified:** `src/store/migrate/tests/backups.rs`, `src/store/migrate/tests/concurrency.rs`
- **Verification:** `mise run --force test -- -E 'test(store::migrate) | binary(migration)'` passed those tests
- **Committed in:** `ea9de3b` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Test expectations only. No production behavior beyond the plan.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `mise run --force test -- -E 'test(store::migrate) | binary(migration)'` exited 0: 16 passed, 325 skipped.
- `mise run --force lint` exited 0: rustfmt, clippy with warnings denied, cargo-machete, gitleaks (no leaks), and actionlint.
- `mise run --force test` exited 0: 330 passed, 11 skipped.
- The task commit's pre-commit hook passed large-files and secrets. Lint, test, actions, and docs were skipped as sources up-to-date after those runs. `mise run test:e2e` was not run.

## Next Phase Readiness

- Later queue plans can read and write `request_seen` and `queued_head` on a version 3 database.
- Phase 2 UAT was not modified and no UAT item was marked.

## Self-Check: PASSED

- FOUND: `src/store/migrate_v3.sql`
- FOUND: `src/store/migrate/tests/v3.rs`
- FOUND: `ea9de3b` as an ancestor of HEAD
- `src/store/schema.sql` does not contain `request_seen`

---
*Phase: 03-queue*
*Completed: 2026-11-09*
