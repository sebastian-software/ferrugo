# 0194: Forms Appearance State Mutation Boundary

Status: todo
Phase: 36
Size: medium
Depends on: 0193

## Goal

Define the renderer boundary for AcroForm appearance states, value changes, and
viewer-side form preview without becoming a PDF form editor.

## Scope

- Add fixtures for checkboxes, radio buttons, text fields, choice fields, and
  stale appearance streams.
- Distinguish rendering existing appearances from synthesizing changed states.
- Document which mutations consumers may request and which require external
  form editing.
- Keep synthesized appearances bounded and deterministic.

## Non-Goals

- Implement full form filling and saving.
- Execute JavaScript calculation or validation actions.
- Mutate source PDFs during rendering.

## Deliverables

- Form appearance state policy.
- Form preview fixture set.
- Typed unsupported reasons for mutation-only behavior.

## Acceptance Criteria

- Existing common widget appearances render consistently.
- Requested state changes have explicit support or rejection behavior.
- The renderer does not silently alter document bytes.

## Validation

- Run native-only `cargo test`.
- Run form appearance visual comparisons.
- Run API behavior tests for requested state changes.
- Review public documentation for mutation boundaries.

## Completion Notes

Empty until done.
