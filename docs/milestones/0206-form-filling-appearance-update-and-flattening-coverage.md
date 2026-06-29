# 0206: Form Filling Appearance Update And Flattening Coverage

Status: done
Phase: 39
Size: medium
Depends on: 0205

## Goal

Cover common form workflows where field values, generated appearances, and
flattened output must remain visually stable without PDFium.

## Scope

- Add fixtures for filled text fields, checkboxes, radio buttons, combo boxes,
  signatures, and flattened form exports.
- Validate appearance stream reuse, regenerated appearances, default resources,
  field rotations, and missing appearance fallbacks.
- Document the boundary between visual rendering, form mutation, and signature
  validation.
- Add privacy-safe diagnostics for form appearance failures.

## Non-Goals

- Implement a full PDF form editor.
- Certify cryptographic signature validity beyond the documented boundary.
- Support dynamic XFA as a native renderer feature.

## Deliverables

- Form filling and flattening regression corpus.
- Appearance update and unsupported-boundary report.
- Diagnostics for missing or inconsistent form appearances.

## Acceptance Criteria

- Common filled and flattened forms render accurately in native-only mode.
- Missing appearance cases are typed and recoverable where safe.
- Form handling stays within documented mutation and validation boundaries.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run form appearance corpus comparisons.
- Run signature-boundary fixture checks.
- Run privacy-safe diagnostics snapshot tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added generated fixtures for a filled combo-box appearance, a rotated
  text-field appearance, and an already-flattened static form export.
- Added `fixtures/form-filling-flattening-manifest.tsv` and documented it in
  `docs/corpus-taxonomy.md`.
- Added the new fixtures to `fixtures/corpus-manifest.tsv` under the `form`
  family.
- Updated `docs/policies/acroform-appearances.md` to clarify that source
  generated combo/rotated appearances and already-flattened form exports are
  supported as static render inputs, while native form editing/flattening
  remains outside thumbnail rendering.
- Added native regression coverage in
  `native_backend_should_render_generated_form_filling_flattening_fixtures`.
- Produced `docs/reports/form-filling-flattening-2026-06-29.md`.
- Supported 0206 families pass 15/15 native with zero fallbacks, errors, or
  budget failures. Dynamic XFA without static AcroForm fields remains a typed
  `form.xfa-dynamic` unsupported boundary.

Validation run:

- `python3 scripts/generate_fixtures.py`
- `cargo fmt --check`
- `python3 -m py_compile scripts/generate_fixtures.py`
- `cargo test -p pdfrust-native native_backend_should_render_generated_form_filling_flattening_fixtures -- --nocapture`
- `cargo test -p pdfrust-native acroform -- --nocapture`
- `cargo test -p pdfrust-native form_filling -- --nocapture`
- `cargo test -p pdfrust-native signature_presence -- --nocapture`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family existing-appearance --include-family signature-boundary --include-family synthesized-static --include-family flattened-static --fail-on-fallback --max-edge 160 --output target/form-filling-0206-supported.json`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family xfa-boundary --max-edge 160 --output target/form-filling-0206-xfa-boundary.json`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family existing-appearance --include-family signature-boundary --include-family synthesized-static --include-family flattened-static --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/form-filling-0206-benchmark.json`
- `cargo run -p pdfrust-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family existing-appearance --include-family signature-boundary --include-family synthesized-static --include-family flattened-static --include-family xfa-boundary --output target/form-filling-0206-operators.json`
- `cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/form-filling-flattening-manifest.tsv --include-family existing-appearance --include-family signature-boundary --include-family flattened-static --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/form-filling-0206-poppler.json`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
