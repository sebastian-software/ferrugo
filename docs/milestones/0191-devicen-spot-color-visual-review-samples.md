# 0191: DeviceN Spot Color Visual Review Samples

Status: todo
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

Empty until done.
