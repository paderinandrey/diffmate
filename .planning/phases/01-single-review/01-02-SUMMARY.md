---
phase: 01-single-review
plan: 02
subsystem: testing
tags: [github, gh, fixtures, live-checks, fork-pr]

requires:
  - phase: 01-single-review
    provides: "01-CONTEXT.md decision that fixture PRs are owner-created, and SPEC.md fork handling through refs/pull/N/head"
provides:
  - "tests/fixtures/live-prs.json: URLs, head SHAs and expected anchors (calc.py line 7 RIGHT) for the live REV-03, REV-04 and REV-10 checks"
  - "Three open fixture PRs in the private repository paderinandrey/diffmate-fixtures, one of them from the fork tileforge/diffmate-fixtures"
affects: [01-12]

actuals:
  tokens: 160
  tasks: 2
  commits: 1

plan_head_before: 20639b8db5209d7701d2f1516ef145b396a5600a
plan_head_after: 6dd862744fc51da927788473e57668774a55c55f
commits: 1

tech-stack:
  added: []
  patterns:
    - "Fixture PR metadata is recorded once from read-only gh calls and consumed by #[ignore] live tests"

key-files:
  created:
    - tests/fixtures/live-prs.json
  modified: []

key-decisions:
  - "The owner explicitly asked the orchestrator to create the fixture repository, fork and PRs on 2026-10-27, superseding for this plan only the CONTEXT.md rule that Claude does not create repositories or PRs"
  - "Fixture repository and fork are PRIVATE; diffmate reads them through the owner's gh login, so the recorded data stays URLs, SHAs and anchors"

patterns-established:
  - "Head SHAs in live-prs.json are the SHAs observed at recording time; a live test must compare them with the PR before trusting the anchors"

requirements-completed: [REV-03, REV-04, REV-10]

coverage:
  - id: D1
    description: "Three open fixture PRs exist (obvious bug, clean, fork) and are recorded with head SHAs and, for both bug PRs, the expected anchor calc.py line 7 RIGHT"
    requirement: REV-03
    verification:
      - kind: other
        ref: "jq -e on tests/fixtures/live-prs.json (anchor, fork flag, clean URL pattern)"
        status: pass
      - kind: other
        ref: "gh pr view state check over all three recorded URLs"
        status: pass
    human_judgment: false
  - id: D2
    description: "The fork PR head is reachable only as a commit of the fork and through refs/pull/3/head of the base repository"
    requirement: REV-10
    verification:
      - kind: other
        ref: "gh pr view 3 isCrossRepository true; gh api git/ref/pull/3/head equals the recorded head_sha"
        status: pass
    human_judgment: false

duration: 15min
completed: 2026-10-27
status: complete
---

# Phase 1 Plan 2: Live fixture PRs Summary

**Three owner-approved fixture PRs (obvious off-by-one, clean README change, fork PR) were verified with read-only gh calls and recorded in tests/fixtures/live-prs.json with head SHAs and the calc.py line 7 RIGHT expected anchor for plan 01-12's live checks**

## Performance

- **Duration:** about 15 min for Task 2 and the close-out (Task 1 ran in the orchestrator before this continuation; start time was not recorded)
- **Tasks:** 2 (1 commit: Task 1 produced no repository change)
- **Files:** 1 created (25 lines)

## Accomplishments

- Task 1 (checkpoint:human-action) resolved by an owner decision: on 2026-10-27 the owner explicitly asked the orchestrator to create the fixtures itself. This supersedes, for this plan only, the CONTEXT.md rule that Claude does not create repositories or PRs.
- Fixture repository `paderinandrey/diffmate-fixtures` (PRIVATE, default branch `main`, `README.md` and `calc.py` as the plan specifies) and fork `tileforge/diffmate-fixtures` (PRIVATE, fork of the repository above).
- Task 2 verified each PR with read-only `gh pr view` and `gh api` GET calls and wrote `tests/fixtures/live-prs.json`.

## Fixture PRs

| Key | URL | Head branch | Head SHA |
|-----|-----|-------------|----------|
| obvious_bug | https://github.com/paderinandrey/diffmate-fixtures/pull/1 ("Add last_item") | obvious-bug | 5c5924713de3e990f738f022daf1ed031daf6378 |
| clean | https://github.com/paderinandrey/diffmate-fixtures/pull/2 ("Describe the fixtures") | clean-change | 3a226759707ca75b6105b177854bd907d90481e4 |
| fork | https://github.com/paderinandrey/diffmate-fixtures/pull/3 ("Add first_item") | tileforge:fork-bug | 3d5320d144aea2c7420a9c310df7d24a9901a9ee |

## Checks Observed (Task 2)

All results come from `gh pr view URL --json ...`, `gh api` GET of repository contents and pulls, and local comparison.

| Check | PR 1 obvious_bug | PR 2 clean | PR 3 fork |
|-------|------------------|------------|-----------|
| state is OPEN | pass | pass | pass |
| baseRefName is main | pass | pass | pass |
| isCrossRepository | false (pass) | false (pass) | true (pass; head repo tileforge/diffmate-fixtures) |
| changed files | only calc.py (+4 -0) | only README.md (+1 -0) | only calc.py (+4 -0) |
| content at head SHA | calc.py is byte-identical to the 7 lines the plan specifies; line 7 is `    return items[len(items)]` | README.md is byte-identical to the two expected lines | calc.py is byte-identical to the 7 lines the plan specifies; line 7 is `    return items[1]` |
| reviews | none | none | none |

Further observations:

- `gh api repos/paderinandrey/diffmate-fixtures/pulls/3` reports head repo `tileforge/diffmate-fixtures`, and `git/ref/pull/3/head` of the base repository resolves to the recorded head SHA `3d5320d...`.
- The recorded head SHAs equal the live `headRefOid` of each PR at recording time.
- `main` of the base repository holds exactly the 3-line `calc.py` and the one-line `README.md`; branches are `main`, `obvious-bug`, `clean-change`.
- Both Task 2 automated verify commands exited 0, `jq -r '.[].head_sha' ... | grep -cE '^[0-9a-f]{40}$'` printed 3, and `mise run lint:secrets` reported no leaks. The pre-commit hook (lint, large-files, secrets, docs, test, actions) passed on the task commit.

## Incident: first push reached the upstream repository

`gh repo create --push` pushed the checked-out `fork-bug` branch to the upstream repository and made it the default branch. The orchestrator pushed `main`, set `main` as the default branch and deleted `fork-bug` from upstream before forking. `fork-bug` now exists only in the fork `tileforge/diffmate-fixtures`. Branch listing of upstream confirmed after the fact: `clean-change`, `main`, `obvious-bug`, default branch `main`.

## Deviations from Plan

### Owner-directed changes

**1. [Owner decision] Orchestrator created the fixtures (Task 1)**
- **Found during:** Task 1
- **Issue:** The plan has the owner create the repository and PRs by hand; the owner asked the orchestrator to do it instead.
- **Fix:** Created as listed above; the owner is the author and assignee of the three PRs (the global assignee rule).
- **Files modified:** none in the repository
- **Commit:** none

**2. [Observation] Repository is private, not public**
- The plan allowed either. Private means `gh` needs the owner's login for every read, which is how diffmate runs anyway.

**Total deviations:** 0 auto-fixed, 1 owner decision. **Impact:** none on the recorded data.

## Issues Encountered

- A CodeRabbit bot (`coderabbitai[bot]`) posted one summary issue comment on each of the three PRs on 2026-10-26 (PR 1: walkthrough; PRs 2 and 3: "Review limit reached" notice). The task description said the PRs have no comments. The plan checks only require no reviews, and `reviews` is empty and there are no inline review comments on PR 1 (`pulls/1/comments` length 0). Plan 01-12 should know that each PR carries one bot issue comment, and the owner may want to uninstall the app from this repository so it does not post again. Nothing was changed on GitHub.
- `gh api ... --jq .content | base64 -d` on macOS truncated the decoded file after line 3 for the PR heads, which would have looked like a missing planted line. Decoding with `python3 -I` (and comparing byte-for-byte with the expected file) gave the correct 7-line files. No data was affected.

## Known Stubs

None.

## Threat Flags

None. The file holds only URLs, SHAs and anchors; gitleaks reported no leaks.

## Next Phase Readiness

Plan 01-12 can read `tests/fixtures/live-prs.json` for the live REV-03, REV-04 and REV-10 checks. The fixtures must stay unmodified: no pushes to the three branches and no merges.

## Self-Check: PASSED

- tests/fixtures/live-prs.json exists.
- Commit 6dd8627 is an ancestor of HEAD.
- Acceptance criteria re-run: file has keys obvious_bug, clean and fork; head_sha count printed 3; both automated verify commands exit 0.
