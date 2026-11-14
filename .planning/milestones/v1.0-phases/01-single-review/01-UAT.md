---
status: complete
phase: 01-single-review
source: [01-VERIFICATION.md]
started: 2026-10-27T00:00:00Z
updated: 2026-10-26T23:09:01.280Z
---

## Current Test

[testing complete]

## Tests

### 1. Obvious-bug and fork PRs are anchored to calc.py:7 (criteria 1 and 6)
expected: `diffmate review` on pull/1 and pull/3 each exit 0 and print a block starting `calc.py:7 [`; pull/1's title is about the out-of-range index (`items[len(items)]`), pull/3's about the wrong index (`items[1]` in `first_item`)
result: pass

### 2. Clean PR has no findings (criterion 2)
expected: `diffmate review https://github.com/paderinandrey/diffmate-fixtures/pull/2` exits 0 and the first line ends `succeeded (no_findings)`; limitation and improvement lines are fine
result: pass

### 3. Second review waits for the first (criterion 8)
expected: terminal 1 `diffmate review BUG_URL`; while it runs, terminal 2 `diffmate review CLEAN_URL` prints a line starting `waiting:` and finishes after terminal 1; terminal 3 `diffmate status` returns at once and lists the first target running and the second queued
result: pass

### 4. Isolation evidence and residual risk (criterion 4)
expected: in 01-11-SUMMARY.md and the `mise run test:e2e` output, each REV-06, REV-07 and REV-08 control shows its canary or marker, each isolated run shows none, and isolation_flags_keep_authentication passes; the owner accepts that tool-level confinement (an agent Read outside the worktree or through a symlink) has no direct control (01-11 coverage D6)
result: pass

### 5. `mise run vuln` license policy
expected: the owner decides on adding `MIT-0` to deny.toml [licenses].allow (thiserror-impl, tokio-macros, borrow-or-share) or another remedy, and `mise run vuln` exits 0
result: pass

### 6. Owner edit in 01-11-SUMMARY.md
expected: the edit recording acceptance of the substitute controls (option a) is committed or discarded, and the working tree is clean apart from the untracked planning state files
result: pass

## Summary

total: 6
passed: 6
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
