---
phase: 02-reliability
plan: 02
subsystem: configuration
tags: [toml, serde, configuration, validation, proptest]
requires:
  - phase: 01-single-review
    provides: "CLI dispatch, AppConfig and isolated black-box test harness"
provides:
  - "Strict startup loading of config.toml before review enqueues work"
  - "SPEC defaults and validation for all four settings"
  - "Single-line errors with source path, line and key"
affects: [02-05, 02-08, 02-09, 03-queue]
actuals:
  tokens: 6336
  tasks: 3
  commits: 2
plan_head_before: 2655ae06822a7d7a11a741242cb199c74469568a
plan_head_after: ddcde20dc9b89df780939b1ddc9f7d9f8527e577
commits: 2
tech-stack:
  added: ["toml 1.1.8+spec-1.1.0 (std, parse, serde; default features disabled)"]
  patterns:
    - "Spanned integer deserialization followed by range validation"
    - "Load configuration before starting the async runtime"
key-files:
  created:
    - src/config.rs
    - src/config/raw.rs
    - src/config/error.rs
    - src/config/error/key.rs
    - src/config/tests.rs
    - src/config/tests/errors.rs
    - src/config/tests/files.rs
    - tests/config.rs
    - .planning/phases/02-reliability/02-02-RED.json
  modified:
    - Cargo.toml
    - Cargo.lock
    - mise.toml
    - hk.pkl
    - src/main.rs
    - src/app.rs
    - src/tests/live/support.rs
key-decisions:
  - "Owner approved the toml dependency on 2026-11-01"
  - "Unknown keys, invalid values and unreadable files stop review before creating work"
  - "Settings remain defaults for commands that do not load config.toml"
requirements-completed: [REL-13]
coverage:
  - id: D1
    description: "Missing and partial config files use SPEC defaults while each supplied setting is validated"
    requirement: REL-13
    verification:
      - kind: unit
        ref: "src/config/tests.rs#each_key_overrides_only_its_default"
        status: pass
      - kind: unit
        ref: "src/config/tests.rs#duration_ranges_match_their_units"
        status: pass
      - kind: unit
        ref: "src/config/tests.rs#retry_range_matches_u32"
        status: pass
      - kind: integration
        ref: "tests/config.rs#missing_config_uses_defaults"
        status: pass
    human_judgment: false
  - id: D2
    description: "Invalid configuration prints a precise single-line error before any database, job or subprocess call"
    requirement: REL-13
    verification:
      - kind: integration
        ref: "tests/config.rs#invalid_config_stops_review_before_enqueue"
        status: pass
      - kind: integration
        ref: "tests/config.rs#config_errors_are_one_line_and_strip_terminal_controls"
        status: pass
      - kind: unit
        ref: "src/config/tests/errors.rs#syntax_errors_keep_the_key_and_line_when_an_assignment_exists"
        status: pass
      - kind: unit
        ref: "src/config/tests/files.rs#unreadable_file_is_an_error_with_its_source"
        status: pass
    human_judgment: false
  - id: D3
    description: "Status ignores even a malformed configuration file"
    requirement: REL-13
    verification:
      - kind: integration
        ref: "tests/config.rs#status_ignores_broken_config"
        status: pass
    human_judgment: false
duration: not recorded
completed: 2026-11-01
status: complete
---

# Phase 2 Plan 02: Configuration loading and validation

**Review reads the four SPEC settings once at startup, refuses invalid configuration before enqueue, and prints source-aware single-line errors.**

## Accomplishments

- `Settings` supplies defaults of 15 minutes, 60 seconds, 60 seconds and 2 retries. Missing files and keys keep those defaults. The three durations must be positive integers; the timeout must fit when converted to seconds, and retry count must fit `u32`, including zero.
- `review` loads `$HOME/.config/diffmate/config.toml` before building the tokio runtime. `AppConfig` carries the path and settings. `status` does not load the file. The existing live-test helper uses defaults and an isolated path.
- Unknown keys and tables, wrong types, duplicate keys, malformed values and I/O errors fail explicitly. Error chains retain the underlying TOML or I/O cause. Source spans supply line numbers; error context and TOML-parsed assignment keys preserve names even for quoted keys. CLI rendering removes terminal controls and collapses newlines.
- Three proptests cover signed integer ranges and arbitrary input. Integration tests verify no database and no fake gh/claude calls on invalid configuration, successful default behavior, status independence and safe error rendering.

## Task Commits

1. **Task 1: Package checkpoint** — the owner replied `additive-backup, approved`; `approved` satisfies this plan's dependency gate.
2. **Task 2: Startup tracer and dependency update** — `b772750`, `feat(02-02): load validated configuration before review`.
3. **Task 3: Validation matrix and error fixes** — `ddcde20`, `fix(02-02): keep config errors precise and verify validation`.

This summary is committed separately. Shared STATE, ROADMAP and REQUIREMENTS updates belong to the orchestrator.

## Dependency Rationale

PR note: SPEC "Configuration" requires a TOML file. The `toml` crate parses that format through serde, which is already a dependency; no duration parser or additional direct crate is needed. Installed `toml` is **1.1.8+spec-1.1.0**, with only `std`, `parse` and `serde` enabled. The inspected installed source confirms `Spanned`, error spans/messages and these features. The owner approved installation after the orchestrator verified its official repository, MIT OR Apache-2.0 license and October 9 release date.

The required `mise run deps:check` found eligible updates, so `mise run deps:update` also moved hk 2.8.1 to 2.9.0, nextest 0.9.149 to 0.9.150, jsonschema 0.60.1 to 0.60.2, rusqlite 0.41.0 to 0.41.1 and rustix 1.1.5 to 1.1.6. Rust remains 1.99.1. `hk.pkl` matches the new hk version. The final check confirms all direct crates and tools satisfy the project's latest-release and three-day rules.

## Verification

Observed results:

- `mise run --force test -- -E 'binary(config) | binary(review) | binary(cli)'`: **8 passed**, after dependency updates.
- `mise run --force test -- --no-fail-fast -E 'test(config::) | binary(config)'`: **18 passed**, including all three proptests; 195 excluded by the filter.
- `mise run --force lint`: passed (rustfmt, clippy with warnings denied, cargo-machete, gitleaks and actionlint). The final task hook reran lint on the committed source.
- `mise run deps:check`: passed; Rust 1.99.1, tools and direct crates are at their latest eligible releases.
- `mise run vuln`: passed advisories, bans, licenses and sources. Existing duplicate-version warnings for `hashbrown` and `syn` are non-failing under the repository policy.
- Mandatory hooks passed on both task commits. The first ran **186 tests**, the second **202 tests**; both passed with **11 live tests skipped**. Hooks also checked docs, secrets, large files and actions. No bypass was used.
- `git diff --check` passed. Each changed source/test file is at most 100 lines. The required `deny_unknown_fields`, `config::load` and `proptest!` declarations are present.

Not run in this plan: release build, coverage or live e2e. Coverage and release acceptance belong to the phase gate. No real user configuration or live database was touched.

## RED / GREEN Evidence

The new validation tests exposed two defects in the tracer: duplicate/malformed assignments lost their key, and an escaped newline in an unknown key produced three CLI output lines. A fresh nextest JUnit run executed both intended tests and failed both assertions with successful compilation and fixtures. The unchanged report, console output, command, exit 100 and semantic assessment are preserved in [02-02-RED.json](./02-02-RED.json). `gsd-tools check tdd-red-evidence` returned **RED_EVIDENCE_OK** before the fixes.

Key inference now parses the relevant assignment's key with the same TOML parser; CLI rendering collapses newlines after sanitization. The final 18-test selection and full 202-test hook both pass. Tests and fixes share the green Task 3 commit because project hooks prohibit committing a failing suite.

## Deviations from Plan

1. **[Rule 3 — Blocking] Required dependency updates expanded tooling scope.** The planned freshness gate failed on five old pins. The standard update task fixed them; the resulting lint, tests, freshness and advisory checks passed. `mise` initially rewrote hk as an inline table, which the existing update script cannot parse. Restored the established plain version-string convention and reran the update successfully; no script behavior changed.
2. **[Rule 3 — Blocking] Split error handling and tests into small submodules.** Keeps the 100-line source convention. The existing live-test `AppConfig` literal gained the two new fields so ignored tests continue to compile.
3. **[Rule 1 — Bug] Retain keys in syntax errors and keep CLI output on one line.** Both gaps were discovered by the Task 3 RED run and fixed without weakening assertions. Additional quoted-key tests cover `=` and backticks in names.
4. **Project hook policy takes precedence over separate failing commits.** Machine-validated RED evidence is retained, while tests and fixes are committed together after GREEN.

## Issues Encountered

- Sandbox restrictions blocked mise cache writes, actionlint launch and advisory database access; scoped escalation completed the same commands.
- Initial `toml::Table` code assumed an unavailable `into_keys` method; used the inspected map's iterator instead. The failed compilation was not counted as RED evidence.
- Clippy requested an equality assertion for an empty collection, byte-slice splitting for line counting and a collapsed conditional in a proptest; all corrected.

## Performance

- Start time was not recorded; task closeout was at 2026-11-01T00:01:33Z.
- Tasks: 3 including the owner checkpoint. Production/evidence files changed: 16 before this summary.
- Actual token estimate: realized source/tooling diff bytes divided by four (25,344 / 4), not a runtime token count.

## Next Phase Readiness

Plans 02-05 and 02-08 can use `Settings::max_retries()` and `Settings::run_timeout()`; remove their production-only dead-code expectations when first consumed. Plan 02-09 can load the same config for `watch`. Poll and debounce settings are accepted and validated but remain unused until Phase 3. No additional owner action is needed for this plan.

## Self-Check: PASSED

Both task commits are on the phase branch, required files and evidence exist, observed automated checks passed and the shared GSD artifacts were preserved.
