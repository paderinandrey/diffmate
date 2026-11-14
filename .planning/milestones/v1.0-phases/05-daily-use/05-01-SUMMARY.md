---
phase: 05-daily-use
plan: 01
subsystem: doctor
tags: [doctor, gh, claude, flock, sqlite, read-only]
requires:
  - phase: 04-review-and-publish
    provides: "Store::open_read_only, schema error Display, and the Claude flag probe"
provides:
  - "diffmate doctor prints five stdout lines and exits 0 only when every check passed"
  - "A missing database is schema: ok — no database yet and is not opened"
  - "A readable lock that nobody holds is runner lock: ok — not held"
  - "A broken config.toml still prints all five lines"
affects: [05-daily-use, install]
actuals:
  tokens: 5155
  tasks: 2
  commits: 2
plan_head_before: 4f6a3d56e63585b35d606f3c0cb586196ae43270
plan_head_after: a64ba9fe9549be6a7f179b2298570bb1425d1026
commits: 2
tech-stack:
  added: []
  patterns:
    - "Doctor checks are awaited in order and printed only after every check returns"
    - "The runner lock is queried with fcntl_getlk on a read-only fd"
    - "A missing database is decided by stat and never passed to Store::open_read_only"
key-files:
  created:
    - src/doctor.rs
    - src/doctor/which.rs
    - src/doctor/gh_auth.rs
    - src/doctor/scratch.rs
    - src/doctor/claude.rs
    - src/doctor/storage.rs
    - src/doctor/lock_line.rs
    - src/doctor/schema.rs
    - tests/doctor.rs
    - tests/doctor/fresh.rs
    - tests/doctor/pass.rs
  modified:
    - src/cli.rs
    - src/main.rs
    - src/gh.rs
    - src/agent/claude.rs
    - src/agent/claude/contract.rs
key-decisions:
  - "Doctor is not part of the config preload match, so a broken config.toml still reaches doctor::run"
  - "A missing diffmate.db is reported from stat. Store::open_read_only is not called on that path"
  - "The Claude scratch directory is removed before doctor::run prints any line"
patterns-established:
  - "Pass and fail doctor lines share one Check and are sanitized, then folded to a single physical line"
  - "Lock role text is only the three fixed words watch, review, and tui"
requirements-completed: [OPS-03, OPS-05]
coverage:
  - id: D1
    description: "A fresh home prints five stdout lines in order, exits 1, creates no state directory, and does not start a Claude agent run."
    requirement: OPS-03
    verification:
      - kind: integration
        ref: "tests/doctor/fresh.rs#fresh_home_prints_five_lines_and_creates_nothing"
        status: pass
    human_judgment: false
  - id: D2
    description: "A broken config.toml still prints five lines. The storage line names config.toml and run_timeout_minutes, and the lock line still fails."
    requirement: OPS-05
    verification:
      - kind: integration
        ref: "tests/doctor/fresh.rs#broken_config_still_prints_five_lines"
        status: pass
    human_judgment: false
  - id: D3
    description: "A mode 0700 state directory and an unheld runner.lock containing watch exit 0 with runner lock: ok — not held. The database stays absent and try_lock succeeds after doctor exits."
    requirement: OPS-03
    verification:
      - kind: integration
        ref: "tests/doctor/pass.rs#free_lock_and_no_database_exits_zero"
        status: pass
    human_judgment: false
duration: 24min
completed: 2026-11-11
status: complete
---

# Phase 5 Plan 01: Five-line doctor Summary

**Read-only `diffmate doctor` prints gh auth, Claude, storage, the runner lock, and schema, and exits 0 only when every check passes**

## Performance

- **Duration:** 24 min
- **Started:** 2026-11-11T16:01:46Z
- **Completed:** 2026-11-11T16:25:40Z
- **Tasks:** 2
- **Files modified:** 16

## Accomplishments

- `diffmate doctor` always prints five sanitized stdout lines, in order, and exits 0 only when all five passed.
- A missing state directory, cache directory, config file, or database is ok. Doctor creates none of them. A missing database is not opened.
- A readable `runner.lock` that nobody holds passes as `runner lock: ok — not held`, even when the file text is `watch`.
- A broken `config.toml` fails only the storage line. The other four lines still print.

## Task Commits

Each task was committed atomically:

1. **Task 1: Five-line doctor on a fresh home** - `db5c95a` (feat)
2. **Task 2: Exit 0 when the lock is free and no database exists** - `a64ba9f` (test)

## Files Created/Modified

- `src/doctor.rs` - Awaits the five checks, sanitizes each line, then returns the exit code
- `src/doctor/gh_auth.rs` - `gh auth status` through `GhCli`, or the PATH when `gh` is missing
- `src/doctor/claude.rs` - Version probe, then the flag probe, with no agent run
- `src/doctor/scratch.rs` - Temp worktree, context, and prompt file, removed before the lines print
- `src/doctor/storage.rs` - State and cache mode `0700` plus `config::load`, still one line
- `src/doctor/lock_line.rs` - Read-only `fcntl_getlk`; does not call `holder` or `try_acquire`
- `src/doctor/schema.rs` - Stat first; `Store::open_read_only` only when the file exists
- `src/doctor/which.rs` - PATH lookup in `spawn_blocking`
- `src/cli.rs` - `Command::Doctor`
- `src/main.rs` - Dispatches doctor outside the config preload match
- `src/gh.rs` - `GhCli::command` is `pub(crate)`
- `src/agent/claude.rs` - Re-exports `AUTH_ENV`
- `tests/doctor/fresh.rs` - Fresh home and broken config
- `tests/doctor/pass.rs` - Free lock and no database

## Decisions Made

- Doctor does not join the Review, Watch, and Tui config preload. A broken `config.toml` is a storage failure, not an early return.
- `Store::open_read_only` is not called when `stat` says `diffmate.db` is absent. `Ok(None)` while a second stat still sees the file is `Error::SchemaOutdated(0)`.
- The Claude scratch directory is removed with `spawn_blocking` before `claude::check` returns, so it is gone before `doctor::run` prints. `Drop` removes it only if that cleanup did not run.
- The lock role is never copied into the line. Only `watch`, `review`, and `tui` select a held-by sentence.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Open the test lock with truncate set**
- **Found during:** Task 2 (Exit 0 when the lock is free and no database exists)
- **Issue:** Clippy `suspicious_open_options` denied `OpenOptions` that set `create(true)` without `truncate`
- **Fix:** Set `truncate(true)` on the new `runner.lock` before writing `watch\n`
- **Files modified:** `tests/doctor/pass.rs`
- **Verification:** `cargo clippy --locked --all-targets --all-features -- -D warnings` passed, and the free-lock test passed
- **Committed in:** `a64ba9f` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The test file now satisfies the existing clippy gate. No behavior change to doctor.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `diffmate doctor` is ready for the schema, held-lock, storage, and tool tests in the later plans of this phase.
- `Store::open_read_only` was not called on a missing database. The schema line for that case is `schema: ok — no database yet`.
- The scratch directory is removed before the five lines are printed, including when a probe fails.

## Self-Check: PASSED

- FOUND: src/doctor.rs
- FOUND: src/doctor/schema.rs
- FOUND: src/doctor/lock_line.rs
- FOUND: src/doctor/scratch.rs
- FOUND: tests/doctor/fresh.rs
- FOUND: tests/doctor/pass.rs
- FOUND: db5c95a
- FOUND: a64ba9f

---
*Phase: 05-daily-use*
*Completed: 2026-11-11*
