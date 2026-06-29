# 0207: Annotation Popup Stamp And FreeText Fidelity

Status: done
Phase: 39
Size: medium
Depends on: 0206

## Goal

Improve Rust-native fidelity for common review, markup, stamp, popup, and
FreeText annotations found in legal, government, and business workflows.

## Scope

- Add fixtures for stamps, FreeText boxes, popups, highlights, comments,
  callouts, and print-visible annotation states.
- Validate annotation appearance streams, default appearances, opacity,
  rotation, page boxes, and print-preview behavior.
- Track unsupported annotation behavior through typed diagnostics.
- Keep interactive state handling separate from static render fidelity.

## Non-Goals

- Build a complete annotation editor.
- Synchronize collaborative review comments.
- Render JavaScript-driven annotation behavior.

## Deliverables

- Annotation fidelity corpus.
- Print-preview and screen-rendering comparison report.
- Unsupported annotation taxonomy update.

## Acceptance Criteria

- Common markup annotations render in the expected screen and print states.
- Missing or malformed appearances are handled consistently.
- Unsupported annotation types are documented without silent visual loss.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run annotation visual comparisons.
- Run print-preview annotation checks.
- Run unsupported annotation snapshot tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added generated fixtures for FreeText with explicit normal appearance, a
  print-visible rotated stamp appearance, and an inert popup annotation state.
- Added `fixtures/annotation-popup-stamp-freetext-manifest.tsv` and documented
  it in `docs/corpus-taxonomy.md`.
- Added the new annotation fixtures to `fixtures/corpus-manifest.tsv` under
  `mixed-layout`.
- Updated `docs/policies/annotation-fallbacks.md` to clarify supported FreeText
  appearance streams and inert popup metadata, while keeping FreeText without a
  usable appearance typed as `annotation.appearance`.
- Added native regression coverage in
  `native_backend_should_render_generated_popup_stamp_freetext_annotation_fixtures`.
- Produced `docs/reports/annotation-popup-stamp-freetext-2026-06-29.md`.
- Supported 0207 families pass 10/10 native with zero fallbacks, errors, or
  budget failures. FreeText without appearance remains a typed unsupported
  boundary.
- Poppler visual review found 2 exact matches, 5 accepted drifts, 0 blockers, 0
  native errors, and 3 reference errors.

Validation run:

- `python3 scripts/generate_fixtures.py`
- `cargo fmt --check`
- `python3 -m py_compile scripts/generate_fixtures.py`
- `cargo test -p ferrugo-native popup_stamp_freetext -- --nocapture`
- `cargo test -p ferrugo-native annotation -- --nocapture`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family appearance-stream --include-family stamp-appearance --include-family print-state --include-family synthesized-markup --include-family popup-boundary --include-family nonvisual-link --fail-on-fallback --max-edge 160 --output target/annotation-0207-supported.json`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family unsupported-synthesis --max-edge 160 --output target/annotation-0207-unsupported.json`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family appearance-stream --include-family stamp-appearance --include-family print-state --include-family synthesized-markup --include-family popup-boundary --include-family nonvisual-link --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/annotation-0207-benchmark.json`
- `cargo run -p ferrugo-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family appearance-stream --include-family stamp-appearance --include-family print-state --include-family synthesized-markup --include-family popup-boundary --include-family nonvisual-link --include-family unsupported-synthesis --output target/annotation-0207-operators.json`
- `cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/annotation-popup-stamp-freetext-manifest.tsv --include-family appearance-stream --include-family stamp-appearance --include-family print-state --include-family synthesized-markup --include-family popup-boundary --include-family nonvisual-link --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/annotation-0207-poppler.json`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
