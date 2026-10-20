# Decisions

No ADR documents were found in the ingest set. The single entry below is a fixed decision declared by the SPEC and designated fixed by the orchestrator. It is not an ADR-classified lock (classification `locked: false`).

## DEC-stack: Implementation stack
- source: /Users/apaderin/dev/diffmate/SPEC.md (section "Stack")
- status: locked (fixed per orchestrator instruction; SPEC classification locked: false)
- decision: Rust. Ratatui for the TUI, tokio for the queue and subprocesses, rusqlite for state, `jsonschema` for output validation. GitHub access goes through the `gh` CLI.
- scope: implementation language, TUI, async runtime and subprocesses, state storage, output validation, GitHub access
