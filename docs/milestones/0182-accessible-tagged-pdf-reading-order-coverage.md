# 0182: Accessible Tagged PDF Reading Order Coverage

Status: todo
Phase: 34
Size: medium
Depends on: 0181

## Goal

Use tagged PDF structure to validate visual integrity and expose enough reading
order signals for searchable, accessible, and reviewable typical documents.

## Scope

- Parse structure tree relationships needed for page content association.
- Detect reading-order mismatches that indicate hidden or misplaced visual text.
- Add fixtures for tagged reports, forms, invoices, and generated office PDFs.
- Document the boundary between visual rendering, text extraction, and
  accessibility metadata.

## Non-Goals

- Implement a full accessibility API.
- Repair incorrect producer tags.
- Change visual output solely to match logical order.

## Deliverables

- Tagged PDF coverage report.
- Structure tree parsing or classification improvements.
- Fixtures covering common tagged producer output.

## Acceptance Criteria

- Tagged PDFs do not regress visual rendering.
- Reading-order signals are classified without excessive allocations.
- Unsupported accessibility structures are documented and typed.

## Validation

- Run native-only `cargo test`.
- Run tagged PDF fixture rendering.
- Run text and metadata classification tests.
- Run visual comparison for tagged fixtures.

## Completion Notes

Empty until done.
