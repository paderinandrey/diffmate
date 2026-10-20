# Synthesis Summary

Mode: new. Precedence: ADR > SPEC > PRD > DOC (per-doc overrides: SPEC.md = 0, prompts/review.md = 1).

## Doc counts by type
- SPEC: 1 (/Users/apaderin/dev/diffmate/SPEC.md)
- DOC: 1 (/Users/apaderin/dev/diffmate/prompts/review.md)
- ADR: 0
- PRD: 0
- UNKNOWN: 0
- Additional non-classified artifact referenced by SPEC: /Users/apaderin/dev/diffmate/schemas/review.schema.json (captured as CON-output-schema)

## Decisions locked: 1
- DEC-stack (Rust, Ratatui, tokio, rusqlite, `jsonschema`, `gh` CLI). Source: SPEC.md "Stack". Locked per orchestrator instruction; no ADRs exist.
- File: /Users/apaderin/dev/diffmate/.planning/intel/decisions.md

## Requirements: 5 (one per delivery slice, in intended phase order)
1. REQ-single-review
2. REQ-reliability
3. REQ-queue
4. REQ-review-and-publish
5. REQ-daily-use

Acceptance criteria are the verbatim "Done when" items from SPEC.md. File: /Users/apaderin/dev/diffmate/.planning/intel/requirements.md

## Constraints: 26
- nfr: 9 (CON-prop-1 to CON-prop-9, SPEC "Required properties" 1-9, hard invariants)
- protocol: 12 (run, target, draft freshness, publication, job state machines; concurrency; recovery; process model; git isolation; agent contract; publication procedure; queue rules)
- schema: 2 (storage layout, review output JSON schema)
- api-contract: 3 (agent input files, CLI commands, configuration)

File: /Users/apaderin/dev/diffmate/.planning/intel/constraints.md

## Context topics: 4
- Goals, Non-goals (out-of-scope boundaries), Domain concepts glossary, Review agent system prompt (prompts/review.md, verbatim, not a requirements source)

File: /Users/apaderin/dev/diffmate/.planning/intel/context.md

## Conflicts: 0 blockers, 0 competing variants, 3 info (no auto-resolutions were needed)
Detail: /Users/apaderin/dev/diffmate/.planning/INGEST-CONFLICTS.md

## Notes for the roadmapper
- Slice 4 (Review and publish) implements publication rules that are specified in full now (CON-publication-states, CON-job-states, CON-publication-procedure); earlier slices must keep the schema and job model compatible with them.
- Slice 1 begins with a spike-like first task (isolation checks, CLI result envelope fields, full-schema support via `--json-schema`); see REQ-single-review description.
- Transitions table (job x publication) lives only in SPEC.md lines 117-143 and is referenced, not copied.
