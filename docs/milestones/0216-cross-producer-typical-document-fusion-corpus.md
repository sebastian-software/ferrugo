# 0216: Cross-Producer Typical Document Fusion Corpus

Status: todo
Phase: 41
Size: medium
Depends on: 0215

## Goal

Build a fused corpus that combines office suites, browsers, scanners, mobile
apps, report generators, government systems, and design tools into a single
typical-document confidence gate.

## Scope

- Classify equivalent document workflows across multiple producers.
- Add reduced fixtures that isolate producer-specific differences for the same
  document family.
- Track renderer behavior by workflow, producer, feature category, and profile.
- Keep privacy review and fixture minimization requirements enforced.

## Non-Goals

- Store private user documents.
- Prefer producer popularity over measured user workflow impact.
- Remove focused feature corpora that still catch specific regressions.

## Deliverables

- Cross-producer typical-document corpus.
- Producer compatibility matrix update.
- Fixture minimization and privacy report.

## Acceptance Criteria

- Common workflows have coverage across multiple producer implementations.
- Producer-specific failures are linked to reduced fixtures and typed causes.
- The corpus remains runnable in native-only CI profiles.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run cross-producer corpus gate.
- Run fixture privacy and minimization checks.
- Run producer compatibility matrix generation.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
