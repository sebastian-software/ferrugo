# 0176: WASM Viewer Integration Performance Gate

Status: done
Phase: 33
Size: medium
Depends on: 0175

## Goal

Validate WASM viewer integration as a secondary deployment profile, after the
server-side Rust-native renderer has the relevant correctness and resource
behavior for typical documents.

## Scope

- Define WASM package size, initialization, first-page, and thumbnail latency
  budgets.
- Identify renderer dependencies that block or bloat WASM builds.
- Add a small viewer-oriented smoke harness for supported documents.
- Document unsupported APIs or features in WASM mode.
- Keep WASM findings as profile-specific follow-up unless they expose a shared
  renderer correctness or safety defect.

## Non-Goals

- Build a full production web viewer.
- Add PDFium, native plugins, or browser-only dependencies.
- Let WASM packaging or latency concerns block server-side PDFium replacement
  gates by themselves.

## Deliverables

- WASM viewer performance report.
- Package size and latency budget checks.
- Follow-up backlog for WASM-specific improvements.

## Acceptance Criteria

- WASM build is PDFium-free and has documented feature flags.
- Typical sample documents render within agreed viewer budgets.
- Unsupported WASM features are explicit.
- Shared renderer correctness failures are promoted to the main backlog; purely
  browser-profile limitations remain secondary.

## Validation

- Run WASM build check where available.
- Run native-only `cargo test`.
- Run viewer smoke harness.
- Measure package size and first-page latency.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Completed on 2026-06-28 as a secondary WASM viewer-profile gate.
- Reused the PDFium-free `pdfrust-wasm-smoke` package from 0132 as the current
  viewer integration boundary. It exports `pdfrust_wasm_smoke_status` and
  renders a supported low-memory thumbnail without depending on PDFium,
  native plugins, or `pdfrust-cli`.
- Fresh gate run passed with artifact size 723687 bytes, compile 2.231 ms,
  instantiate 0.096 ms, smoke render 5.970 ms, and 96x51 output.
- 0176 report copy: `target/wasm-0176-smoke.json` measured compile 1.028 ms,
  instantiate 0.076 ms, smoke render 5.687 ms, and 96x51 output.
- Unsupported WASM surface remains explicit: production browser viewer
  bindings, browser-only APIs, and further size optimization are follow-up
  profile work unless they expose shared renderer correctness or safety
  defects.
- Report: `docs/reports/wasm-viewer-integration-performance-2026-06-28.md`.
