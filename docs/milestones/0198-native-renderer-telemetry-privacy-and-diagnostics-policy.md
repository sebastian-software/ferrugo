# 0198: Native Renderer Telemetry Privacy And Diagnostics Policy

Status: todo
Phase: 37
Size: small
Depends on: 0197

## Goal

Define privacy-safe renderer diagnostics so consumers can report failures,
unsupported features, and performance issues without leaking document content.

## Scope

- Classify diagnostic fields as safe, sensitive, local-only, or experimental.
- Add redaction rules for fixture paths, metadata, text snippets, and object
  identifiers.
- Document how consumers should attach debug bundles to issue reports.
- Ensure telemetry remains optional and application-controlled.

## Non-Goals

- Add hosted telemetry collection.
- Log document text or image content by default.
- Make diagnostics required for rendering.

## Deliverables

- Telemetry and diagnostics privacy policy.
- Redaction checklist for debug artifacts.
- Tests or review checks for generated diagnostic bundles.

## Acceptance Criteria

- Diagnostics avoid document content unless explicitly local-only.
- Unsupported and budget outcomes remain reportable.
- Public docs describe safe issue-reporting data.

## Validation

- Review generated debug artifacts.
- Run diagnostic bundle tests if available.
- Run native-only `cargo test`.
- Audit docs for privacy-sensitive examples.

## Completion Notes

Empty until done.
