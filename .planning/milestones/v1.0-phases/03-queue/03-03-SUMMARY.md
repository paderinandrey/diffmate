---
phase: 03-queue
plan: 03
subsystem: github
tags: [watch, poll, debounce, gh, sqlite, rust]

requires:
  - phase: 03-queue
    provides: "Store::apply_poll and Store::schedule_after_run, plus Gh::search_review_requested"
provides:
  - "diffmate watch polls direct review requests on poll_interval_seconds and arms one debounced automatic job"
  - "The idle sleep is the sooner of 1s, the next poll, and the earliest next_attempt_at, and that 1s wake does not call gh"
  - "A successful automatic run calls schedule_after_run after the runner has stored the post-agent head"
affects: [03-queue, notifications, discard]
actuals:
  tokens: 8626
  tasks: 3
  commits: 3
plan_head_before: ce730ac9d01c196775d4d411f253cf89235a0e4b
plan_head_after: 96edf7e52970a86e57c644097209eff16c6ca572
commits: 3
tech-stack:
  added: []
  patterns:
    - "Watch calls Store::apply_poll and Store::schedule_after_run and does not reimplement queue rules"
    - "GitHub runs only when next_poll_at is due; next_poll_at moves forward after both a successful poll and a failed search"
    - "schedule_after_run runs after ReviewResult::Succeeded for an automatic job and before the outcome line"
key-files:
  created:
    - src/watch/poll.rs
    - src/watch/timing.rs
    - src/watch/timing/tests.rs
    - src/watch/job.rs
    - tests/watch/poll.rs
    - tests/watch/poll/ready.rs
    - tests/watch/poll/draft.rs
    - tests/watch/poll/absent.rs
    - tests/watch/poll/manual.rs
    - tests/watch/poll/debounce.rs
    - tests/watch/poll/follow.rs
    - tests/watch/poll/head.rs
    - tests/watch/poll/sync.rs
  modified:
    - src/watch/run.rs
    - src/watch.rs
    - src/config.rs
    - src/gh.rs
    - src/gh/meta.rs
    - src/store.rs
    - src/store/poll.rs
    - src/store/retry.rs
    - src/store/retry/queue.rs
    - tests/watch.rs
key-decisions:
  - "The first poll is due because next_poll_at starts at 0, after the lock and recovery"
  - "A failed search logs one poll-failed line, skips apply_poll and pr view, and still advances next_poll_at"
  - "Follow-up arming reads the head the runner stored, so schedule_after_run stays after the post-agent pr view"
patterns-established:
  - "Idle delay is timing::idle_delay in src/watch/timing.rs, capped by WATCH_IDLE_POLL"
  - "One poll is poll::once: search, then pr views, then apply_poll"
requirements-completed: [QUE-02, QUE-03, QUE-04, QUE-06, QUE-11]
coverage:
  - id: D1
    description: "Watch reads poll_interval_seconds and debounce_seconds, polls immediately after startup, and contacts GitHub only when the interval is due."
    requirement: QUE-11
    verification:
      - kind: integration
        ref: "tests/watch/poll/ready.rs#a_ready_request_becomes_one_automatic_run"
        status: pass
      - kind: integration
        ref: "tests/watch/poll/ready.rs#a_failed_search_does_not_change_request_seen"
        status: pass
    human_judgment: false
  - id: D2
    description: "The sleep is the sooner of the 1s idle poll, the next poll, and the earliest next_attempt_at. The 1s wake does not call gh."
    requirement: QUE-11
    verification:
      - kind: unit
        ref: "src/watch/timing/tests.rs#idle_delay_is_the_soonest_of_the_cap_poll_and_attempt"
        status: pass
      - kind: unit
        ref: "src/watch/timing/tests.rs#a_nearer_attempt_wakes_before_the_poll_on_a_paused_clock"
        status: pass
    human_judgment: false
  - id: D3
    description: "A draft in search is tracked with no job. The next ready poll arms one automatic run."
    requirement: QUE-04
    verification:
      - kind: integration
        ref: "tests/watch/poll/draft.rs#a_draft_is_tracked_without_a_job_until_it_is_ready"
        status: pass
    human_judgment: false
  - id: D7
    description: "A disappeared open request stays tracked with request_seen 0 and one delayed job for the later head. Closed or merged untracks, leaves request_seen 1, and inserts no job."
    requirement: QUE-03
    verification:
      - kind: integration
        ref: "tests/watch/poll/absent.rs#an_open_absence_keeps_one_delayed_job_for_the_new_head"
        status: pass
      - kind: integration
        ref: "tests/watch/poll/absent.rs#a_closed_absence_untracks_and_inserts_no_job"
        status: pass
    human_judgment: false
  - id: D4
    description: "A manual review stays untracked with no automatic job until a later search contains it, and that later poll leaves one delayed automatic job."
    requirement: QUE-06
    verification:
      - kind: integration
        ref: "tests/watch/poll/manual.rs#a_manual_review_stays_untracked_until_search_contains_it"
        status: pass
    human_judgment: false
  - id: D5
    description: "Two heads before the debounce job starts leave one job and one later run. A head change during a held run does not cancel it, and one follow-up job carries the replaced head."
    requirement: QUE-02
    verification:
      - kind: integration
        ref: "tests/watch/poll/debounce.rs#two_heads_before_the_job_starts_leave_one_later_run"
        status: pass
      - kind: integration
        ref: "tests/watch/poll/follow.rs#a_head_change_during_a_run_leaves_a_follow_up_job"
        status: pass
    human_judgment: false
  - id: D6
    description: "A failed pr view for a stored target missing from search becomes Absent::Skip, and an untracked target with request_seen already 1 is applied with head_sha None and no pr view."
    requirement: QUE-03
    verification: []
    human_judgment: true
    rationale: "No watch test fails a pr view or seeds an untracked request_seen row that is still present in search. Both branches are in src/watch/poll.rs."
duration: 40min
completed: 2026-11-09
status: complete
---

# Phase 3 Plan 03: Watch Poll Summary

**Watch polls direct review requests on the configured interval and arms one debounced automatic job through the existing store**

## Performance

- **Duration:** 40 min
- **Started:** 2026-11-09T11:25:53Z
- **Completed:** 2026-11-09T12:05:37Z
- **Tasks:** 3
- **Files modified:** 23

## Accomplishments

- `Settings::poll_interval` and `Settings::debounce` read the existing config fields. Both default to 60 seconds. `next_poll_at` starts at 0, so the first search runs on the first loop after the lock and recovery.
- `poll::once` searches, views missing stored targets and wanted ready heads, then calls `Store::apply_poll`. A failed search prints one `poll failed:` line and leaves `request_seen` unchanged.
- `timing::idle_delay` sleeps until the sooner of the 1-second idle poll, the next poll, and the earliest `next_attempt_at`. A successful automatic review calls `schedule_after_run` after the runner's post-agent `set_known_head`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Poll a ready request into one job** - `08990a3` (feat)
2. **Task 2: Cover drafts, absence, and manual runs** - `1639f41` (test)
3. **Task 3: Debounce a moving head and follow a held run** - `96edf7e` (test)

## Files Created/Modified

- `src/watch/poll.rs` - One poll: search, pr views, `apply_poll`
- `src/watch/timing.rs` - Idle delay capped at 1 second
- `src/watch/timing/tests.rs` - Paused-clock idle delay
- `src/watch/job.rs` - Run one job, then `schedule_after_run` on automatic success
- `src/watch/run.rs` - Poll when due, otherwise the next job or the idle sleep
- `src/config.rs` - `poll_interval` and `debounce` accessors
- `src/store/retry.rs` - `QueuedJob.origin`
- `src/store/retry/queue.rs` - Select and parse `origin`
- `src/gh.rs` - Search method is now used
- `src/gh/meta.rs` - `is_draft` is now read
- `src/store.rs` - `mod poll` is now used
- `src/store/poll.rs` - `dead_code` remains only on `queued_head` and `discard`
- `tests/watch.rs` - Declares the poll tests
- `tests/watch/poll.rs` - Shared config, search JSON, and tracked seed
- `tests/watch/poll/ready.rs` - Ready request and failed search
- `tests/watch/poll/draft.rs` - Draft tracked with no job until ready
- `tests/watch/poll/absent.rs` - Open disappearance and merged close
- `tests/watch/poll/manual.rs` - Manual review stays untracked until search
- `tests/watch/poll/debounce.rs` - Two heads, one later run
- `tests/watch/poll/follow.rs` - Head change during a held run
- `tests/watch/poll/head.rs` - Push a later pull-request head and replace its view
- `tests/watch/poll/sync.rs` - Spawn watch, wait on stderr and the database, count gh calls

## Decisions Made

- The outcome line is printed after `schedule_after_run`, so a test that blocks on `succeeded` sees the follow-up row.
- A draft in the search is passed with `head_sha` None and gets no pr view. A seen non-draft that will be tracked, or that is already tracked, gets one pr view. A failed view still applies `SeenRequest` with `head_sha` None.
- Closed and merged states compare with `eq_ignore_ascii_case`. Any other open view becomes `Absent::Open` with `is_draft` and `head_sha`. A view error becomes `Absent::Skip`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split the poll loop so each file stays inside the line limits**
- **Found during:** Task 1 (Poll a ready request into one job) and Task 3 (Debounce a moving head)
- **Issue:** `src/watch/run.rs` must stay at or under 90 lines, and a commit that adds more than 100 lines to one file is rejected. After rustfmt, the combined debounce test was 102 lines.
- **Fix:** The job body is `src/watch/job.rs`. Tests live under `tests/watch/poll/` with `#[path]` modules. The follow-up test and the git helpers are `follow.rs` and `head.rs`.
- **Files modified:** `src/watch/job.rs`, `src/watch/run.rs`, `tests/watch/poll.rs`, `tests/watch/poll/follow.rs`, `tests/watch/poll/head.rs`
- **Verification:** `src/watch/run.rs` is 69 lines. Each new file in the task 3 commit adds at most 54 lines. The pre-commit hook accepted the commit.
- **Committed in:** `08990a3`, `96edf7e`

**2. [Rule 1 - Bug] A poll wait could open the database before migration created `targets`**
- **Found during:** Task 2 (Cover drafts, absence, and manual runs)
- **Issue:** `sync::query` panicked with `no such table: targets` when the database file existed and the schema was not committed yet.
- **Fix:** A missing file, no rows, or `no such table` counts as 0 so the wait keeps polling.
- **Files modified:** `tests/watch/poll/sync.rs`
- **Verification:** Draft, open-absence, closed-absence, and manual tests passed afterward.
- **Committed in:** `1639f41`

**3. [Rule 3 - Blocking] `idle_delay` cannot be a const function**
- **Found during:** Task 1 (Poll a ready request into one job)
- **Issue:** `Duration::min` is not const, and `u64::try_from` is not stably const.
- **Fix:** `idle_delay` is an ordinary function in `src/watch/timing.rs`.
- **Files modified:** `src/watch/timing.rs`
- **Verification:** Paused-clock tests passed.
- **Committed in:** `08990a3`, `96edf7e`

**4. [Rule 1 - Bug] The paused-clock sketch waited 1499ms for a delay the 1s cap shortens to 1s**
- **Found during:** Task 3 (Debounce a moving head)
- **Issue:** `idle_delay(0, 60_000, Some(1_500))` is 1 second, because the idle cap is sooner than a 1500ms attempt.
- **Fix:** The test asserts that 1 second delay, advances 999ms, checks the sleep is still pending, then advances 1ms.
- **Files modified:** `src/watch/timing/tests.rs`
- **Verification:** `watch::timing::tests::a_nearer_attempt_wakes_before_the_poll_on_a_paused_clock` passed on a paused clock.
- **Committed in:** `96edf7e`

### Test fixtures

Draft and manual tests use `poll_interval_seconds = 1` so a later poll is observable in the suite. The disappeared-open test uses poll 1 and debounce 60. Runs that fetch a head use the real git SHA from the sandbox. The closed view uses `MERGED` so the case-insensitive close rule is what the test exercises. `allow(dead_code)` remains only on `KnownTarget.queued_head` and `Store::discard`.

---

**Total deviations:** 4 auto-fixed (2 blocking, 2 bug)
**Impact on plan:** The queue rules stay in the store. The splits and the 1-second cap assertion keep the hook and the specified idle delay.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Watch now polls and arms automatic work. Notifications and `debug discard` remain for later plans in this phase. `Store::discard` is still unused by the watch loop.

---
*Phase: 03-queue*
*Completed: 2026-11-09*

## Self-Check: PASSED

- FOUND: src/watch/poll.rs, src/watch/timing.rs, src/watch/job.rs, src/watch/run.rs (69 lines), tests/watch/poll/*.rs
- FOUND: 08990a3, 1639f41, 96edf7e as ancestors of HEAD
- commits measured from ce730ac9d01c196775d4d411f253cf89235a0e4b: 3
- plan_head_after: 96edf7e52970a86e57c644097209eff16c6ca572
