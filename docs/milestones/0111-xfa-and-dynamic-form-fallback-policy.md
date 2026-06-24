# 0111: XFA And Dynamic Form Fallback Policy

Status: todo
Phase: 20
Size: small
Depends on: 0110

## Goal

Define and enforce native behavior for XFA and dynamic-form PDFs without
pretending to support interactive form runtimes.

## Scope

- Detect XFA and dynamic form packets early.
- Render static fallback appearances when they are present.
- Return typed unsupported reasons when rendering would require an XFA runtime.
- Add fixtures for static XFA, dynamic XFA, and AcroForm hybrids.

## Non-Goals

- Implement an XFA JavaScript or layout runtime.
- Execute embedded scripts.
- Modify or submit forms.

## Deliverables

- XFA detection and fallback policy.
- Form fixture classification report.
- Error taxonomy update.

## Acceptance Criteria

- Static appearances render when available.
- Dynamic XFA documents fail predictably with a typed reason.
- No embedded script execution is introduced.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run form fixture classification.
- Run security-oriented form smoke tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
