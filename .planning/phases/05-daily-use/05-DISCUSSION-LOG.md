# Phase 5: Daily use - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-11-11
**Phase:** 5-Daily use
**Areas discussed:** Doctor report, Install text, Watch log file, Binary path

---

## Doctor report

### How doctor shows a failure

| Option | Description | Selected |
|--------|-------------|----------|
| One line and exit 1 | One line per check. A failure names what to do. Exit 0 when every check passes, otherwise 1. | ✓ |
| A paragraph and exit 1 | A short paragraph per failure. Success stays one line. Same exit rule. | |
| You decide | The planner picks the line shape and the exit code as long as each failure names the cause. | |

**User's choice:** One line and exit 1
**Notes:** Recorded as D-01.

### Held runner lock

| Option | Description | Selected |
|--------|-------------|----------|
| Held is ok | The line names the holder. A missing or unreadable lock file fails. Doctor does not take the lock. | ✓ |
| Held is a failure | Any holder exits 1. Doctor would fail whenever launchd is running watch. | |
| You decide | The planner decides whether a holder is a failure. | |

**User's choice:** Held is ok
**Notes:** Recorded as D-02.

### SQLite schema

| Option | Description | Selected |
|--------|-------------|----------|
| Report only | Doctor does not write. Version 5 is ok. Older fails with the existing review-once sentence. Newer fails because the binary is older. No database file is ok and is not created. | ✓ |
| Doctor migrates | Doctor opens the database for write and migrates it to 5, creating it on a fresh machine. | |
| You decide | The planner decides whether doctor writes. | |

**User's choice:** Report only
**Notes:** Recorded as D-03. `user_version` stays 5.

### Missing directories on a fresh machine

| Option | Description | Selected |
|--------|-------------|----------|
| Missing directory is ok | A missing state dir, cache dir, or config.toml is ok. Doctor creates nothing. An existing directory that is not writable or not mode 0700, or a broken config.toml, fails. | ✓ |
| Missing directory is a failure | Doctor exits 1 until the directories exist. It still does not create them. | |
| You decide | The planner decides whether a missing directory is a failure. | |

**User's choice:** Missing directory is ok
**Notes:** Recorded as D-04.

---

## Install text

### Where the instructions live

| Option | Description | Selected |
|--------|-------------|----------|
| docs/install.md | One install file. This phase does not add a README. | ✓ |
| README.md only | The instructions are the repository landing page. | |
| README and docs/install.md | A short README that links to the full page. | |
| You decide | The planner picks the file. | |

**User's choice:** docs/install.md
**Notes:** Recorded as D-05. The live plist stays in dotfiles.

### How the doc shows the agent

| Option | Description | Selected |
|--------|-------------|----------|
| Example in the text | A full plist in a code block, with path placeholders. No .plist file in the repo. | ✓ |
| Keys only | The keys and the launchctl commands, without a whole plist. | |
| A docs/*.plist file | A plist in the repo that the owner copies into dotfiles. | |
| You decide | The planner picks the form. | |

**User's choice:** Example in the text
**Notes:** Recorded as D-06.

### If watch exits while logged in

| Option | Description | Selected |
|--------|-------------|----------|
| Always restart | KeepAlive true, RunAtLoad true, ThrottleInterval 10. Stop with launchctl bootout. | ✓ |
| Only after a crash | KeepAlive only when the exit is not 0. A clean SIGTERM stays down until the next login. | |
| You decide | The planner picks the KeepAlive keys. | |

**User's choice:** Always restart
**Notes:** Recorded as D-07. The agent is a user agent in ~/Library/LaunchAgents.

### Reboot check

| Option | Description | Selected |
|--------|-------------|----------|
| Through doctor | After login, doctor prints `runner lock: ok — held by watch` and exits 0. | ✓ |
| Through launchctl print | The doc asks for launchctl print and state = running. | |
| You decide | The planner picks the check command. | |

**User's choice:** Through doctor
**Notes:** Recorded as D-08. The reboot itself is not automated.

---

## Watch log file

### Where stderr goes

| Option | Description | Selected |
|--------|-------------|----------|
| ~/Library/Logs/diffmate/watch.log | StandardErrorPath. launchd creates the file. diffmate does not open a log. | ✓ |
| Next to the database | ~/Library/Application Support/diffmate/watch.log | |
| You decide | The planner picks a path under the home directory that survives a reboot. | |

**User's choice:** ~/Library/Logs/diffmate/watch.log
**Notes:** Recorded as D-09. Phase 2 D-03 already sends one timestamped stderr line per event.

### Where stdout goes

| Option | Description | Selected |
|--------|-------------|----------|
| Two files | stdout to ~/Library/Logs/diffmate/watch.out.log. stderr stays in watch.log. | ✓ |
| stderr only | The example sets StandardErrorPath only. | |
| You decide | The planner decides whether stdout has its own file. | |

**User's choice:** Two files
**Notes:** Recorded as D-09.

### Who can read the log

| Option | Description | Selected |
|--------|-------------|----------|
| Owner only | Umask 0077. mkdir -p -m 0700 ~/Library/Logs/diffmate before bootstrap. | ✓ |
| Normal umask | No Umask key and no 0700 on the directory. | |
| You decide | The planner picks the mode. | |

**User's choice:** Owner only
**Notes:** Recorded as D-10.

### When the log grows

| Option | Description | Selected |
|--------|-------------|----------|
| Grow, no rotation | No rotator and no newsyslog example. The doc says to bootout, truncate or delete, then bootstrap. | ✓ |
| A newsyslog example | docs/install.md also shows a newsyslog snippet. The live config stays in dotfiles. | |
| You decide | The planner decides whether to describe rotation. | |

**User's choice:** Grow, no rotation
**Notes:** Recorded as D-11.

---

## Binary path

### Which file launchd runs

| Option | Description | Selected |
|--------|-------------|----------|
| ~/.local/bin/diffmate | mise run build, then cp target/release/diffmate to that path. ProgramArguments is the absolute path. | ✓ |
| target/release in the checkout | ProgramArguments points at the cargo build. cargo clean breaks the agent. | |
| You decide | The planner picks an absolute path outside target/. | |

**User's choice:** ~/.local/bin/diffmate
**Notes:** Recorded as D-12.

### PATH under launchd

| Option | Description | Selected |
|--------|-------------|----------|
| Narrow PATH in the plist | Directories from which git, which gh, and which claude, then the system directories. On this machine, ~/.local/bin and /opt/homebrew/bin. No full shell PATH. No tokens. | ✓ |
| The full shell PATH | Copy the interactive PATH, including mise and cargo. | |
| Do not set PATH | Leave launchd's default PATH. git, gh, and claude would have to live in /usr/bin or /bin. | |
| You decide | The planner picks a PATH that finds git, gh, and claude and contains no tokens. | |

**User's choice:** Narrow PATH in the plist
**Notes:** Recorded as D-13.

### Label

| Option | Description | Selected |
|--------|-------------|----------|
| dev.diffmate.watch | Same label in the repo example and in dotfiles. File dev.diffmate.watch.plist. No user name in the example. | ✓ |
| com.apaderin.diffmate | A personal reverse-DNS label in the public install doc. | |
| You decide | The planner picks one label and uses it in the doc and the launchctl commands. | |

**User's choice:** dev.diffmate.watch
**Notes:** Recorded as D-14.

### Picking up a new binary

| Option | Description | Selected |
|--------|-------------|----------|
| kickstart -k | Update with launchctl kickstart -k. Install is bootstrap. Remove is bootout. | ✓ |
| Always bootout | bootout then bootstrap even for a binary update. | |
| You decide | The planner picks the update command. | |

**User's choice:** kickstart -k
**Notes:** Recorded as D-15. kickstart sends SIGTERM, which watch already records as shutdown.

---

## Claude's Discretion

No question was answered with "You decide". Wording that was not asked is listed in `05-CONTEXT.md` under Claude's Discretion.

## Deferred Ideas

None. The offered `newsyslog` snippet was declined. The owner chose growth without rotation.
