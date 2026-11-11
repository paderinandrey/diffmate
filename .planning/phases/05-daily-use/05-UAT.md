---
status: testing
phase: 05-daily-use
source: [05-VERIFICATION.md]
started: 2026-11-11T17:24:47Z
updated: 2026-11-11T17:24:47Z
---

## Current Test

number: 1
name: Reboot, log in to the GUI session, then run diffmate doctor.
expected: |
  Stdout contains the line `runner lock: ok — held by watch` (em dash, U+2014) and the process exits 0.
awaiting: user response

## Tests

### 1. Reboot, log in to the GUI session, then run diffmate doctor.
expected: Stdout contains the line `runner lock: ok — held by watch` (em dash, U+2014) and the process exits 0.
result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
