---
phase: 03-queue
plan: 02
subsystem: github
tags: [gh, search, pagination, isDraft, rust]

requires:
  - phase: 03-queue
    provides: "Schema version 3; this plan does not write SQLite"
provides:
  - "Gh::search_review_requested lists directly requested open pull requests, drafts included"
  - "Search pages are typed only when url, number, and repository.nameWithOwner agree"
  - "PrMeta.is_draft from gh pr view, defaulting to false when isDraft is absent"
affects: [03-queue, poll, tracking]
actuals:
  tokens: 4315
  tasks: 2
  commits: 2
plan_head_before: f124782c1ea4451a2d0584a626c8559a99985ba9
plan_head_after: 4d9c98756c612a6b6b4e5888b016e4bc4c971513
commits: 2
tech-stack:
  added: []
  patterns:
    - "gh search prs is paged by raising --limit in steps of 100 and replacing the list with the later full response"
    - "One malformed search element is skipped; a non-array or a failed gh call fails the whole poll"
    - "A missing pr view isDraft deserializes as false; a missing search isDraft drops that element"
key-files:
  created:
    - src/gh/search.rs
    - src/gh/search/parse.rs
    - src/gh/search/tests.rs
    - src/gh/search/tests/malformed.rs
    - src/gh/search/tests/view.rs
    - src/gh/search/tests/pages.rs
    - src/gh/search/tests/args.rs
  modified:
    - src/gh.rs
    - src/gh/meta.rs
    - src/context/tests/fake_gh.rs
    - src/runner/resolve/tests/gh.rs
    - src/runner/agent_run/fakes.rs
key-decisions:
  - "Paging replaces the accumulated list because gh search has no page token and a higher --limit returns the whole prefix"
  - "Search elements require a boolean isDraft; pr view defaults a missing isDraft to false so existing fixtures still parse"
  - "dead_code is allowed on the new trait method until a later plan calls it"
patterns-established:
  - "Direct review requests are Gh::search_review_requested, not a shell and not a team qualifier"
requirements-completed: [QUE-04, QUE-05]
coverage:
  - id: D1
    description: "A search page keeps drafts and ready pull requests, and drops elements whose url, number, or repository do not agree. pr view exposes is_draft, false when the field is absent."
    requirement: QUE-04
    verification:
      - kind: unit
        ref: "src/gh/search/tests.rs#a_draft_and_a_ready_pull_request_are_kept"
        status: pass
      - kind: unit
        ref: "src/gh/search/tests/view.rs#pr_view_without_is_draft_is_false_and_true_stays_true"
        status: pass
      - kind: unit
        ref: "src/gh/search/tests/malformed.rs#malformed_elements_are_skipped_and_still_counted"
        status: pass
    human_judgment: false
  - id: D2
    description: "The search argv is --review-requested=@me with no --draft and no --team flag. A short page ends the search, a full page is replaced by the next higher limit, a failed page returns no partial list, and a full 1000 stops."
    requirement: QUE-05
    verification:
      - kind: unit
        ref: "src/gh/search/tests/args.rs#search_args_request_the_owner_and_not_a_team"
        status: pass
      - kind: unit
        ref: "src/gh/search/tests/pages.rs#a_full_page_is_replaced_by_the_next_shorter_page"
        status: pass
      - kind: unit
        ref: "src/gh/search/tests/pages.rs#a_failed_second_page_discards_the_first"
        status: pass
      - kind: unit
        ref: "src/gh/search/tests/pages.rs#a_full_thousand_stops_without_another_call"
        status: pass
    human_judgment: false
duration: 27min
completed: 2026-11-09
status: complete
---

# Phase 3 Plan 02: Direct Review Search Summary

**Paged `gh search prs --review-requested=@me` on the Gh trait, keeping drafts and rejecting search rows whose URL, number, and repository disagree**

## Performance

- **Duration:** 27 min
- **Started:** 2026-11-09T10:16:17Z
- **Completed:** 2026-11-09T10:43:09Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- `Gh::search_review_requested` runs `gh` with an argument vector: `search`, `prs`, `--review-requested=@me`, `--state=open`, `--limit`, and `--json` fields `number,repository,url,isDraft,state`. No `--draft` and no `--team` argument.
- The search starts at limit 100 and, while the raw array length equals the limit, calls again at limit + 100, replacing the previous list. It stops on a shorter array, or at 1000 even if that page is full. A process error or a non-array fails the call and discards pages already read.
- A draft stays in the list with `is_draft: true`. An element is kept only when `pr_url::parse` accepts the URL, the number matches, `repository.nameWithOwner` matches owner/repo ignoring case, and state is open ignoring case.
- `gh pr view` requests `isDraft`. Old fixture JSON without the field parses with `is_draft` false. The three existing Gh fakes return an empty list.

## Task Commits

Each task was committed atomically:

1. **Task 1: Parse one search page and request isDraft from pr view** - `e98f819` (feat)
2. **Task 2: Paged gh search through the Gh trait** - `4d9c987` (feat)

## Files Created/Modified

- `src/gh/search.rs` - Pager and `search_args`
- `src/gh/search/parse.rs` - One search page into `DirectRequest` values plus the raw array length
- `src/gh/search/tests.rs` - Kept and dropped search rows
- `src/gh/search/tests/malformed.rs` - Malformed elements skipped and still counted
- `src/gh/search/tests/view.rs` - `pr view` `isDraft` absent and true
- `src/gh/search/tests/pages.rs` - Limit steps, replacement, failed page, stop at 1000
- `src/gh/search/tests/args.rs` - Qualifier, JSON fields, no `--draft`, no `--team`
- `src/gh.rs` - `DirectRequest`, trait method, `GhCli` wiring through `command` and `run_tool`
- `src/gh/meta.rs` - `is_draft` on `PrMeta` with serde default
- `src/context/tests/fake_gh.rs` - Empty search result
- `src/runner/resolve/tests/gh.rs` - Empty search result and `is_draft: false`
- `src/runner/agent_run/fakes.rs` - Empty search result and `is_draft: false`

## Decisions Made

- A higher `--limit` returns the whole result prefix, so each successful page replaces the previous list instead of appending.
- Search JSON does not default `isDraft`: a missing or non-boolean value drops that element. `pr view` defaults a missing `isDraft` to false so current review fixtures still parse.
- Nothing in this plan calls the new method from `watch` or `review`, so `dead_code` is allowed on the trait method, `DirectRequest`, and the search module outside tests. The next plan that polls should remove those allows when it reads `is_draft`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split search tests so the 100-line hook accepts the commit**
- **Found during:** Task 1 (parse one search page) and Task 2 (paged search)
- **Issue:** `hk` rejects a commit that adds more than 100 lines to one file outside `.planning/` and `Cargo.lock`. One `tests.rs` could not hold the parse cases and the pager cases.
- **Fix:** Kept `src/gh/search/tests.rs` as the module root and moved cases into `malformed.rs`, `view.rs`, `pages.rs`, and `args.rs`.
- **Files modified:** `src/gh/search/tests.rs` and those submodules
- **Verification:** `mise run --force test -- -E 'test(gh::search::)'` — 13 passed
- **Committed in:** `e98f819` and `4d9c987`

**2. [Rule 3 - Blocking] Allow dead_code until a later plan calls the search**
- **Found during:** Task 1 (`PrMeta.is_draft`) and Task 2 (the trait method and search module)
- **Issue:** Clippy denies warnings. This plan must not enqueue or call GitHub from `watch`, so the new field and method have no production reader yet.
- **Fix:** `#[cfg_attr(not(test), allow(dead_code))]` on `PrMeta.is_draft`'s sibling field, on `DirectRequest`, and on `mod search`. Unconditional `#[allow(dead_code)]` on the trait method, which tests do not call either.
- **Files modified:** `src/gh.rs`, `src/gh/meta.rs`
- **Verification:** `cargo clippy --locked --all-targets --all-features -- -D warnings` exited 0
- **Committed in:** `e98f819` and `4d9c987`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both were required by the repo hooks. Search behavior matches the plan. No new crate and no SQLite writes.

## Issues Encountered

Task 1 compiled the parser only under `cfg(test)` so an unused production module would not fail clippy. Task 2 promotes that module and wires `GhCli`. The final tree always compiles the search module.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- A later poll can call `Gh::search_review_requested`. Drafts are present with `is_draft` true; the caller decides not to enqueue them. Team-shaped rows never become elements.
- A failed search is `Err`, not an empty `Ok`. The caller must not clear `request_seen` on that error.
- `PrMeta.is_draft` is available for the already-tracked `gh pr view` read. Remove the `dead_code` allows when production reads the field and calls the method.

## Self-Check: PASSED

- FOUND: `e98f819`
- FOUND: `4d9c987`
- FOUND: `src/gh/search.rs`
- FOUND: `src/gh/search/parse.rs`
- FOUND: `src/gh/search/tests.rs`
- FOUND: `src/gh/search/tests/malformed.rs`
- FOUND: `src/gh/search/tests/view.rs`
- FOUND: `src/gh/search/tests/pages.rs`
- FOUND: `src/gh/search/tests/args.rs`
- FOUND: `src/gh.rs`
- FOUND: `src/gh/meta.rs`

---
*Phase: 03-queue*
*Completed: 2026-11-09*
