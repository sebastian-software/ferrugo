# 0191: DeviceN Spot Color Visual Review Samples

Status: done
Phase: 36
Size: medium
Depends on: 0190

## Goal

Improve confidence for DeviceN and spot-color approximations by adding visual
review samples that reflect common business and print-adjacent documents.

## Scope

- Add fixtures with common Separation, DeviceN, and tint transform usage.
- Compare native approximation output against documented expectations.
- Add visual review notes for cases where exact colorimetry is outside scope.
- Track performance cost of tint transform evaluation.

## Non-Goals

- Provide certified prepress color proofing.
- Implement every proprietary spot-color library.
- Treat print-only color precision as a blocker for all viewer workflows.

## Deliverables

- Spot-color visual sample set.
- Approximation policy update.
- Performance notes for color conversion paths.

## Acceptance Criteria

- Typical spot-color documents render with understandable approximations.
- Out-of-scope color fidelity cases are documented.
- Tint transform evaluation remains bounded.

## Validation

- Run native-only `cargo test`.
- Run spot-color fixture visual comparisons.
- Run color conversion benchmark profiles.
- Review approximation policy text.

## Completion Notes

Completed on 2026-06-29.

- Added three generated visual-review samples:
  - `fixtures/generated/spot-letterhead-separation.pdf`
  - `fixtures/generated/spot-invoice-devicen-stamp.pdf`
  - `fixtures/generated/spot-cmyk-tint-swatch.pdf`
- Added `fixtures/spot-color-visual-review-manifest.tsv` for Separation,
  DeviceN, CMYK-alternate tint, overprint, and prepress spot-color review.
- Added native regression coverage that renders the new samples and verifies
  visible spot-color content at stable sample points.
- Updated approximation policy documentation and recorded evidence in
  `docs/reports/devicen-spot-color-visual-review-2026-06-29.md`.

The review suite renders 7/7 fixtures natively with 0 fallbacks and 0 errors.
Poppler visual review reports 7 accepted drift rows and 0 blockers under
spot-color thumbnail thresholds. This remains RGB thumbnail approximation, not
certified print proofing or separations output.
