---
phase: 04-review-and-publish
plan: 07
subsystem: tui
tags: [ratatui, tokio, sqlite, gh, publish, recovery]
requires:
  - phase: 04-review-and-publish
    provides: "confirm_publication, execute_publish, and the publish checks from plan 04-06"
provides:
  - "A crash after sending is committed leaves the publication unknown, fails the publish job with reason outcome unknown, and queues one reconcile job. Recovery does not call create_review."
  - "A crash while the publication is still pending requeues that same job. The next run checks again and posts once."
  - "A 30 second timeout or a server error on create_review leaves unknown and blocks a new confirm. HTTP 422 rejects. Reconcile submits only when a review body contains the publication marker."
  - "y enqueues and runs one reconcile. b asks first. Leave unknown is highlighted. Abandon sets reason confirmed by me and does not post."
affects: [04-review-and-publish, tui, publish, recovery]
actuals:
  tokens: 14502
  tasks: 3
  commits: 3
plan_head_before: 3898b691bccebdeedeb02c41926203af54c01a3e
plan_head_after: d62d024fc2c30e9a62bce0ad8ee4d288a1f5b1e1
tech-stack:
  added: []
  patterns:
    - "create_review is wrapped in tokio::time::timeout of 30 seconds. transient_gh maps a timeout, a network error, and HTTP 500-599 to unknown. Any other HTTP status, including 401, 403, and 422, rejects. The POST is not retried."
    - "Recovery settles running publish, reconcile, and sync jobs in the same transaction as review recovery, then inserts one reconcile row per unknown publication that has no queued or running reconcile. It makes no gh call."
    - "The tui runner starts publish and reconcile. A sync job ends the loop without start_job."
key-files:
  created:
    - src/publish/outcome/mod.rs
    - src/publish/outcome/settle.rs
    - src/publish/reconcile/mod.rs
    - src/publish/reconcile/write.rs
    - src/store/recovery/service.rs
    - src/publish/tests/crash.rs
  modified:
    - src/crash.rs
    - src/publish/post.rs
    - src/store/publish.rs
    - src/store/recovery.rs
    - src/tui/runner.rs
    - src/tui/keys.rs
    - src/tui/model.rs
    - src/tui/post.rs
    - src/tui/confirm.rs
    - src/gh/reviews.rs
key-decisions:
  - "In-process crash points are an AtomicU8 behind Hold. Code 1 hangs before sending and code 2 hangs after the sending commit. set_var is unsafe in edition 2024, so tests do not set DIFFMATE_CRASH_POINT. The env var still hangs a child process."
  - "A running publish that is still pending is requeued on the same job and publication. sending and an already unknown publication fail the job with reason outcome unknown. submitted succeeds the job. rejected and abandoned fail the job and do not post again."
  - "Recovery inserts reconcile rows itself. A second recovery does not insert another while one is queued or running."
  - "Reconcile submits only from unknown, and only when a listed review has the current login as author and the publication marker in its body. A missing marker succeeds the job and leaves unknown. A list or login failure fails the job and leaves unknown."
  - "The tui serves publish and reconcile. Sync still ends the loop without being started. y inserts one reconcile only for an unknown publication that has no queued or running reconcile, then drive runs it. b highlights Leave unknown. Abandon writes confirmed by me and refuses any other state."
patterns-established:
  - "Marker text is <!-- diffmate:publication=<publication-id> --> copied from SPEC Publication step 5. Reconcile searches review bodies for that string and never calls create_review."
  - "Schema user_version stays 5. One publication per draft. enqueue_reconcile is a no-op unless state is unknown and no reconcile is queued or running."
requirements-completed: [PUB-03, PUB-10, PUB-11, PUB-14]
coverage:
  - id: D1
    description: "A crash after sending is committed leaves the publication unknown, the publish job failed with reason outcome unknown, and one queued reconcile. Recovery and the tui runner do not call create_review."
    requirement: PUB-11
    verification:
      - kind: unit
        ref: "src/publish/tests/crash.rs#crash_after_sending_is_unknown"
        status: pass
    human_judgment: false
  - id: D2
    description: "A crash before sending requeues the same publish job and leaves the publication pending. The resumed run calls create_review once and ends submitted."
    requirement: PUB-10
    verification:
      - kind: unit
        ref: "src/publish/tests/crash/before.rs#crash_before_sending_posts_once"
        status: pass
    human_judgment: false
  - id: D3
    description: "A 30 second timeout or HTTP 500 leaves unknown, records outcome unknown, posts once, and blocks confirm_publication. HTTP 422 rejects and does not retry. A missing marker leaves unknown. A found marker stores that review id and submits."
    requirement: PUB-14
    verification:
      - kind: unit
        ref: "src/publish/tests/crash/outcome.rs#timeout_is_unknown"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/outcome.rs#server_error_is_unknown"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/outcome.rs#client_error_rejects"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/marker.rs#reconcile_missing_marker_stays_unknown"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/marker.rs#reconcile_marker_submits"
        status: pass
    human_judgment: false
  - id: D4
    description: "y runs reconcile for the selected target, including an open draft. A marker draws submitted. A missing marker stays unknown. y on any other state inserts nothing. b highlights Leave unknown and that choice writes nothing. Abandon sets abandoned with reason confirmed by me and does not post. b on another state does not ask."
    requirement: PUB-03
    verification:
      - kind: unit
        ref: "src/publish/tests/crash/keys.rs#y_marker_draws_submitted"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/keys.rs#y_missing_marker_stays_unknown"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/keys.rs#y_on_the_open_draft_submits"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/keys.rs#y_when_not_unknown_does_nothing"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/abandon.rs#b_safe_choice_leaves_unknown"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/abandon.rs#b_abandon_sets_confirmed_by_me"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/abandon.rs#b_on_the_open_draft_abandons"
        status: pass
      - kind: unit
        ref: "src/publish/tests/crash/abandon.rs#b_on_other_state_does_not_ask"
        status: pass
    human_judgment: false
duration: 43min
completed: 2026-11-10
status: complete
---

# Phase 04 Plan 07: Unknown publication Summary

**A crash or timeout after create_review leaves one unknown publication, and y or b settles it without a second POST**

## Performance

- **Duration:** 43 min
- **Started:** 2026-11-10T18:40:40Z
- **Completed:** 2026-11-10T19:23:42Z
- **Tasks:** 3
- **Files modified:** 29

## Accomplishments

- Recovery of a sent publication records unknown, fails the job with `outcome unknown`, and queues one reconcile without calling `create_review`.
- `create_review` is bounded at 30 seconds. A timeout or HTTP 500 stays unknown and blocks a new publish. HTTP 422 rejects. The POST is not retried.
- Reconcile submits only when one of the owner's reviews contains the publication marker. A miss stays unknown.
- `y` runs that reconcile from the queue or an open draft. `b` highlights Leave unknown. Abandon writes `confirmed by me`.

## Task Commits

Each task was committed atomically:

1. **Task 1: A crash after sending stays unknown and does not POST again** - `52ed46d` (feat)
2. **Task 2: Resume before sending, and block after a timeout** - `1af68ed` (feat)
3. **Task 3: y reconciles and b abandons only on purpose** - `d62d024` (feat)

## Files Created/Modified

- `src/crash.rs` - in-process crash points for before and after sending
- `src/store/recovery/service.rs` - settle running publish, reconcile, and sync jobs, then queue one reconcile per unknown publication
- `src/publish/post.rs` - hang points around `mark_sending`, then record the `create_review` outcome
- `src/publish/outcome/mod.rs` - 30 second timeout and the unknown, rejected, and submitted outcomes
- `src/publish/reconcile/mod.rs` - list reviews, search for the publication marker, never POST
- `src/store/publish.rs` - `abandon_unknown`, `unknown_publication`, and `enqueue_reconcile`
- `src/tui/runner.rs` - start publish and reconcile; stop before starting sync
- `src/tui/keys.rs` - `y` reconciles and `b` asks to abandon
- `src/publish/tests/crash.rs` - crash, timeout, reconcile, and tui cases

## Decisions Made

- In-process crash points use an `AtomicU8` and `Hold` because `std::env::set_var` is unsafe in edition 2024. `DIFFMATE_CRASH_POINT` still hangs a child process on the same point names.
- Pending is the only running publish that is requeued. `sending` and `unknown` fail with `outcome unknown`. `submitted` succeeds the job. `rejected` and `abandoned` fail without a second POST.
- Reconcile updates a publication only from `unknown`. A missing marker or a failed list leaves that state. `abandon_unknown` is the other way out, and it refuses every other state.
- The tui runner now executes reconcile. Sync still ends the loop without `start_job`. This replaces the earlier decision that any non-publish service job ended the loop.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] In-process crash points cannot set the environment**
- **Found during:** Task 1 (crash after sending)
- **Issue:** Edition 2024 makes `std::env::set_var` unsafe, and `unsafe_code` is forbidden. The plan's tracer sets `DIFFMATE_CRASH_POINT`.
- **Fix:** An `AtomicU8` arms the same point names inside the process. The environment variable still hangs a child.
- **Files modified:** `src/crash.rs`
- **Verification:** `crash_after_sending_is_unknown` and `crash_before_sending_posts_once`
- **Committed in:** `52ed46d`

**2. [Rule 3 - Blocking] New files had to stay at or under 100 added lines**
- **Found during:** Task 1 and Task 2
- **Issue:** The pre-commit hook rejects a staged file with more than 100 insertions. Recovery settlement and the outcome SQL did not fit in one new file.
- **Fix:** Writers live in `src/store/recovery/service/write.rs`, `src/publish/outcome/settle.rs`, and `src/publish/reconcile/write.rs`. `execute_reconcile` remains the entry named by the plan.
- **Files modified:** `src/store/recovery/service.rs`, `src/publish/outcome/mod.rs`, `src/publish/reconcile/mod.rs`
- **Verification:** hook `lint:large-files` passed on `52ed46d` and `1af68ed`
- **Committed in:** `52ed46d`, `1af68ed`

**3. [Rule 2 - Missing critical behavior] A failed review list must not reject the publication**
- **Found during:** Task 2 (reconcile)
- **Issue:** The behavior block covers a found marker and a missing marker. A login or `list_my_reviews` error had no recorded outcome, so the job would have stayed queued and the runner would have stopped on `Err`.
- **Fix:** The job is marked failed with the error text and the publication stays unknown. `execute_reconcile` returns `Ok` after that write.
- **Files modified:** `src/publish/reconcile/write.rs`
- **Verification:** `reconcile_marker_submits` and `reconcile_missing_marker_stays_unknown`; clippy `-D warnings`
- **Committed in:** `1af68ed`

**4. [Rule 1 - Bug] The before-sending wait could abort before the hang**
- **Found during:** Task 2
- **Issue:** The first version of the wait treated "still pending and no posts" as the crash point. That is true before the task is polled, so disarm could let it post.
- **Fix:** The test waits until the crash point is parked, aborts, joins, and only then drops `Hold`.
- **Files modified:** `src/crash.rs`, `src/publish/tests/crash/before.rs`
- **Verification:** `crash_before_sending_posts_once`
- **Committed in:** `1af68ed`

**5. [Rule 1 - Bug] `list_my_reviews` made the reviews module's dead-code expectation unfulfilled**
- **Found during:** Task 3
- **Issue:** The binary now calls `list_my_reviews` and `current_login` from reconcile. `src/gh/reviews.rs` expected unused items.
- **Fix:** Removed `expect(dead_code)` from that module. `Intent.lock` was also removed after `drive` started reading the session lock path.
- **Files modified:** `src/gh/reviews.rs`, `src/tui/model.rs`, `src/tui/runner.rs`
- **Verification:** `cargo clippy --locked --all-targets --all-features -- -D warnings`
- **Committed in:** `d62d024`

**Total deviations:** 5 auto-fixed (2 Rule 3, 1 Rule 2, 2 Rule 1)
**Impact on plan:** The behavior block is unchanged. The splits and the crash-point arm exist so the build and the hook can accept the code.

## TDD notes

`workflow.tdd_mode` is false and each task has one named `feat` subject, so the failing tests and the implementation share that commit.

- Task 2 RED: `timeout_is_unknown`, `client_error_rejects`, `server_error_is_unknown`, `reconcile_marker_submits`, `reconcile_missing_marker_stays_unknown`, and `abandon_unknown_sets_confirmed_by_me` failed. `crash_before_sending_posts_once` already passed because Task 1 requeues a pending publish.
- Task 3 RED: `y_marker_draws_submitted`, `y_missing_marker_stays_unknown`, `y_on_the_open_draft_submits`, `b_safe_choice_leaves_unknown`, `b_abandon_sets_confirmed_by_me`, and `b_on_the_open_draft_abandons` failed. `y_when_not_unknown_does_nothing` and `b_on_other_state_does_not_ask` already passed because those keys were ignored.

## Issues Encountered

None that remain. The hook suite on `d62d024` ran 482 tests, 482 passed, 11 skipped. `mise run lint` passed in that same hook (fmt, clippy `-D warnings`, cargo-machete).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 08 of phase 04 is next. Phase 5 was not started. Phase 2 and phase 3 acceptance was not changed.

`PUB-03` is shared with plan 04-04. This plan covers reconcile and abandon. The requirements handler decides whether the checkbox is checked.

## Self-Check: PASSED

- FOUND: `52ed46d`
- FOUND: `1af68ed`
- FOUND: `d62d024`
- FOUND: `src/publish/reconcile/mod.rs`
- FOUND: `src/publish/outcome/mod.rs`
- FOUND: `src/publish/tests/crash.rs` contains `crash_after_sending_is_unknown`
- FOUND: `.planning/phases/04-review-and-publish/04-07-SUMMARY.md`

---
*Phase: 04-review-and-publish*
*Completed: 2026-11-10*
