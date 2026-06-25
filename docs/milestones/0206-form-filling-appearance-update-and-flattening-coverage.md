# 0206: Form Filling Appearance Update And Flattening Coverage

Status: todo
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

Empty until done.
