# 0075: Color Management And Output Intent Policy

Status: done
Phase: 12
Size: medium
Depends on: 0074

## Goal

Define a practical color-management policy for typical documents without
overfitting to prepress requirements.

## Scope

- Evaluate ICCBased, OutputIntent, Lab, and calibrated color usage in corpus
  documents.
- Decide where approximate conversion is acceptable for thumbnails.
- Add safe ICC handling only if a Rust dependency and memory profile are
  acceptable.
- Preserve typed unsupported results for color workflows beyond the policy.

## Non-Goals

- Full print-proof color accuracy.
- Native C color-management integrations without a decision record.
- Silent conversion of unsupported color spaces.

## Deliverables

- Color management decision record.
- Renderer support for accepted common color cases.
- Tests for approximate and unsupported color behavior.

## Acceptance Criteria

- Common office, browser, and scan color spaces render predictably.
- ICC and OutputIntent behavior is documented for callers.
- Color conversion stays within image and page memory budgets.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run color-focused corpus comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed with the `test: add output intent color coverage` implementation and
the `docs: complete color management policy` report update.

- Added `fixtures/generated/output-intent-rgb.pdf`, a DeviceRGB page with
  catalog OutputIntent metadata and a profile stream.
- Added native-backend coverage proving supported DeviceRGB content still
  renders natively when OutputIntent metadata is present.
- Accepted `docs/decisions/0005-color-management-and-output-intent-policy.md`:
  OutputIntent is metadata-only for thumbnails; ICCBased, Lab, Separation,
  DeviceN, spot color, and proofing workflows remain explicit unsupported
  cases.
- Recorded corpus summary and exact native/PDFium comparison results in
  `docs/reports/color-management-coverage-2026-06-24.md`.
