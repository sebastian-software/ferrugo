# 0215: PDFium Comparison Tool Removal Decision Gate

Status: done
Phase: 40
Size: medium
Depends on: 0214

## Goal

Decide whether the remaining PDFium comparison tooling can be deleted, replaced,
or retained only in external historical workflows after Rust-native validation
has enough independent oracle coverage.

## Scope

- Inventory every remaining PDFium comparison hook, fixture, script, feature,
  documentation reference, and CI path.
- Compare current independent oracle coverage against retained PDFium comparison
  value.
- Remove or quarantine comparison code that no longer informs native rendering
  decisions.
- Produce a final decision for any PDFium references that remain.

## Non-Goals

- Delete historical reports or attribution records.
- Reintroduce PDFium into supported runtime packages.
- Remove comparison evidence before independent checks cover the same risk.

## Deliverables

- PDFium comparison inventory.
- Delete, replace, quarantine, or retain decision table.
- Patch set for approved removal or quarantine work.

## Acceptance Criteria

- No supported build or validation path requires PDFium.
- Remaining PDFium references are historical, external, or explicitly
  maintainer-only.
- Independent oracle coverage is sufficient for release decisions.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run repository scan for PDFium references.
- Run package dry-runs without PDFium assets.
- Run independent visual oracle validation.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-29.

- Inventoried remaining active PDFium surfaces across workspace membership,
  optional CLI feature wiring, maintainer commands, package/quarantine scripts,
  backend docs, corpus policy docs, and fixture manifests.
- Decision: retain `ferrugo-pdfium` and the PDFium-specific CLI commands only as
  explicit maintainer oracle tooling behind `--features pdfium`; do not delete
  them until native-only golden comparison and multi-oracle records cover the
  same debugging value.
- Replaced active fixture `expected:pdfium-fallback` tags with
  `expected:native-unsupported`.
- Corrected stale native backend guidance that still suggested explicit PDFium
  runtime retry for product code.
- Hardened `scripts/check_pdfium_quarantine.sh` to use `rg` consistently.
- Report: `docs/reports/pdfium-comparison-tool-removal-decision-2026-06-29.md`.

Validation:

- `rg -n "expected:pdfium-fallback|explicitly retry|PDFium remains the oracle and explicit fallback" fixtures docs --glob '!docs/reports/**' --glob '!docs/milestones/**'`
- `cargo fmt --check`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace --no-default-features`
- `bash scripts/check_pdfium_quarantine.sh`
- `cargo package -p ferrugo-cli --allow-dirty --no-verify --list`
- `cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/transparency-stack-memory-manifest.tsv --include-family alpha-stack --include-family group-stack --include-family soft-mask-stack --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/pdfium-removal-0215-poppler.json`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `git diff --check -- scripts/check_pdfium_quarantine.sh fixtures/scanner-ocr-workflow-manifest.tsv fixtures/transparency-conformance-manifest.tsv fixtures/mobile-scan-manifest.tsv fixtures/optional-content-ui-state-manifest.tsv fixtures/annotation-print-preview-manifest.tsv fixtures/government-form-manifest.tsv fixtures/real-world-style-manifest.tsv fixtures/map-rendering-manifest.tsv fixtures/corpus-manifest.tsv docs/backend/native.md docs/backend/pdfium.md docs/corpus-taxonomy.md docs/policies/corpus-intake.md docs/policies/reference-oracle-strategy.md docs/backlogs/reference-oracle-tooling-backlog.md docs/milestones/README.md docs/milestones/0215-pdfium-comparison-tool-removal-decision-gate.md docs/reports/pdfium-comparison-tool-removal-decision-2026-06-29.md`
