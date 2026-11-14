---
phase: 05-daily-use
plan: 03
subsystem: infra
tags: [launchd, install, watch, macos]

requires:
  - phase: 05-daily-use
    provides: "diffmate doctor prints runner lock: ok — held by watch when watch holds the lock"
  - phase: 02-reliability
    provides: "watch waits for the runner lock and exits 0 on SIGTERM"
provides:
  - "docs/install.md is the only install document for the GUI launchd agent"
  - "The example plist uses string umask 0077, an absolute binary plus watch, and PATH with no token"
  - "bootstrap, kickstart -k, and bootout stay separate, and the two logs are emptied without a rotator"
affects: [daily-use, dotfiles]
actuals:
  tokens: 1101
  tasks: 2
  commits: 2
plan_head_before: b075a9c2ef0dc37ac40f3b1c1ff731500c85093a
plan_head_after: 73f0c475a440dd981b9a4ba2fa133d9c95ec9292
tech-stack:
  added: []
  patterns:
    - "The live plist stays in dotfiles. This repo documents it and does not commit it."
    - "bootstrap, kickstart -k, and bootout are three commands, not one reload."
key-files:
  created:
    - docs/install.md
  modified: []
key-decisions:
  - "The example plist keeps /Users/YOU. Prose tells the owner to replace it with echo $HOME and states this machine's PATH with /Users/apaderin."
  - "Umask is the string 0077. The integer form is named only as the measured umask 0115, not as a plist element."
patterns-established:
  - "docs/install.md is the only install page. There is no README and no committed plist."
  - "A new binary is cp plus kickstart -k. A second bootstrap is not the update step."
requirements-completed: [OPS-01, OPS-02, OPS-04]
coverage:
  - id: D1
    description: "docs/install.md documents the GUI launchd agent, the string umask 0077, the machine PATH, and bootstrap, and the repo adds neither a plist nor a README."
    requirement: OPS-01
    verification:
      - kind: other
        ref: "python3 docs/install.md task 1 string check"
        status: pass
      - kind: other
        ref: "mise run docs:check"
        status: pass
    human_judgment: true
    rationale: "The checks prove the text. The owner still copies the example into dotfiles. This phase does not bootstrap a job."
  - id: D2
    description: "Update is copy plus kickstart -k, remove is bootout, and emptying the logs is bootout, then truncate or delete, then bootstrap, with no rotator."
    requirement: OPS-02
    verification:
      - kind: other
        ref: "python3 docs/install.md task 2 string check"
        status: pass
    human_judgment: true
    rationale: "The three launchctl commands are separate in the page. No live job was loaded, updated, or removed."
  - id: D3
    description: "After login the owner looks for runner lock: ok — held by watch and exit 0. The page relies on the existing watch wait and SIGTERM exit 0."
    requirement: OPS-04
    verification:
      - kind: integration
        ref: "tests/watch/wait.rs#watch_waits_for_the_lock_and_stops_on_sigterm"
        status: pass
      - kind: integration
        ref: "tests/watch/shutdown.rs#sigterm_during_an_automatic_run_requeues_it"
        status: pass
    human_judgment: true
    rationale: "Surviving a reboot is a manual check after login. This phase does not reboot and does not change watch."
duration: 8min
completed: 2026-11-11
status: complete
---

# Phase 5 Plan 03: Launchd install page Summary

**`docs/install.md` tells a fresh Mac how to build watch, install a GUI launchd agent with string umask 0077, update it with kickstart, remove it with bootout, and empty the two logs**

## Performance

- **Duration:** 8 min
- **Started:** 2026-11-11T16:28:09Z
- **Completed:** 2026-11-11T16:36:00Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- `docs/install.md` is the only install document. The repo does not add a README or a plist.
- The example agent is label `dev.diffmate.watch`. `ProgramArguments` is the absolute `~/.local/bin/diffmate` path and then `watch`. `KeepAlive` and `RunAtLoad` are true, `ThrottleInterval` is the integer 10, and `Umask` is the string `0077`.
- The log directory is `mkdir -p -m 0700 ~/Library/Logs/diffmate` before bootstrap, and `chmod 0700` when that directory already exists.
- Install, update, and remove stay separate: `launchctl bootstrap gui/$UID`, `launchctl kickstart -k gui/$UID/dev.diffmate.watch`, and `launchctl bootout gui/$UID/dev.diffmate.watch`.
- The reboot sentence in the page is: After login, `diffmate doctor` prints `runner lock: ok — held by watch` and exits 0.

## Task Commits

Each task was committed atomically:

1. **Task 1: Install page a fresh machine can follow** - `3ce094a` (docs)
2. **Task 2: Update, remove, and empty the logs** - `73f0c47` (docs)

## Files Created/Modified

- `docs/install.md` - Launchd install, update, remove, and log cleanup. The plist is an example in a fenced block, not a repo file.

## Decisions Made

None - followed plan as specified. The example keeps `/Users/YOU` inside the plist. The prose says to replace it with the output of `echo $HOME` and writes this machine's PATH as `/opt/homebrew/bin:/Users/apaderin/.local/bin:/usr/bin:/bin:/usr/sbin:/sbin`.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration file was generated. The owner follows `docs/install.md` when installing the agent. This phase did not bootstrap a job and did not reboot.

## Next Phase Readiness

- The install page is ready for the owner to copy into dotfiles. The manual reboot check is still the owner's, after login.
- `watch` was not edited. The two SIGTERM tests passed: `watch_waits_for_the_lock_and_stops_on_sigterm` and `sigterm_during_an_automatic_run_requeues_it`.
- No plist and no README were added. `git ls-files` shows `docs/install.md` and no `*.plist` or `README`.

## Reboot sentence

As it appears in `docs/install.md`:

> After login, `diffmate doctor` prints `runner lock: ok — held by watch` and exits 0.

The em dash is U+2014. No plist and no README were added.

## Self-Check: PASSED

- FOUND: docs/install.md
- FOUND: .planning/phases/05-daily-use/05-03-SUMMARY.md
- FOUND: 3ce094a
- FOUND: 73f0c47

---
*Phase: 05-daily-use*
*Completed: 2026-11-11*
