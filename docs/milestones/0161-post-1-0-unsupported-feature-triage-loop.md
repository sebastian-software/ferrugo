# 0161: Post-1.0 Unsupported Feature Triage Loop

Status: done
Phase: 30
Size: medium
Depends on: 0160

## Goal

Turn the PDFium-free 1.0 unsupported backlog into a repeatable triage loop that
prioritizes real-world document impact over theoretical specification coverage.

## Scope

- Categorize every remaining typed unsupported reason by document family,
  frequency, render impact, and implementation risk.
- Add a lightweight triage report format for newly observed unsupported cases.
- Identify the next native-renderer slices that improve typical-document
  coverage with bounded implementation size.
- Keep runtime PDFium out of triage and use historical PDFium evidence only as
  archived context.

## Non-Goals

- Implement the unsupported features discovered by the triage.
- Claim complete PDF specification support.
- Reopen PDFium runtime fallback as a mitigation.

## Deliverables

- Unsupported-feature triage report.
- Ranked post-1.0 implementation backlog.
- Updated support matrix with frequency and impact fields.

## Acceptance Criteria

- Unsupported reasons have stable categories and owners in the backlog.
- The top follow-up slices are small enough for milestone execution.
- No triage path requires PDFium at runtime.

## Validation

- Run native-only `cargo test`.
- Run supported corpus fallback summary.
- Run unsupported corpus classification.
- Review the support matrix for missing categories.

## Completion Notes

- Added `docs/reports/post-1-0-unsupported-feature-triage-2026-06-26.md`.
- Updated `docs/reports/native-renderer-support-matrix-2026-06-24.md` with
  frequency, impact, implementation risk, owner route, and representative
  fixtures for the 5 remaining typed unsupported buckets.
- Extended `docs/policies/native-conformance-triage.md` with a lightweight
  unsupported-feature report format for newly observed cases.
- Supported runtime families remain green at 87/87 native renders with 0
  fallbacks and 0 errors.
- Full corpus classification reports 8 typed unsupported rows:
  `image.filter` (3), `graphics.transparency` (2),
  `graphics.optional-content` (1), `graphics.pattern-shading` (1), and
  `form.xfa-dynamic` (1), plus 1 encrypted policy error.
- Native-only workspace tests and 0161 supported/unsupported fallback summaries
  passed.
