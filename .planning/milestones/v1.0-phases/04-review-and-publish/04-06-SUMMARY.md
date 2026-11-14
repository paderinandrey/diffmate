---
phase: 04-review-and-publish
plan: 06
subsystem: tui
tags: [ratatui, tokio, sqlite, gh, publish]
requires:
  - phase: 04-review-and-publish
    provides: "confirm_publication, execute_publish, and the publish-only tui runner"
provides:
  - "A stale draft asks before sending. Don't send creates nothing. Send anyway stores the fetched head as confirmed_sha and posts commit_id as the run head."
  - "A runner fetch that differs from confirmed_sha sets needs_confirmation and leaves the publication pending. Decline abandons it. Redelivery does not revive it."
  - "A review of mine in state PENDING stops publication before create_review. Transient gh checks retry. 401 and 403 abandon immediately."
affects: [04-review-and-publish, tui, publish]
actuals:
  tokens: 12734
  tasks: 2
  commits: 2
plan_head_before: eada59dfce523fd030dd22b80f792c4f31a883b1
plan_head_after: 359493f0afcab15cce75ac21c7944923517b4651
tech-stack:
  added: []
  patterns:
    - "Publish checks run in gate::before_send before mark_sending. create_review is not retried."
    - "A moved head is stored on jobs.head_sha. reconfirm updates confirmed_sha and queues the same publication. decline abandons it in one transaction."
    - "HTTP status is the first 3-digit token from 100 through 599 in Error::Command.detail. 408, 429, and 500-599 are transient. Any other 4xx is terminal. No token is transient."
key-files:
  created:
    - src/publish/gate.rs
    - src/store/publish/confirm.rs
    - src/publish/tests/confirm/moved.rs
    - src/publish/tests/confirm/reviews.rs
    - src/publish/tests/confirm/budget.rs
  modified:
    - src/tui/post.rs
    - src/tui/model.rs
    - src/tui/confirm.rs
    - src/publish/post.rs
    - src/publish/checks.rs
    - src/publish/load.rs
    - src/store/publish.rs
    - src/tui/runner.rs
    - src/gh.rs
key-decisions:
  - "Don't send is highlighted for a stale draft and for a head that moved after confirm. Send anyway stores the fetched head as confirmed_sha. commit_id stays the run head."
  - "Decline sets the job cancelled and the publication abandoned with reason declined, in one transaction. Delivering that job again fails it and leaves the publication abandoned."
  - "A pending review matches the exact login and the exact state PENDING. The drawn sentence is you already have a pending review on this pull request. The listed review is not modified."
  - "pr_view, current_login, and list_my_reviews retry up to Settings::max_retries extra attempts, sleeping 1s then 5s on a paused clock. 401 and 403 abandon on the first failure. create_review is not retried."
patterns-established:
  - "surface() after drive opens the moved confirm or draws an abandoned reason. Accepting a moved head calls Store::reconfirm and drives the same publication again."
  - "Schema user_version stays 5. needs_confirmation reuses jobs.head_sha. There is still one publication per draft."
requirements-completed: [PUB-07, PUB-08, PUB-12]
coverage:
  - id: D1
    description: "A stale draft asks with Don't send highlighted. That choice creates no publication and no create_review. Send anyway stores the fetched head as confirmed_sha and posts commit_id as the run head."
    requirement: PUB-07
    verification:
      - kind: unit
        ref: "src/publish/tests/confirm.rs#stale_highlight_does_not_send"
        status: pass
    human_judgment: false
  - id: D2
    description: "When the runner fetch differs from confirmed_sha, the job is needs_confirmation, the publication stays pending, and tui asks again. Accepting the new head posts the run head. No POST happens before that accept."
    requirement: PUB-07
    verification:
      - kind: unit
        ref: "src/publish/tests/confirm/moved.rs#head_moved_needs_confirmation"
        status: pass
      - kind: unit
        ref: "src/publish/tests/confirm/accept.rs#accepting_the_moved_head_posts_the_run"
        status: pass
    human_judgment: false
  - id: D3
    description: "Declining needs_confirmation cancels the job and abandons the publication with reason declined. Running that job again fails it, leaves the publication abandoned, and does not insert another publication or call create_review."
    requirement: PUB-12
    verification:
      - kind: unit
        ref: "src/publish/tests/confirm/moved.rs#decline_cancels_and_abandons"
        status: pass
      - kind: unit
        ref: "src/publish/tests/confirm/moved.rs#redelivered_job_stays_abandoned"
        status: pass
    human_judgment: false
  - id: D4
    description: "A review whose author equals the current login and whose state is exactly PENDING abandons the publication, draws you already have a pending review on this pull request, and does not call create_review or change the listed review. An empty list, another author, APPROVED, a lowercase state, and a padded login still post."
    requirement: PUB-08
    verification:
      - kind: unit
        ref: "src/publish/tests/confirm/reviews.rs#pending_review_of_mine_posts_nothing"
        status: pass
      - kind: unit
        ref: "src/publish/tests/confirm/reviews.rs#empty_review_list_does_not_block"
        status: pass
      - kind: unit
        ref: "src/publish/tests/confirm/reviews.rs#lowercase_pending_does_not_match"
        status: pass
      - kind: unit
        ref: "src/publish/tests/confirm/reviews.rs#other_author_and_approved_do_not_block"
        status: pass
      - kind: unit
        ref: "src/publish/tests/confirm/reviews.rs#padded_login_and_state_do_not_match"
        status: pass
    human_judgment: false
  - id: D5
    description: "Transient pr_view and list_my_reviews failures retry twice, sleeping 1s then 5s, then abandon with no create_review. HTTP 401 and 403 abandon on the first call with no sleep."
    requirement: PUB-08
    verification:
      - kind: unit
        ref: "src/publish/tests/confirm/budget.rs#transient_checks_sleep_then_abandon"
        status: pass
      - kind: unit
        ref: "src/publish/tests/confirm/budget.rs#auth_status_abandons_without_sleep"
        status: pass
      - kind: unit
        ref: "src/publish/tests/confirm/budget.rs#http_status_classifies_retries"
        status: pass
    human_judgment: false
duration: 41min
completed: 2026-11-10
status: complete
---

# Phase 4 Plan 6: Stale head and pending review Summary

**A stale or moved head asks before send, decline sticks, and a pending review of mine is left untouched**

## Performance

- **Duration:** 41 min
- **Started:** 2026-11-10T17:54:44Z
- **Completed:** 2026-11-10T18:36:00Z
- **Tasks:** 2
- **Files modified:** 28

## Accomplishments

- A stale draft opens a confirm whose highlighted choice is Don't send. That choice writes no publication. Send anyway stores the fetched head as `confirmed_sha` and the later `commit_id` is the run head.
- If the runner's next fetch differs from `confirmed_sha`, the job becomes `needs_confirmation`, the publication stays pending, and tui asks again. Accepting calls `Store::reconfirm`. Declining abandons with reason `declined`. Delivering that job again fails it and does not insert a publication.
- `list_my_reviews` runs before the sending commit. Author and state `PENDING` must match exactly. The notice is `you already have a pending review on this pull request`. Transient gh failures retry. 401 and 403 abandon immediately. `create_review` is not retried.

## Task Commits

Each task was committed atomically:

1. **Task 1: A stale draft does not send until the owner accepts** - `989adb3` (feat)
2. **Task 2: Head moved, decline, and a pending review of mine** - `359493f` (feat)

**Plan metadata:** docs commit follows this summary.

Task 2 declared `tdd="true"`. The plan type stays `execute`, and the phase commit rule is one commit per task. The RED run failed to compile (`Store::reconfirm` was missing and `execute_publish` did not yet take `max_retries`). That failure was not committed. Tests and the implementation landed together in `359493f`.

## Files Created/Modified

- `src/tui/post.rs` - stale ask before confirm; after drive, surface a moved head or an abandoned reason; Send anyway on a moved head calls `reconfirm`
- `src/tui/confirm.rs` - stale and moved confirms, both with Don't send highlighted
- `src/tui/model.rs` - Don't send on a moved head calls `Store::decline`
- `src/publish/gate.rs` - head check, pending-review check, and retry before `mark_sending`
- `src/publish/checks.rs` - exact pending-review match and HTTP status classification
- `src/publish/post.rs` - `execute_publish` takes `max_retries` and returns before sending when the gate says so
- `src/store/publish/confirm.rs` - `needs_confirmation`, `reconfirm`, `decline`, and `abandon` in Immediate transactions where more than one row changes
- `src/publish/tests/confirm.rs` - `stale_highlight_does_not_send`
- `src/publish/tests/confirm/moved.rs` - moved head, decline, and redelivery
- `src/publish/tests/confirm/reviews.rs` - pending review of mine and the non-matches
- `src/publish/tests/confirm/budget.rs` - paused-time retries and terminal 401 and 403

## Decisions Made

- The highlighted choice never sends. Send anyway on a stale draft confirms the fetched head and drives. `commit_id` remains `runs.head_sha`.
- A head that moves after confirm does not POST. The seen sha is stored on the existing `jobs.head_sha` column. `user_version` stays 5.
- Accepting that head updates `confirmed_sha` and queues the same job. The snapshot is unchanged. There is still one publication per draft.
- Decline is one transaction: job `cancelled`, publication `abandoned`, reason `declined`. An abandoned publication short-circuits `execute_publish` with `fail_job` and no GitHub call, so the reason stays `declined`.
- Pending means author equals `current_login` and state equals `PENDING`. Comparison is not trimmed, not case-folded, and not based on body length.
- Retry budget is `Settings::max_retries` extra attempts (default 2), with sleeps of 1s then 5s. Status 408, 429, and 500 through 599 are transient. Any other 4xx, including 401 and 403, abandons immediately. A `Command` error with no HTTP status is transient. Other error variants are terminal. `create_review` is a single attempt.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Stale ask uses the existing confirm keys**
- **Found during:** Task 1 (stale draft)
- **Issue:** The plan named `src/tui/keys.rs`. Enter, Left, and Right already drive the confirm modal.
- **Fix:** The stale prompt is opened from `post::send` and `Session::ask_stale`. `keys.rs` is unchanged.
- **Files modified:** `src/tui/post.rs`, `src/tui/model.rs`, `src/tui/confirm.rs`
- **Verification:** `stale_highlight_does_not_send` passed in the task 1 hook (454 tests) and again in the task 2 hook.
- **Committed in:** `989adb3`

**2. [Rule 3 - Blocking] Scripted gh must answer the new checks**
- **Found during:** Task 2 (head moved and pending review)
- **Issue:** `Script` left `pr_view`, `current_login`, and `list_my_reviews` on the trait defaults. That error has no HTTP status, so the gate would sleep and then abandon every existing publish test.
- **Fix:** `Script::pr_view` returns the publication's `confirmed_sha`. Login is `tester` and the review list is empty. The default-method assertion moved to a `Plain` impl that still uses the trait defaults.
- **Files modified:** `src/publish/tests/script.rs`
- **Verification:** Existing publish tests passed in the task 2 hook (466 tests).
- **Committed in:** `359493f`

**3. [Rule 1 - Bug] `Confirm::moved` cannot be both a constructor and a predicate**
- **Found during:** Task 2
- **Issue:** `Confirm::moved(prompt)` is the constructor. A method of the same name does not compile.
- **Fix:** The predicate is `is_moved`.
- **Files modified:** `src/tui/confirm.rs`, `src/tui/model.rs`
- **Verification:** `decline_cancels_and_abandons` passed.
- **Committed in:** `359493f`

**4. [Rule 3 - Blocking] `serve` takes the retry budget**
- **Found during:** Task 2
- **Issue:** `execute_publish` gained `max_retries`. `runner::serve` and one scripted call site still used the old arity.
- **Fix:** `serve` and `drive` pass `Settings::max_retries`. The refusal test passes `2`.
- **Files modified:** `src/tui/runner.rs`, `src/tui/tests/refusals.rs`
- **Verification:** `second_c_while_pending_posts_once` passed in the full hook.
- **Committed in:** `359493f`

**5. [Rule 1 - Bug] `expect(dead_code)` on the gh defaults was unfulfilled**
- **Found during:** Task 2
- **Issue:** `gate` calls `current_login` and `list_my_reviews`, so the non-test build warned that the `dead_code` expectation did not fire. Clippy denies warnings.
- **Fix:** Removed both `expect(dead_code)` attributes.
- **Files modified:** `src/gh.rs`
- **Verification:** `cargo clippy --locked --all-targets --all-features -- -D warnings` exited 0, and the commit hook lint step passed.
- **Committed in:** `359493f`

**6. [Rule 3 - Blocking] Confirm tests are split so the 100-line hook can pass**
- **Found during:** Task 2
- **Issue:** One confirm test file, plus a gh fake that fails on a status code, would add more than 100 lines to a single new file.
- **Fix:** Cases live in `confirm/moved.rs`, `confirm/reviews.rs`, `confirm/accept.rs`, `confirm/budget.rs`, and `confirm/boom.rs`. Each new file is at or under 100 lines.
- **Files modified:** `src/publish/tests/confirm/`
- **Verification:** The hook large-files step passed. `mise run --force test -- -E 'test(confirm::)' --no-tests=fail` ran 13 tests, 13 passed.
- **Committed in:** `359493f`

---

**Total deviations:** 6 auto-fixed (2 bug, 4 blocking)
**Impact on plan:** The checks, the confirm, and the store transitions are the ones the plan required. The splits and the scripted gh answers keep the existing suite and the line hook working. No second publication identity was added.

## Issues Encountered

Task 2's first test compile failed before `reconfirm` and the fourth `execute_publish` argument existed. That RED was a compile error, not a committed test commit. The named tests then passed: `head_moved_needs_confirmation`, `decline_cancels_and_abandons`, `redelivered_job_stays_abandoned`, `pending_review_of_mine_posts_nothing`, `empty_review_list_does_not_block`, and `lowercase_pending_does_not_match`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 7 of phase 4 can start. `user_version` is still 5. tui still does not run review jobs. Phase 2 and phase 3 are not accepted by this plan.

## Self-Check: PASSED

- FOUND: `.planning/phases/04-review-and-publish/04-06-SUMMARY.md`
- FOUND: `src/publish/gate.rs`
- FOUND: `src/store/publish/confirm.rs`
- FOUND: `src/publish/checks.rs`
- FOUND: `src/tui/post.rs`
- FOUND: `989adb3`
- FOUND: `359493f`
