---
phase: 04-review-and-publish
plan: 08
subsystem: publish
tags: [tokio, rusqlite, gh, graphql, watch, ratatui]
requires:
  - phase: 04-review-and-publish
    provides: "execute_publish, execute_reconcile, and the submitted publication row from plans 04-06 and 04-07"
provides:
  - "Watch runs one due publish, reconcile, or sync job on the existing one-second lock poll while an agent run stays in flight."
  - "A submitted publication queues one sync job. Comment sync links inline findings by marker, reads isResolved, and retries the same job after 60 seconds without create_review."
  - "previous.json thread_status for a linked inline finding comes from a fresh GraphQL read. A failed read writes unknown. Body findings stay none."
affects: [04-review-and-publish, watch, publish, context]
actuals:
  tokens: 14210
  tasks: 3
  commits: 3
plan_head_before: 0c1dc5434cbec5dd10b1cd4bbc4c1d27b323fbd4
plan_head_after: f70267398fe8c0f05242cfd188cf32ff9c50bd5d
tech-stack:
  added: []
  patterns:
    - "next_service_job selects one queued publish, reconcile, or sync job. next_runnable_job stays review-only."
    - "Comment sync delay is next_attempt_at = now_ms() + 60000. The watch tick does not sleep for that delay."
    - "Review comment ids and reviewThreads.isResolved are separate gh api graphql queries. The discussion query is unchanged."
key-files:
  created:
    - src/publish/sync.rs
    - src/gh/threads.rs
    - src/watch/service_tests.rs
    - src/context/tests/previous_status.rs
  modified:
    - src/watch/run.rs
    - src/store/publish.rs
    - src/publish/outcome/settle.rs
    - src/gh.rs
    - src/context/previous.rs
    - src/store/previous.rs
key-decisions:
  - "A due service job runs inside the same select that owns the agent future. Shutdown uses request_stop so a failed stop-reason write retries and the agent stays owned."
  - "The sync job is inserted in the transaction that sets the publication submitted, including reconcile. NOT EXISTS blocks a second queued or running sync for that publication."
  - "A failed comment list writes no links. A failed isResolved read keeps links and leaves thread_status unknown. Both requeue the same job. create_review is not called."
  - "previous.json calls thread_resolved only when a linked inline finding exists. An error or a missing thread id writes unknown and context write still succeeds. The decision field is the stored finding decision."
patterns-established:
  - "Finding marker text is <!-- diffmate:finding=<id> -->. Sync links only kept or edited findings whose line is not null."
  - "Schema user_version stays 5. No new crate and no new config key. 401 and 403 stay terminal on publish."
requirements-completed: [PUB-09, PUB-13, PUB-15]
coverage:
  - id: D1
    description: "While watch is inside an agent run, a queued publish finishes, create_review runs once, and the review job is still running. With no service job, the review still completes."
    requirement: PUB-09
    verification:
      - kind: unit
        ref: "src/watch/service_tests.rs#publish_finishes_during_an_agent_run"
        status: pass
      - kind: integration
        ref: "tests/watch binary via mise run --force test -- -E 'binary(watch)'"
        status: pass
    human_judgment: false
  - id: D2
    description: "A failed comment sync leaves the publication submitted and inline thread status unknown, then retries the same job 60 seconds later without create_review. A later success links the marker and reads isResolved."
    requirement: PUB-15
    verification:
      - kind: unit
        ref: "src/publish/tests/sync.rs#sync_failure_stays_submitted"
        status: pass
      - kind: unit
        ref: "src/publish/tests/sync/link.rs#sync_links_marker_without_post"
        status: pass
      - kind: unit
        ref: "src/publish/tests/sync/link.rs#sync_status_read_failure_keeps_unknown"
        status: pass
    human_judgment: false
  - id: D3
    description: "tui still does not start a review job. Watch can publish during an agent run. The queue runner label was not changed."
    requirement: PUB-13
    verification:
      - kind: unit
        ref: "src/watch/service_tests.rs#publish_finishes_during_an_agent_run"
        status: pass
    human_judgment: false
  - id: D4
    description: "A linked inline finding is resolved in previous.json when GraphQL says so, a body finding is none, and a failed read writes unknown without failing the review."
    requirement: PUB-15
    verification:
      - kind: unit
        ref: "src/context/tests/previous_status.rs#linked_thread_is_resolved_and_a_body_finding_stays_none"
        status: pass
    human_judgment: false
duration: 53min
completed: 2026-11-11
status: complete
---

# Phase 4 Plan 8: Service Sync Summary

**Watch publishes and syncs comments during an agent run, retries a failed sync without another POST, and refreshes previous.json thread status from GraphQL.**

## Performance

- **Duration:** 53 min
- **Started:** 2026-11-10T23:52:43Z
- **Completed:** 2026-11-11T00:46:00Z
- **Tasks:** 3
- **Files modified:** 33

## Accomplishments

- Watch claims one due publish, reconcile, or sync job on the existing one-second lock poll and leaves the in-flight review running.
- Comment sync links inline findings by the finding marker, then reads `reviewThreads.isResolved`. Failure requeues that same job 60 seconds later and does not call `create_review`.
- `previous.json` writes `resolved` or `unresolved` from a fresh GraphQL read of linked inline findings. A failed read writes `unknown`. Body findings stay `none`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Publish during a running review** - `74223f5` (feat)
2. **Task 2: Comment sync retries without a new POST** - `49c237c` (feat)
3. **Task 3: previous.json reads thread status from GraphQL** - `f702673` (feat)

## Files Created/Modified

- `src/watch/run.rs` - polls a service job while the agent future is pending, then runs sync
- `src/store/publish.rs` - `next_service_job` and `enqueue_sync`
- `src/publish/sync.rs` - `execute_sync`
- `src/gh/threads.rs` - GraphQL comment and thread-status reads
- `src/publish/outcome/settle.rs` - queues one sync job when publish submits
- `src/context/previous.rs` - fresh thread status in `previous.json`
- `src/store/previous.rs` - loads `github_thread_id` for that write

## Decisions Made

- The review future is built inside the watch select. Returning it across a function boundary hit RPIT lifetime errors.
- Shutdown during that wait calls `request_stop`, which retries a failed stop-reason write and then cancels the agent.
- Sync delay is `next_attempt_at`, compared with wall-clock `now_ms()`. A 60 second tokio sleep inside the tick would stall the in-flight review. Tests use `start_paused` and assert the 60000 millisecond gap.
- Reconcile's transition to submitted queues sync in that same transaction, so a recovered publication is not left without a sync job.
- `thread_resolved` runs only when at least one kept or edited inline finding already has a thread id. Unlinked inline findings stay `unknown` without a GraphQL call.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split the watch service test so each file stays within the 100-insertion hook**
- **Found during:** Task 1
- **Issue:** One `service_tests.rs` was over 100 added lines, which `hk` rejects.
- **Fix:** Parent module plus `remote.rs` and `rig.rs`.
- **Files modified:** `src/watch/service_tests.rs`, `src/watch/service_tests/remote.rs`, `src/watch/service_tests/rig.rs`
- **Verification:** `mise run --force test -- -E 'test(service_tests) | binary(watch)'` passed, 40 tests, then the commit hook passed 484 tests.
- **Committed in:** `74223f5`

**2. [Rule 1 - Bug] Shutdown must retry a failed stop-reason write**
- **Found during:** Task 1
- **Issue:** A one-shot `set_stop_reason` returned the injected database error and never printed `warning: storing stop reason failed`. The persistence test failed.
- **Fix:** The in-flight path calls the existing `request_stop` helper, now `pub(crate)`.
- **Files modified:** `src/watch/run.rs`, `src/review/signals.rs`
- **Verification:** `tests/watch/persistence.rs` passed inside the watch binary filter.
- **Committed in:** `74223f5`

**3. [Rule 2 - Missing] Watch must run the sync job once `execute_sync` exists**
- **Found during:** Task 2
- **Issue:** Task 1 left unknown service kinds, including sync, sleeping one poll without `start_job`, because the executor did not exist yet.
- **Fix:** The sync arm calls `start_job` and `execute_sync`.
- **Files modified:** `src/watch/run.rs`
- **Verification:** Plan filter includes `binary(watch)` and `test(service_tests)`; both passed after the change.
- **Committed in:** `49c237c`

**4. [Rule 2 - Missing] Reconcile submit also queues comment sync**
- **Found during:** Task 2
- **Issue:** Only the publish settle transaction was going to insert the sync job. A publication that becomes submitted through reconcile would never sync.
- **Fix:** `insert_sync_job` runs in both transactions, and only when the submitted update changes one row.
- **Files modified:** `src/publish/outcome/settle.rs`, `src/publish/reconcile/write.rs`, `src/store/publish/sync.rs`
- **Verification:** `sync_does_not_enqueue_twice` sees one active sync after publish, and a second insert does not add another.
- **Committed in:** `49c237c`

**5. [Rule 3 - Blocking] Split sync, GraphQL parse, and the context test for the 100-insertion hook**
- **Found during:** Tasks 2 and 3
- **Issue:** A single new test or implementation file exceeded 100 added lines.
- **Fix:** `sync.rs` plus `apply.rs`, `apply/write.rs`, `threads/parse.rs`, and the sync test modules. The previous.json case is `previous_status.rs`.
- **Files modified:** those new files
- **Verification:** Each staged file's numstat additions were at or under 100. Both task hooks passed `large-files`.
- **Committed in:** `49c237c`, `f702673`

**6. [Rule 2 - Missing] Load `github_thread_id` for previous.json**
- **Found during:** Task 3
- **Issue:** The stored finding row did not include the thread id the fresh read has to match.
- **Fix:** `latest_succeeded_before` selects `github_thread_id`.
- **Files modified:** `src/store/previous.rs`
- **Verification:** `linked_thread_is_resolved_and_a_body_finding_stays_none` passed.
- **Committed in:** `f702673`

### TDD

Tasks 2 and 3 used one `feat` commit each, which is the project commit rule. `workflow.tdd_mode` is false and the plan type is `execute`, so there is no separate RED commit.

Task 2 RED, before `execute_sync` linked or requeued: `sync_failure_stays_submitted` asserted gap `-1` against `60000`; the link tests failed reading a null `github_comment_id`; `sync_does_not_enqueue_twice` saw `0` sync jobs. After the implementation, `mise run --force test -- -E 'test(sync::) | test(gh::)'` passed 24 tests.

Task 3 RED, with the writer still hardcoding inline status: `linked_thread_is_resolved_and_a_body_finding_stays_none` reached `left: unknown` `right: resolved` after the failed-read assertions had passed. After the GraphQL lookup, `mise run --force test -- -E 'test(sync::) | test(context::)'` passed 22 tests.

---

**Total deviations:** 6 auto-fixed (2 Rule 1/2 correctness, 4 Rule 2/3 blocking or missing). The RPIT lifetime fix is recorded under Issues.
**Impact on plan:** The splits are the hook limit. The other fixes keep shutdown, reconcile, and previous.json on the behavior the plan describes.

## Issues Encountered

Returning `impl Future` for the in-flight review from `job::launch` failed to name the anonymous lifetimes. The future is now created in `alongside`, and `job::complete` still finishes it.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 04-08 is the last plan in phase 4. Phase 5 was not started. `tui` still does not run review jobs. Schema `user_version` stays 5.

## Self-Check: PASSED

- FOUND: `74223f5`
- FOUND: `49c237c`
- FOUND: `f702673`
- FOUND: `src/watch/run.rs`
- FOUND: `src/watch/service_tests.rs`
- FOUND: `src/publish/sync.rs`
- FOUND: `src/gh/threads.rs`
- FOUND: `src/context/previous.rs`
- FOUND: `src/context/tests/previous_status.rs`

## Known Stubs

None.

*Phase: 04-review-and-publish*
*Completed: 2026-11-11*
