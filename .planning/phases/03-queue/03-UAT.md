---
status: complete
phase: 03-queue
source: [03-VERIFICATION.md]
started: 2026-11-12T10:58:02Z
updated: 2026-11-12T11:42:13.580Z
---

## Current Test

[testing complete]

## Tests

### 1. Live direct request
expected: With `diffmate watch` running, request your review directly on a real GitHub pull request. Do not run `diffmate review`. The PR is tracked, one automatic review finishes as a ready draft, and Notification Center shows `diffmate: draft ready` with the PR label. No other action is required.
result: pass

### 2. Live give-up notification
expected: Let an automatic review use up its retries, or fail terminally, while `watch` is running. Separately, send SIGTERM during an automatic run. The give-up shows `diffmate: review failed` with the PR label and the failure reason. SIGTERM requeues the job and does not notify.
result: pass

## Summary

total: 2
passed: 2
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
