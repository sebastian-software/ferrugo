# 0215: PDFium Comparison Tool Removal Decision Gate

Status: todo
Phase: 40
Size: medium
Depends on: 0214

## Goal

Decide whether the remaining PDFium comparison tooling can be deleted, replaced,
or retained only in external historical workflows after Rust-native validation
has enough independent oracle coverage.

## Scope

- Inventory every remaining PDFium comparison hook, fixture, script, feature,
  documentation reference, and CI path.
- Compare current independent oracle coverage against retained PDFium comparison
  value.
- Remove or quarantine comparison code that no longer informs native rendering
  decisions.
- Produce a final decision for any PDFium references that remain.

## Non-Goals

- Delete historical reports or attribution records.
- Reintroduce PDFium into supported runtime packages.
- Remove comparison evidence before independent checks cover the same risk.

## Deliverables

- PDFium comparison inventory.
- Delete, replace, quarantine, or retain decision table.
- Patch set for approved removal or quarantine work.

## Acceptance Criteria

- No supported build or validation path requires PDFium.
- Remaining PDFium references are historical, external, or explicitly
  maintainer-only.
- Independent oracle coverage is sufficient for release decisions.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run repository scan for PDFium references.
- Run package dry-runs without PDFium assets.
- Run independent visual oracle validation.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
