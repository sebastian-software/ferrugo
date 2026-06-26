# 0148: Government Form And Certificate Coverage

Status: done
Phase: 27
Size: medium
Depends on: 0147

## Goal

Cover common government, tax, permit, certificate, and public-agency PDFs that
combine forms, stamps, barcodes, signatures, and strict page geometry.

## Scope

- Add public or synthetic fixtures for static forms and certificate-style pages.
- Cover checkboxes, stamps, barcodes, signature appearances, and page labels.
- Verify annotation appearance and fallback behavior for common widgets.
- Track unsupported dynamic-form behavior explicitly.

## Non-Goals

- Validate legal authenticity or signatures cryptographically.
- Execute XFA or JavaScript.
- Store private identity documents.

## Deliverables

- Government-form corpus entries.
- Widget and appearance coverage report.
- Unsupported dynamic-form policy updates if needed.

## Acceptance Criteria

- Static government-style forms render without PDFium fallback.
- Missing dynamic behavior is reported with typed reasons.
- Visual artifacts preserve page geometry and form readability.

## Validation

- Run form-family visual comparison.
- Run annotation appearance tests.
- Run native-only supported corpus gate.
- Run privacy/provenance review for new fixtures.

## Completion Notes

Completed on 2026-06-26.

- Added three synthetic government-style fixtures:
  `government-permit-checkbox-form.pdf`,
  `government-certificate-seal-signature.pdf`, and
  `government-tax-notice-barcode.pdf`.
- Added `fixtures/government-form-manifest.tsv` to separate supported static
  form/certificate coverage from the dynamic-XFA unsupported boundary.
- Native supported gate is green at 8/8 rendered, 0 fallbacks, and 0 errors.
- Dynamic XFA remains explicitly typed as `form.xfa-dynamic`.
- Visual oracle reports 6 fidelity blockers and 0 native/PDFium errors; those
  deltas are routed to form appearance, line/table geometry, and stamp/barcode
  composition follow-ups.
- Report:
  `docs/reports/government-form-certificate-coverage-2026-06-26.md`.
