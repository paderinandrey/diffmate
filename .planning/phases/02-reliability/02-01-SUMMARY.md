---
phase: 02-reliability
plan: 01
subsystem: database
tags: [sqlite, migration, backup, recovery, rust]
requires:
  - phase: 01-single-review
    provides: "Version 1 store, read-only status and black-box CLI harness"
provides:
  - "Transactional schema v2 upgrade preserving every Phase 1 value"
  - "One-time private version 1 backup before upgrading an existing database"
  - "Read-only outdated-schema error with an upgrade hint"
affects: [02-03, 02-04, 02-05, 02-06, 02-07, 02-08, 02-09, 02-10]
actuals:
  tokens: 5959
  tasks: 3
  commits: 2
plan_head_before: 4c8cf7d94518f5eebd46b6b96f29e8c762e91f9b
plan_head_after: cd57d9a6adbf956512e1d4d2050fc27399582719
commits: 2
tech-stack:
  added: []
  patterns:
    - "An IMMEDIATE transaction serializes migration and backup publication"
    - "VACUUM INTO uses a separate read-only connection while the migration connection holds the writer reservation"
    - "A private temporary backup is validated, synced and published without replacing an existing file"
key-files:
  created:
    - src/store/migrate.rs
    - src/store/migrate_v2.sql
    - src/store/migrate/backup.rs
    - src/store/migrate/tests.rs
    - src/store/migrate/tests/backups.rs
    - src/store/migrate/tests/concurrency.rs
    - src/store/migrate/tests/helpers.rs
    - src/store/migrate/tests/rollback.rs
    - tests/migration.rs
    - tests/migration/fixture.rs
    - tests/migration/fixture.sql
    - .planning/phases/02-reliability/02-01-RED.json
  modified:
    - src/store.rs
    - src/store/readonly.rs
    - src/error.rs
key-decisions:
  - "Owner selected additive-backup on 2026-11-01, with the proposed columns unchanged"
  - "Existing backups must pass version and quick_check validation; corrupt backups are preserved and prevent migration"
  - "Enable WAL only after migration succeeds, so refusal of a newer database leaves its bytes unchanged"
requirements-completed: [REL-06, REL-07]
coverage:
  - id: D1
    description: "Existing Phase 1 rows survive the v2 upgrade and receive the approved defaults"
    verification:
      - kind: integration
        ref: "tests/migration.rs#phase_one_database_is_upgraded_and_kept"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests.rs#fresh_and_upgraded_schemas_match"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests/rollback.rs#migration_failure_rolls_back_columns_and_keeps_version_one_values"
        status: pass
    human_judgment: false
  - id: D2
    description: "A consistent private v1 backup survives concurrent opens and interrupted publication"
    verification:
      - kind: unit
        ref: "src/store/migrate/tests/backups.rs#backup_preserves_version_one_rows_and_is_not_rewritten"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests/backups.rs#interrupted_temporary_backup_is_replaced"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests/backups.rs#published_backup_before_interruption_is_kept_and_temporary_removed"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests/concurrency.rs#concurrent_first_opens_share_one_backup_and_migration"
        status: pass
    human_judgment: false
  - id: D3
    description: "Unsupported versions are refused without modifying the database; reopening v2 is idempotent"
    verification:
      - kind: integration
        ref: "tests/migration.rs#status_on_phase_one_database_asks_for_an_upgrade"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests.rs#newer_version_is_refused_and_left_alone"
        status: pass
      - kind: unit
        ref: "src/store/migrate/tests.rs#reopening_version_two_changes_nothing"
        status: pass
    human_judgment: false
  - id: D4
    description: "Release-binary upgrade of the owner's actual Phase 1 database"
    verification: []
    human_judgment: true
    rationale: "Deferred to end-of-phase UAT; the owner's live database was not opened or modified in this plan"
duration: not recorded
completed: 2026-11-01
status: complete
---

# Phase 2 Plan 01: Schema migration and version 1 backup

**Schema v2 adds the queue and freshness fields in one transaction, preserving Phase 1 data and keeping a private consistent v1 backup for manual restoration.**

## Performance

- Start time was not recorded; implementation closeout was at 2026-10-31T23:45:34Z.
- Tasks: 3, including the owner's checkpoint decision.
- Files changed: 15 before this summary; no dependency changes.
- Actual token estimate: source/test diff bytes divided by four (23,836 / 4), not a runtime token count.

## Accomplishments

- Fresh databases apply the unchanged `schema.sql` followed by `migrate_v2.sql`; version 1 databases apply only the latter. Both paths produce identical `sqlite_master` entries. Version 2 opens without changing rows; newer versions are refused without changing database bytes.
- Existing version 1 databases get `diffmate.db.v1-backup`, mode `0600`, before any schema change. The backup includes committed WAL data, preserves all fixture values and is never rewritten on subsequent opens.
- Backup creation and migration are serialized by the main connection's IMMEDIATE transaction. A second read-only connection performs `VACUUM INTO` into a private temporary file. Validation, file sync, no-replacement publication by hard link and directory sync precede the schema change. Recovery removes abandoned temporary files; an existing valid backup is synced again before proceeding. A corrupt or wrong-version backup prevents the upgrade and is kept.
- `status` on schema v1 exits 1 and asks for `diffmate review` once. The test verifies both rows and database bytes remain unchanged and no sidecar or backup is created.

## Task Commits

1. **Task 1: Owner confirms schema shape** — owner replied `additive-backup, approved`; no code commit.
2. **Task 2: Upgrade tracer and backup** — `1ef670f` (`feat`).
3. **Task 3: Migration properties and interrupted-backup recovery** — `cd57d9a` (`fix`).

Plan metadata is the commit adding this summary. Shared STATE, ROADMAP, REQUIREMENTS and config updates remain with the orchestrator.

## Schema v2 as built

| Table | Column | Default and constraint |
| --- | --- | --- |
| jobs | origin | NOT NULL, `'manual'`; manual or automatic |
| jobs | force | NOT NULL, 0; 0 or 1 |
| jobs | stop_reason | NULL; timeout, shutdown or user_cancel |
| jobs | retry_count | NOT NULL, 0; nonnegative |
| jobs | max_retries | NULL; nonnegative when present |
| jobs | next_attempt_at | NULL; epoch milliseconds |
| jobs | result_run_id | NULL; references runs(id) |
| targets | head_sha | NULL |
| targets | head_checked_at | NULL; epoch milliseconds |

Also added `jobs_state ON jobs (state, next_attempt_at)`. The runs table and original `schema.sql` are unchanged. This plan supplies storage for REL-06 and REL-07; later plans implement their runtime semantics.

## Verification

Observed results:

- `mise run --force test -- -E 'binary(migration) | binary(status) | binary(recovery) | binary(concurrency) | test(store::)'`: 7 passed.
- `mise run --force test -- -E 'test(store::migrate::) | binary(migration)'`: final run 13 passed, 182 excluded by the filter.
- `mise run --force lint`: passed on the final code (rustfmt, clippy with warnings denied, cargo-machete, gitleaks, actionlint).
- `mise run --force coverage`: final exact-code run 184 passed, 11 skipped; **93.73% line coverage**, above the 80% gate.
- Mandatory hk hooks passed on both task commits. The final hook's `mise run test` executed all 184 tests: 184 passed, 11 skipped. Hook checks also covered large-files, secrets, docs and actions; no bypass was used.
- `git diff --check` passed. Acceptance checks found one `migrate::run` call, nine `ADD COLUMN` statements and one test-module declaration. Every changed source/test file has at most 100 lines. `git diff 4c8cf7d -- src/store/schema.sql` is empty.

Not run in this plan: release build, live `test:e2e`, dependency/advisory checks and the owner's database hand check. No crate or tool changed. Live checks and release acceptance remain with the phase gate.

## RED / GREEN Evidence

The initial property run executed 11 tests: 10 passed and `published_backup_before_interruption_is_kept_and_temporary_removed` failed at `assert!(!temporary.exists())`. The already-published backup remained intact, but its temporary hard link survived. The fix validates the existing copy, removes the temporary path and syncs the published backup before migration. The final targeted run passed all 13 tests, including the two additional rollback/refusal checks.

The original command, output, exit code and semantic assessment are saved in [02-01-RED.json](./02-01-RED.json). Tests and correction were committed together after GREEN because the mandatory project hook requires passing tests in every commit. There was no failing-test commit and no hook bypass.

## Deviations from Plan

1. **[Rule 2 — Missing critical] Serialize and publish backups safely.** The proposed existence check plus direct `VACUUM INTO` could race or accept an incomplete output after interruption. The implementation uses the transaction's writer reservation, a second read-only connection and a synced temporary file published without replacement. Existing backups are validated and interrupted temporary files are cleaned. [SQLite documents](https://www.sqlite.org/lang_vacuum.html) that VACUUM cannot run on a connection with an active transaction and that interrupted INTO output can be incomplete. Covered by concurrency, WAL-content, interruption, corrupt-backup and rollback tests; commits `1ef670f`, `cd57d9a`.
2. **[Rule 1 — Bug] Avoid modifying a newer database before rejecting it.** WAL activation moved after migration so an unsupported version is refused before journal-mode changes. The test compares the complete file bytes. Commit `1ef670f`.
3. **[Rule 3 — Blocking] Split helpers to satisfy the file-size convention.** Backup logic, property tests and the Phase 1 fixture use small submodules. `migrate::run` receives the database path explicitly as its second argument. No public CLI interface changed.
4. **Project commit policy takes precedence over RED commits.** Actual RED evidence was retained, but the failing test and fix share the green Task 3 commit so every mandatory hook passes.

## Issues Encountered

- Initial compilation required collecting rusqlite 0.41's `column_names()` iterator before joining it. Corrected before the tracer passed.
- Mise required execution outside the filesystem sandbox for its cache and test subprocesses; scoped escalation was used.
- Existing dirty `.planning/STATE.md`, `.planning/config.json`, `.planning/milestone.lock` and `.planning/state.json` were not staged or changed by this executor.

## User Setup Required

None. The application upgrades an existing version 1 database on its next read-write open. The approved backup remains beside it for manual restoration or later removal.

## Known Stubs

None in this plan. New queue and head columns are intentionally unused until later plans.

## Threat Flags

T-02-01 is covered by preservation, rollback and backup tests; T-02-02 by the concurrent-open test and the IMMEDIATE transaction; T-02-03 by the byte-preserving read-only status test. No crate was added.

## Next Phase Readiness

Plans 02-03 onward can use the approved column names and `SCHEMA_VERSION = 2`. Plan 02-02's dependency approval is recorded by the orchestrator and was not consumed here. The owner's live-database check remains an end-of-phase UAT item; no live database was touched.

## Self-Check: PASSED

Both task commits are reachable from the phase branch, all created files exist, source diff and acceptance checks pass, and automated verification results above were observed.
