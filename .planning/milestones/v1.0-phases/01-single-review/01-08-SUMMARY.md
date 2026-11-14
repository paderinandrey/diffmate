---
phase: 01-single-review
plan: 08
subsystem: runner
tags: [rust, tokio, flock, sqlite, recovery, status, wal]

requires:
  - phase: 01-single-review
    provides: "01-04 tracer slice (runner, store, Paths), 01-07 render and anchoring, 01-03 black-box harness with FIFO-held fake agent"
provides:
  - "Exclusive runner lock: File::try_lock on state/runner.lock (mode 0600), RAII RunnerLock held for the whole review, 500 ms retry loop with a one-time waiting line"
  - "Recovery whenever a process becomes the runner: running runs and review jobs fail as interrupted: crash, run directories and worktrees are removed, worktree metadata of every clone is pruned"
  - "diffmate status: read-only list of targets with tracking and the latest job or run state, never locks, recovers or writes"
affects: [01-09, 01-10, 01-11, 01-12]

actuals:
  tokens: 10500
  tasks: 3
  commits: 3

plan_head_before: 9f4fc424890feef9cdf4d05ae97e03aba67721e3
plan_head_after: 466d2e75820fa25c00a7f51967954e293acbbadf
commits: 3

tech-stack:
  added: []
  patterns:
    - "Blocking std File::try_lock and rusqlite run inside spawn_blocking; the lock guard is a plain File kept alive by the caller"
    - "Lock-wait unit test spawns the waiter and polls an atomic counter with yield_now, so paused time never races a cancelled in-flight blocking attempt"
    - "Recovery scans only direct children of cache/runs: all-digit directories are removed, anything else is reported and kept"

key-files:
  created:
    - src/lock.rs
    - src/lock/tests.rs
    - src/recovery.rs
    - src/recovery/scan.rs
    - src/recovery/fake.rs
    - src/recovery/tests.rs
    - src/status.rs
    - src/store/status.rs
    - src/store/readonly.rs
    - src/store/recovery.rs
    - tests/status.rs
    - tests/concurrency.rs
    - tests/concurrency/helpers.rs
    - tests/recovery.rs
  modified:
    - src/cli.rs
    - src/main.rs
    - src/paths.rs
    - src/review.rs
    - src/store.rs

key-decisions:
  - "Store::open_read_only lives in src/store/readonly.rs (not store.rs, which would pass 100 lines) and returns None when the file is missing or user_version is 0; a newer version is an error"
  - "No read-write query_only fallback was needed: SQLite opens the WAL database read-only both with and without -wal/-shm present, as the existing database() test helper already did"
  - "Recovery marks every running run interrupted: crash (Phase 1 stores no stop cause), fails every running review job with reason interrupted: crash, and also removes the run directory of any all-digit entry whose run is not running, including ids with no row"
  - "A non-directory or symlink entry with an all-digit name is left in place with a warning, like non-numeric entries"
  - "status prints the latest job when it has no run: queued, running, or failed before run: REASON"

patterns-established:
  - "Two-process black-box tests load tests/concurrency/helpers.rs with #[path] (with_two_prs, spawn_review) and synchronize on the fake agent FIFOs and the first stderr line"

requirements-completed: [REV-02, REV-12]

coverage:
  - id: D1
    description: "diffmate status lists one row per target (OWNER/REPO#N, tracking, latest state), prints No targets. without creating anything, and never writes"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/status.rs#status_without_database_prints_no_targets_and_creates_nothing"
        status: pass
      - kind: integration
        ref: "tests/status.rs#status_lists_latest_run_per_target"
        status: pass
      - kind: integration
        ref: "tests/status.rs#status_does_not_write"
        status: pass
    human_judgment: false
  - id: D2
    description: "Two reviews of the same URL create two jobs, runs and drafts; the first draft is untouched and status shows the second run"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/status.rs#two_reviews_of_one_url_create_two_runs"
        status: pass
    human_judgment: false
  - id: D3
    description: "A second review waits for the runner lock with a waiting line, creates no run while it is held, runs after release, and status works meanwhile"
    requirement: REV-12
    verification:
      - kind: integration
        ref: "tests/concurrency.rs#second_review_waits_for_the_lock_and_status_works"
        status: pass
      - kind: unit
        ref: "src/lock/tests.rs#acquire_waits_while_the_lock_is_held_and_notifies_once"
        status: pass
      - kind: unit
        ref: "src/lock/tests.rs#try_acquire_reports_the_holder_and_creates_a_private_file"
        status: pass
    human_judgment: false
  - id: D4
    description: "After SIGKILL of a running review, the next runner marks the run failed (interrupted: crash), fails the job, removes the worktree and run directory and prunes clone metadata"
    requirement: REV-02
    verification:
      - kind: integration
        ref: "tests/recovery.rs#killed_runner_is_recovered_by_next_runner"
        status: pass
      - kind: unit
        ref: "src/recovery/tests.rs#a_running_run_and_job_are_failed_and_their_leftovers_removed"
        status: pass
    human_judgment: false

duration: 18min
completed: 2026-10-20
status: complete
---

# Phase 1 Plan 08: Runner lock, recovery and status Summary

**Exclusive File::try_lock runner lock with a waiting loop, crash recovery that runs before any job, and a read-only `diffmate status`, proven by a FIFO-synchronized two-process test and a SIGKILL recovery test**

## Performance

- **Duration:** about 18 min (start time was not recorded; counted from the previous plan's close at about 18:48Z to the last commit at 19:05Z)
- **Tasks:** 3
- **Files:** 19 changed in the plan range (14 created, 5 modified), 921 insertions and 2 deletions

## Accomplishments

- `diffmate review` now enqueues, takes `state/runner.lock`, runs recovery and only then starts its own job. The lock is a `std::fs::File::try_lock` taken inside `spawn_blocking`, created `0600`, opened close-on-exec so the agent never inherits it, and released by the OS when the process dies. A waiting review prints `waiting: another diffmate process holds the runner lock; job N stays queued until it is released` once and retries every 500 ms.
- Recovery (`src/recovery.rs`, `src/store/recovery.rs`) fails every `running` run as `failed (interrupted: crash)` and every `running` review job with reason `interrupted: crash` in one immediate transaction, removes the worktree (through `Git::worktree_remove` on the run's clone) and the directory of every all-digit entry under `cache/runs` whose run is not running, leaves other entries with a warning, and runs `git worktree prune` on every clone under `state/repos`. Each action is printed on stderr as a `recovery:` line.
- `diffmate status` opens SQLite read-only, never touches the lock or recovery, and prints `OWNER/REPO#N  untracked  LATEST`. LATEST is the latest job's run state (`running`, `succeeded (findings: K)`, `succeeded (no_findings)`, `failed (KIND)`, `failed (interrupted: CAUSE)`) or, without a run, `queued`, `running` or `failed before run: REASON`. With no database it prints `No targets.` and creates nothing.
- The black-box REV-12 test holds review A in the fake agent, starts review B (PR 2), reads B's first stderr line (`waiting:`), runs `status` (exits 0, `o/r#1 ... running`, `o/r#2 ... queued`), checks that B has no run and a queued job, releases A and checks both succeed with run 2 created after run 1 finished. The recovery test SIGKILLs A inside the agent, releases the orphan fake, and checks that B's recovery cleaned everything A left.

## Task Commits

1. **Task 1: read-only diffmate status** - `30e244f` (feat)
2. **Task 2: runner lock and waiting loop** - `7d76408` (feat)
3. **Task 3: recovery whenever a process becomes the runner** - `466d2e7` (feat)

**Plan metadata:** the commit that adds this SUMMARY (docs: complete plan)

## Decisions Made

- See `key-decisions` in the frontmatter.
- `Paths::lock`, `Paths::runs_dir` and `Paths::repos_dir` were added (01-04 had deferred them because dead code is denied); `run_dir` and `repo_dir` now build on them.
- Recovery treats a numeric directory of an unknown run id as removable (no clone to remove a worktree from, the prune step clears any metadata), and never touches a run that is still `running`.
- `ReviewResult` still carries `#[expect(dead_code)]` for `run_id`: this plan does not read it either.

## Verification

Observed in this session:

- `mise run --force test -- -E 'binary(status) | binary(cli)'`: 7 passed (Task 1).
- `mise run --force test -- -E 'test(lock::) | binary(concurrency)'`: 3 passed; the lock unit test was also run 8 times in a row after the flake fix, all passed.
- `mise run --force test -- -E 'test(recovery::) | binary(recovery) | binary(concurrency) | binary(status)'`: 7 passed.
- `mise run --force test`: 112 run, 112 passed, 3 skipped (the live probes).
- `mise run --force lint`: exit 0 (rustfmt, clippy pedantic with warnings denied, cargo-machete, gitleaks, actionlint).
- `mise run coverage`: exit 0, 91.12% line coverage (gate 80%).
- Acceptance greps: `open_read_only` is in `src/status.rs`'s call path (`Store::open_read_only` in `src/status.rs`), `try_lock` in `src/lock.rs`, `start_paused = true` in `src/lock/tests.rs`, `recovery::run` and `lock::acquire` in `src/review.rs`; every file the plan touches is at most 100 lines (largest: tests/status.rs 97, src/recovery.rs 91).
- The hk pre-commit hook (lint, large-files, secrets, docs, test, actions) passed on all three commits without `--no-verify`.
- Not run: `mise run vuln` and `mise run deps:check` (no crate was added), `mise run test:e2e`, `mise run docs:check` outside the hook.

## Deviations from Plan

**1. [Rule 3 - Blocking] Extra files for the 100-line commit hook**
- **Issue:** `src/store.rs` was at 77 lines, so `open_read_only` cannot live there; the recovery module, its fake Git and the two-process test helpers do not fit the planned files.
- **Fix:** added `src/store/readonly.rs` (open_read_only), `src/recovery/scan.rs` (directory scans), `src/recovery/fake.rs` (hand-written typed fake of the `Git` trait, test only) and `tests/concurrency/helpers.rs` (with_two_prs, spawn_review, loaded with `#[path]` by `tests/concurrency.rs` and `tests/recovery.rs`). Public paths named in the plan interfaces hold: `Store::open_read_only`, `Store::target_statuses`, `lock::{RunnerLock, LOCK_POLL, try_acquire, acquire}`, `recovery::{RecoveryReport, run}`, `status::run`.
- **Committed in:** 30e244f, 7d76408, 466d2e7

**2. [Rule 1 - Bug] Lock unit test raced a cancelled blocking attempt**
- **Found during:** Task 2
- **Issue:** the first version wrapped `acquire` in `tokio::time::timeout`; under paused time the clock jumps while a `spawn_blocking` attempt is in flight, so the cancelled attempt could finish after the holder was dropped and make the second `acquire` see a conflict (the waiting callback ran twice, observed once in a run).
- **Fix:** the waiter is spawned as a task, the test yields until the callback ran once, sleeps 5 virtual seconds, asserts the task is unfinished, drops the holder and awaits the task.
- **Committed in:** 7d76408

**3. [Rule 3 - Blocking] `RecoveryReport::failed_jobs` printed as well**
- **Issue:** dead code is denied and nothing else reads `failed_jobs`.
- **Fix:** `RecoveryReport::lines` prints `recovery: job N failed (interrupted: crash)` after the run lines.
- **Committed in:** 466d2e7

**4. [Rule 3 - Blocking] clippy pedantic on the Git fake**
- **Issue:** `unused_async_trait_impl` and `unnecessary_wraps` on a fake whose methods never await.
- **Fix:** the fake implements the trait methods with `impl Future` and `std::future::ready`.
- **Committed in:** 466d2e7

**Total deviations:** 4 (3 blocking, 1 bug). **Impact:** none on behavior beyond the plan; interfaces keep their names.

## Issues Encountered

- `.planning/config.json`, `.planning/milestone.lock` and `.planning/state.json` were already modified or untracked and were left out of every commit.
- Phase 1 has no stop-cause storage, so recovery always records `crash`; `shutdown` and `user_cancel` arrive with 01-10 and Phase 2.
- Steps 2 and 5 of SPEC Recovery (publications, reconcile jobs) have no tables yet and are not implemented.
- A `status` against a database file that exists but has `user_version` 0 (created, never migrated) prints `No targets.`

## User Setup Required

None.

## Known Stubs

None.

## Threat Flags

None beyond the plan's threat model. Mitigations present: T-01-32 (OS lock via File::try_lock, no PID file, two-process test), T-01-33 (std opens the lock close-on-exec, the agent runs in its own process group), T-01-34 (only all-digit directories directly under cache/runs, never a running run, other entries reported and kept), T-01-35 (read-only open, no lock, no recovery, status_does_not_write).

## Next Phase Readiness

- Plan 01-09: `src/runner.rs`, `src/store/runs.rs`, `src/git.rs`, `src/gh.rs` and `src/context*` were not touched. `src/review.rs` is now enqueue, lock, recovery, own job, render.
- Plan 01-10 (signals): the lock guard is dropped at the end of `review::execute`; a signal handler that wants to record `interrupted: shutdown` must run before that drop.
- Plan 01-02 still waits on the owner; `tests/fixtures/live-prs.json` was not touched.

## Self-Check: PASSED

All created files exist and commits 30e244f, 7d76408 and 466d2e7 are ancestors of HEAD (verified with `[ -f ]` and `git merge-base --is-ancestor` in the closing step).

---
*Phase: 01-single-review*
*Completed: 2026-10-20*
