# Deferred items: phase 01-single-review

## Found during 01-10

- `mise run vuln` (`cargo deny check licenses`) fails: `borrow-or-share 0.2.4` (license MIT-0), pulled in by `fluent-uri` -> `referencing` -> `jsonschema 0.59.0`, is not allowed by deny.toml. Predates plan 01-10 (rustix and tokio's signal feature add no such crate). Decide whether to add MIT-0 to the allow list; advisories, bans and sources are ok.
