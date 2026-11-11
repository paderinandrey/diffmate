---
phase: 05-daily-use
plan: 02
subsystem: doctor
tags: [doctor, flock, sqlite, wal, gh, claude]
requires:
  - phase: 05-daily-use
    provides: "Five-line read-only diffmate doctor from plan 05-01"
provides:
  - "A held watch lock prints runner lock: ok — held by watch and the parent flock survives"
  - "Schema versions 0, 1, 5, and 6 use the existing sentences and the file is not migrated"
  - "A loose directory, an unreadable lock, and a missing gh or claude each name the fix on their own line"
affects: [05-daily-use, install]
actuals:
  tokens: 4808
  tasks: 3
  commits: 3
plan_head_before: a883184d6b2baf3b41f976e1781840009f8dd74e
plan_head_after: 3aedd57832c5482643b74b39509cad4da045d1bc
commits: 3
tech-stack:
  added: []
  patterns:
    - "Doctor queries a held BSD flock with fcntl_getlk and treats rustix's negative-pid assertion as held"
    - "Schema age is an immutable URI read of user_version, not Store::open_read_only"
key-files:
  created:
    - tests/doctor/held.rs
    - tests/doctor/schema.rs
    - tests/doctor/wal.rs
    - tests/doctor/lock_role.rs
    - tests/doctor/storage.rs
    - tests/doctor/tools.rs
  modified:
    - src/doctor/lock_line.rs
    - src/doctor/schema.rs
    - src/store.rs
    - tests/doctor.rs
key-decisions:
  - "A second try_lock on the same fd succeeds on macOS, so held-lock tests open a new fd and expect WouldBlock"
  - "rustix debug-asserts when F_GETLK reports l_pid -1; doctor catches that unwind and reports the lock held"
  - "Store::open_read_only of a WAL database created -shm and -wal, so schema age uses an immutable URI"
patterns-established:
  - "Only the words watch, review, and tui are interpolated into a held-lock line"
  - "Doctor never chmods, never migrates, and never takes the runner lock"
requirements-completed: [OPS-03, OPS-05]
coverage:
  - id: D1
    description: "A parent flock whose first line is watch, plus user_version 5, exits 0 with runner lock: ok — held by watch and schema: ok. The flock and the database bytes survive."
    requirement: OPS-03
    verification:
      - kind: integration
        ref: "tests/doctor/held.rs#held_watch_lock_and_schema_five_exit_zero"
        status: pass
    human_judgment: false
  - id: D2
    description: "user_version 0 and 1 print the review-once sentence, user_version 6 prints the newer-schema sentence, and a checkpointed WAL database at version 5 gains no directory entry."
    requirement: OPS-03
    verification:
      - kind: integration
        ref: "tests/doctor/schema.rs#user_version_zero_is_not_a_missing_database"
        status: pass
      - kind: integration
        ref: "tests/doctor/wal.rs#wal_user_version_five_adds_no_directory_entry"
        status: pass
    human_judgment: false
  - id: D3
    description: "Held review, tui, and an unreadable role keep the parent flock. Mode 0755 and a file at the cache path share one storage line and are not changed. A PATH without gh and claude fails only those two lines, and Claude 2.1.303 does not run the flag probe."
    requirement: OPS-05
    verification:
      - kind: integration
        ref: "tests/doctor/lock_role.rs#held_review_tui_and_unreadable_role_keep_the_flock"
        status: pass
      - kind: integration
        ref: "tests/doctor/storage.rs#loose_state_dir_and_file_cache_share_one_storage_line"
        status: pass
      - kind: integration
        ref: "tests/doctor/tools.rs#old_claude_does_not_run_the_flag_probe"
        status: pass
    human_judgment: false
duration: 27min
completed: 2026-11-11
status: complete
---

# Phase 5 Plan 02: Doctor failure lines Summary

**Doctor names a held watch lock, schema age, a loose directory, and a missing gh or claude without migrating, chmodding, or taking the lock**

## Performance

- **Duration:** 27 min
- **Started:** 2026-11-11T16:40:37Z
- **Completed:** 2026-11-11T17:07:27Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- A parent `flock` whose first line is `watch` prints `runner lock: ok — held by watch` and is still held after doctor exits. `user_version` 5 prints `schema: ok` and the database bytes do not change.
- Versions 0 and 1 fail with the existing sentence to run `diffmate review` once. Version 0 is not `no database yet`. Version 6 fails because the database is newer than this diffmate supports. No backup is written.
- A read-only open of a WAL database created `diffmate.db-shm` and `diffmate.db-wal`. Schema age now uses an immutable URI (`file:…?mode=ro&immutable=1`) instead of `Store::open_read_only`. That read does not call `Store::open` or `migrate::run`, and the WAL test gains no directory entry.
- Held `review` and `tui` print their holder. Any other held bytes, including empty, print `runner lock: ok — held, role unreadable`. Mode `000` names `chmod 600` and stays `000`.
- A state directory at mode `0755` and a regular file at the cache path share one storage line, naming `chmod 700` and `not a directory`. Doctor does not chmod or delete them.
- `PATH=/usr/bin:/bin` fails only the gh and claude lines and still prints five lines. Claude `2.1.303` stops before the flag probe. `gh auth status` exit 1 names `gh auth login`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Held watch lock and schema version 5** - `7cf3d85` (test)
2. **Task 2: Schema age is reported and not migrated** - `335410f` (test)
3. **Task 3: Lock, storage, and tool failures name the fix** - `3aedd57` (test)

## Files Created/Modified

- `tests/doctor/held.rs` - Held watch lock, schema version 5, surviving flock, unchanged database
- `tests/doctor/schema.rs` - Versions 0, 1, and 6 stay unmigrated and use the existing sentences
- `tests/doctor/wal.rs` - Checkpointed WAL version 5 adds no directory entry
- `tests/doctor/lock_role.rs` - Held review, tui, unreadable role, and mode 000
- `tests/doctor/storage.rs` - Mode 0755 and a non-directory cache path on one line
- `tests/doctor/tools.rs` - Missing binaries, an old Claude version, and a failed `gh auth status`
- `src/doctor/lock_line.rs` - `fcntl_getlk`, with rustix's negative-pid assertion treated as held
- `src/doctor/schema.rs` - Immutable URI query of `user_version`
- `src/store.rs` - Re-exports `SCHEMA_VERSION` for the doctor schema check
- `tests/doctor.rs` - Declares the new modules

## Decisions Made

- On this Mac, `try_lock` on the fd that already holds the flock succeeds. The tests open a second fd, which returns `WouldBlock` while the parent still holds the lock.
- `rustix` 1.1.6 `Pid::from_raw` debug-asserts `raw >= 0`. macOS `F_GETLK` of a BSD `flock` sets `l_pid` to `-1` when the lock is held, so the debug doctor binary panicked inside `fcntl_getlk`. The probe catches that unwind, silences the hook for the call, and reports the lock held. Release builds do not hit the assertion; `fcntl_getlk` returns the blocking lock directly.
- `Store::open_read_only` created both WAL sidecars. The schema check no longer uses it. The replacement is a read-only immutable URI so SQLite does not create `-shm`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] rustix panics when a held BSD flock reports pid -1**
- **Found during:** Task 1 (Held watch lock and schema version 5)
- **Issue:** `fcntl_getlk` debug-asserts in `Pid::from_raw` because macOS sets `l_pid` to `-1`. The lock line became a task panic and doctor exited 1.
- **Fix:** Catch that unwind, hide its hook for the call, and treat it as a held lock. The role is still read from the file.
- **Files modified:** `src/doctor/lock_line.rs`
- **Verification:** `held_watch_lock_and_schema_five_exit_zero` exits 0 and the second open's `try_lock` returns `WouldBlock`
- **Committed in:** `7cf3d85` (Task 1 commit)

**2. [Rule 1 - Bug] A read-only WAL open created sidecars**
- **Found during:** Task 2 (Schema age is reported and not migrated)
- **Issue:** After a checkpoint, removing setup `-wal` and `-shm`, `Store::open_read_only` recreated both names. The schema line was already `schema: ok`.
- **Fix:** Read `user_version` through `file:…?mode=ro&immutable=1`. Re-export `SCHEMA_VERSION` so the sentences stay on the existing error Display values. The WAL test lives in `tests/doctor/wal.rs` because `tests/doctor/schema.rs` would have passed the 100-line hook.
- **Files modified:** `src/doctor/schema.rs`, `src/store.rs`, `tests/doctor/wal.rs`
- **Verification:** `wal_user_version_five_adds_no_directory_entry` passes, and versions 0, 1, and 6 are unchanged
- **Committed in:** `335410f` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 bug)
**Impact on plan:** Both fixes keep doctor read-only. No new crate, no migration, and no sixth check.

The dispatch required one commit per task, so the failing tests and their fixes share the task commit instead of a separate RED commit.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Doctor's held-lock, schema-age, storage-mode, and tool-failure lines are covered by the sandbox suite.
- `docs/install.md` was left untouched. This plan does not reboot the machine; the reboot sentence is the held-watch line the tests lock.

## Known Stubs

None.

---
*Phase: 05-daily-use*
*Completed: 2026-11-11*

## Self-Check: PASSED

- FOUND: tests/doctor/held.rs
- FOUND: tests/doctor/schema.rs
- FOUND: tests/doctor/wal.rs
- FOUND: tests/doctor/lock_role.rs
- FOUND: tests/doctor/storage.rs
- FOUND: tests/doctor/tools.rs
- FOUND: src/doctor/lock_line.rs
- FOUND: src/doctor/schema.rs
- FOUND: 7cf3d85
- FOUND: 335410f
- FOUND: 3aedd57
