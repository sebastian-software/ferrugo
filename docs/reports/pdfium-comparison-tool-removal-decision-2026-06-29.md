# PDFium Comparison Tool Removal Decision

Date: 2026-06-29
Milestone: 0215

## Summary

The remaining PDFium comparison tooling should be retained for now, but only as
explicit maintainer oracle tooling. It should not be deleted yet because current
release evidence still relies on native-only supported-family, budget, package,
Poppler, and manual-review gates rather than a complete committed golden-image
replacement for all disputed visual behavior.

Runtime PDFium fallback remains removed. Supported rendering, release packages,
and native validation must work without PDFium libraries, PDFium environment
variables, or hidden package assets.

## Decision

Retain the optional PDFium backend and PDFium CLI comparison commands as
maintainer-only tooling behind `--features pdfium`.

Do not use PDFium as:

- a runtime fallback for `render`, `render-auto`, or native library consumers;
- a default dependency of `pdfrust-cli`;
- a release prerequisite for the supported native rendering slice;
- a committed runtime asset or package artifact.

Delete or replace the maintainer-only comparison path only after the reference
oracle backlog has native-only golden comparison coverage and multi-oracle
records for disputed behavior.

## Inventory And Decisions

| Surface | Current role | Decision | Reason |
| --- | --- | --- | --- |
| `crates/pdfrust-pdfium` | Optional local oracle backend. | Retain, maintainer-only. | Still useful for metadata and pixel triage; not in default CLI dependency graph. |
| `pdfrust-cli` `pdfium` feature | Enables optional PDFium dependency. | Retain, opt-in only. | Keeps comparison code available without affecting native-only builds. |
| `render-pdfium` | Direct oracle render. | Retain, maintainer-only. | Useful for disputed raster output and debugging. |
| `render-isolated` / private `render-worker` | Process-isolated oracle probe. | Retain, maintainer-only. | Needed when the external oracle may hang or fault; direct worker invocation remains guarded. |
| `compare-metadata` | Page-count and page-size oracle. | Retain, maintainer-only. | Metadata parity still benefits from a second engine during expansion. |
| `benchmark-pdfium` | Reference performance backend. | Retain, maintainer-only. | Useful for local comparative measurements, not release gates. |
| `visual-diff` | PDFium pixel oracle. | Retain, maintainer-only. | Still useful for subsystem triage until golden and multi-oracle replacement lands. |
| `visual-diff-poppler` | PDFium-free pixel oracle. | Retain and prefer for release-oriented evidence. | Provides independent non-PDFium visual validation. |
| `scripts/check_pdfium_quarantine.sh` | Guard against runtime reintroduction. | Retain and harden. | Enforces native-only dependency and runtime crate boundaries. |
| `scripts/check_plugin_free_distribution.sh` | Guard against hidden runtime/plugin edges. | Retain. | Confirms default CLI tree remains plugin/network/PDFium-free. |
| `scripts/check_native_only_release.sh` | Native package/release gate. | Retain. | Runs native-only check/test, package file inspection, quarantine, and clippy. |
| `docs/build/*`, `docs/measurements/*` | Historical and local maintainer setup docs. | Retain as historical/external setup. | They document reproducible oracle setup without vendoring PDFium. |
| Fixture `expected:pdfium-fallback` tags | Legacy runtime-fallback expectation wording. | Replace with `expected:native-unsupported`. | Active corpus metadata should describe typed native behavior, not runtime retry. |

## Independent Oracle Coverage

Current release-oriented evidence can proceed without PDFium:

- native-only `cargo check` and `cargo test`;
- native-only supported-family fallback summaries;
- native budget and package gates;
- Poppler-backed `visual-diff-poppler` for independent visual checks;
- manual review records for threshold or semantics decisions;
- quarantine/package scans proving default artifacts do not contain PDFium
  runtime assets.

This is sufficient to keep PDFium out of supported runtime and release paths. It
is not yet sufficient to delete the maintainer comparison tooling entirely.
Deletion should wait for:

- a native-only `compare-golden` style command;
- a reviewed golden artifact retention policy;
- a small CI golden set for common document families;
- multi-oracle records for high-impact disputed rendering behavior.

## Patch Scope

This milestone:

- replaced active corpus `expected:pdfium-fallback` tags with
  `expected:native-unsupported`;
- updated corpus policy docs to use typed native unsupported expectations;
- corrected stale native backend docs that still suggested explicit PDFium
  runtime retry;
- switched the PDFium quarantine dependency-tree check from `grep` to `rg`;
- recorded this retain/quarantine/delete decision.

## Validation

Commands run:

```sh
rg -n "expected:pdfium-fallback|explicitly retry|PDFium remains the oracle and explicit fallback" fixtures docs --glob '!docs/reports/**' --glob '!docs/milestones/**'
cargo fmt --check
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
bash scripts/check_pdfium_quarantine.sh
cargo package -p pdfrust-cli --allow-dirty --no-verify --list
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/transparency-stack-memory-manifest.tsv --include-family alpha-stack --include-family group-stack --include-family soft-mask-stack --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/pdfium-removal-0215-poppler.json
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check -- scripts/check_pdfium_quarantine.sh fixtures/scanner-ocr-workflow-manifest.tsv fixtures/transparency-conformance-manifest.tsv fixtures/mobile-scan-manifest.tsv fixtures/optional-content-ui-state-manifest.tsv fixtures/annotation-print-preview-manifest.tsv fixtures/government-form-manifest.tsv fixtures/real-world-style-manifest.tsv fixtures/map-rendering-manifest.tsv fixtures/corpus-manifest.tsv docs/backend/native.md docs/backend/pdfium.md docs/corpus-taxonomy.md docs/policies/corpus-intake.md docs/policies/reference-oracle-strategy.md docs/backlogs/reference-oracle-tooling-backlog.md docs/milestones/README.md docs/milestones/0215-pdfium-comparison-tool-removal-decision-gate.md docs/reports/pdfium-comparison-tool-removal-decision-2026-06-29.md
```

The first scan exits with no matches, confirming active non-report/non-milestone
docs and fixtures no longer contain the stale runtime fallback expectation text.

Results:

- Native-only `cargo check` passed.
- Native-only `cargo test` passed.
- PDFium quarantine check passed.
- `pdfrust-cli` package listing contained only `.cargo_vcs_info.json`,
  `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, and `src/main.rs`.
- Poppler visual oracle summary: 5 fixtures, 5 accepted drift, 0 blockers,
  0 native errors, 0 reference errors.
- All-features clippy passed with `-D warnings`.
