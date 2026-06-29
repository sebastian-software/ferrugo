# 0194: Forms Appearance State Mutation Boundary

Status: done
Phase: 36
Size: medium
Depends on: 0193

## Goal

Define the renderer boundary for AcroForm appearance states, value changes, and
viewer-side form preview without becoming a PDF form editor.

## Scope

- Add fixtures for checkboxes, radio buttons, text fields, choice fields, and
  stale appearance streams.
- Distinguish rendering existing appearances from synthesizing changed states.
- Document which mutations consumers may request and which require external
  form editing.
- Keep synthesized appearances bounded and deterministic.

## Non-Goals

- Implement full form filling and saving.
- Execute JavaScript calculation or validation actions.
- Mutate source PDFs during rendering.

## Deliverables

- Form appearance state policy.
- Form preview fixture set.
- Typed unsupported reasons for mutation-only behavior.

## Acceptance Criteria

- Existing common widget appearances render consistently.
- Requested state changes have explicit support or rejection behavior.
- The renderer does not silently alter document bytes.

## Validation

- Run native-only `cargo test`.
- Run form appearance visual comparisons.
- Run API behavior tests for requested state changes.
- Review public documentation for mutation boundaries.

## Completion Notes

- Commit adds `ThumbnailOptions::form_appearance_mode` with default
  `DocumentState` and explicit `RequestedMutation` rejection.
- Rust-native requested form mutation preview now returns `unsupported` with
  bucket `form.appearance-mutation` before rendering.
- Added stale AcroForm fixtures:
  - `fixtures/generated/acroform-text-field-stale-appearance.pdf`
  - `fixtures/generated/acroform-checkbox-stale-appearance-state.pdf`
- Added focused gate manifest
  `fixtures/form-appearance-mutation-manifest.tsv`.
- Existing `/AP /N` and `/AS` document state remains authoritative over stale
  `/V`; bounded missing-appearance synthesis stays read-only and non-persistent.
- Report:
  `docs/reports/form-appearance-mutation-boundary-2026-06-29.md`.
- Validation:
  - `cargo test -p ferrugo-native acroform -- --nocapture`
  - `cargo test -p ferrugo-native appearance -- --nocapture`
  - `cargo test -p ferrugo-native mutation -- --nocapture`
  - `cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated --manifest fixtures/form-appearance-mutation-manifest.tsv --include-family existing-appearance --include-family stale-appearance --include-family synthesized-static --fail-on-fallback --output target/form-appearance-0194-supported-gate.json`
  - `cargo run -p ferrugo-cli -- visual-diff-poppler fixtures/generated --manifest fixtures/form-appearance-mutation-manifest.tsv --include-family existing-appearance --include-family stale-appearance --max-mae 8 --max-p95 32 --max-changed-ratio 0.15 --output target/form-appearance-0194-document-state-poppler-diff.json`
