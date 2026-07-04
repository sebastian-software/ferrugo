# Scoped Native 1.0 Release Train 2026-07-04

Status: accepted scope decision for issue #58.

## Decision

Use 1.0 as the planning name for the first stable, scoped native
server/runtime release train. Do not force a `0.9`, `1.0`, or other target
version in Cargo manifests, changelogs, Release Please config, or release
notes before the code and docs justify it.

The version line remains commit-driven:

- use Conventional Commits for every release-affecting change;
- let Release Please infer the next version from the merged change history;
- keep `.release-please-manifest.json` and crate `Cargo.toml` versions aligned
  to the current released or prepared crate versions;
- reserve an explicit version override for a concrete release-management need.

## Product Claim

Ferrugo is a Rust-native PDF preview engine for bounded server-side thumbnails,
document intake, and automation workflows. The stable release claim is scoped to
the native, PDFium-free runtime path:

- Rust-native thumbnail and preview rendering is the normal runtime path;
- PDFium remains optional maintainer oracle tooling behind explicit features;
- server/runtime usage is bounded by page index, max edge, timeout, memory, and
  typed unsupported-feature diagnostics;
- release readiness is proven by local native-only and crate-publish gates.

This is the claim the README, packaging guide, release notes, and changelogs
should support.

## Explicit Non-Claim

Do not claim broad drop-in PDF renderer replacement readiness.

That broader claim stays deferred until visual-fidelity, optional-content,
specialized codec, transparency, annotation/form, dense table/report, and
multi-oracle evidence changes. A passing native server/runtime gate is not the
same thing as pixel-perfect compatibility across arbitrary PDFs.

## Release Blockers

These must be resolved before the scoped stable release can be described as
ready:

- #61 freezes the public API and CLI consumer contract.
- #59 cleans up Phase 0 ADR and planning drift so durable docs are the source of
  truth.
- #63 provides the user guide for the scoped native path.
- #62 proves the CLI end-to-end contract.
- #70 makes fuzzing and adversarial coverage visible enough for release.
- #60 creates the durable benchmark suite that supports performance claims.
- #68 turns native-only golden image comparison into a release gate.
- #64 confirms comparison providers beyond PDFium and Poppler.

## Non-Blockers For This Scope

These areas remain visible, but they do not block the scoped native
server/runtime release unless the issue-specific evidence changes:

- #66 optional-content membership policy;
- #71 office text and font visual-fidelity blockers;
- #72 dense table/grid and report rendering-core blockers;
- #69 static form and annotation appearance parity;
- #67 CCITT, JBIG2, and JPX scan codec strategy;
- #65 transparency soft-mask and advanced blend boundaries;
- #49, #50, #51, and #52 performance implementation slices.

If one of these becomes release-blocking, update this report, #58, and the
tracking epic before changing release copy.

## Post-Scoped-Release Work

Post-release work can broaden the product claim only after the corresponding
evidence exists:

- multi-oracle visual parity across real producer families;
- optional-content UI state and membership semantics;
- specialized image codecs for scan/fax/archive documents;
- advanced transparency, soft masks, blends, and color-management fidelity;
- annotation appearance and static form parity;
- dense office tables, reports, and spreadsheet layouts;
- platform/API direction beyond the server/runtime preview path.

## Required Local Gates

These are the named local release gates for the scoped native release train:

```sh
bash scripts/check_native_only_release.sh
bash scripts/check_crate_publish_ready.sh
cargo test -p ferrugo --test cli_contract --no-default-features
bash scripts/check_fuzz_smoke.sh
bash scripts/check_benchmark_suite.sh
bash scripts/check_native_golden_images.sh
```

`check_native_only_release.sh` verifies the native-only workspace path, PDFium
quarantine, plugin-free distribution, CLI binary contract behavior,
fuzz/adversarial smoke coverage, benchmark suite smoke coverage, native golden
image coverage, package file list, leaf package dry-runs, and all-features
Clippy. Set
`FERRUGO_NATIVE_RELEASE_VERIFY_REGISTRY=1` only when registry-backed package
verification is available.

The `cli_contract` integration test invokes the built `ferrugo` binary as a
process and covers help/version output, successful native rendering, stable JSON
report shapes, durable usage errors, and native-only PDFium-disabled failures
without requiring PDFium, Poppler, or private corpus files.

`check_fuzz_smoke.sh` runs the required fuzz/adversarial release matrix for
`primitive_parse`, `xref_load`, `stream_decode`, `content_tokenize`, and
`render_setup`. It writes `target/fuzz-smoke-summary.txt`; the expected artifact
contains one completed smoke-case line per target plus `Fuzz smoke gate passed`.
Scheduled or long-running fuzz campaigns remain post-scoped-release hardening
unless this local release gate discovers a concrete crash, panic, timeout, or
unstable error-boundary gap.

`check_benchmark_suite.sh` runs the required benchmark release smoke over the
committed `small-text` performance-matrix family with the native backend only.
It writes `target/benchmark-suite/performance-matrix-smoke.json`,
`target/benchmark-suite/performance-matrix-smoke.md`, and
`target/benchmark-suite/benchmark-suite-summary.txt`. This gate proves the
durable benchmark JSON/Markdown contract and native timing path without
reference renderer availability.

`check_native_golden_images.sh` runs `ferrugo compare-golden` against
`fixtures/native-golden-manifest.tsv` with PDFium disabled. It writes
`target/native-golden/native-golden-comparison.json` and
`target/native-golden/native-golden-summary.txt`. The committed manifest covers
browser print, office export, static form, scanner, and PDF 2.0 accepted basics
as hash-only reviewed baselines without requiring PDFium, Poppler, private
corpus files, or broad cross-renderer performance claims.

`check_crate_publish_ready.sh` verifies Cargo metadata, package file lists, leaf
package archive dry-runs, and optionally registry-backed dependency-chain
package dry-runs with `FERRUGO_VERIFY_REGISTRY_PACKAGES=1`.

## Release Order

Publish the workspace crates in dependency order:

1. `ferrugo-syntax`
2. `ferrugo-thumbnail`
3. `ferrugo-simd`
4. `ferrugo-object`
5. `ferrugo-content`
6. `ferrugo-render`
7. `ferrugo-native`
8. `ferrugo-pdfium` for maintainer comparison workflows
9. `ferrugo`

Release Please keeps the train linked through the Cargo workspace and
`linked-versions` plugin. The publish workflow then skips versions that already
exist on crates.io and publishes missing crates in dependency order.

## Current Metadata State

The current Cargo manifests and `.release-please-manifest.json` use the prepared
`0.1.x` line. That is intentional and should not be rewritten to a placeholder
`0.9` or `1.0` plan. A future major version should appear because the merged
Conventional Commit history and Release Please release PR produce it, or because
maintainers deliberately choose a documented release override.
