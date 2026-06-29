# WASM Low Memory Mobile Browser Gate

Date: 2026-06-29
Milestone: 0196

## Summary

Milestone 0196 promotes the WASM packaging smoke from a single inline PDF to a
representative low-memory preview suite. The suite remains a secondary browser
compatibility signal: failures only block server-side PDFium-free work when
they expose shared renderer correctness, safety, or unbounded allocation
defects.

New coverage:

- `crates/ferrugo-wasm-smoke/src/lib.rs`
- `scripts/check_wasm_smoke.sh`
- `scripts/wasm_smoke.mjs`
- `target/wasm-0196-mobile-smoke.json`

## Mobile Preview Suite

The WASM smoke crate embeds common preview fixtures and renders them through
`NativeBackend::low_memory()` with `max_edge = 96`, RGBA output, screen
annotations, and document-state form appearances.

| Fixture | Source | Workflow |
| --- | --- | --- |
| `text-page` | `fixtures/generated/text-page.pdf` | Plain text preview |
| `browser-print` | `fixtures/generated/browser-chromium-article-print.pdf` | Browser print preview |
| `mobile-scan` | `fixtures/generated/mobile-cropped-photo-scan.pdf` | Camera or scan preview |
| `form-preview` | `fixtures/generated/acroform-text-field.pdf` | AcroForm thumbnail preview |
| `invoice-preview` | `fixtures/generated/business-invoice-dense.pdf` | Dense business document preview |

The exported WASM API now reports the fixture count and total rendered RGBA
bytes in addition to the packed smoke status. That keeps allocation pressure
observable without adding a viewer UI or browser-only dependencies to the
crate.

## Gate Budgets

Command:

```sh
FERRUGO_WASM_REPORT=target/wasm-0196-mobile-smoke.json bash scripts/check_wasm_smoke.sh
```

Result:

| Metric | Measured | Gate |
| --- | ---: | ---: |
| Artifact size bytes | 737477 | 4194304 |
| WebAssembly compile ms | 1.026 | 250 |
| WebAssembly instantiate ms | 0.073 | 100 |
| Smoke render ms | 18.202 | 250 |
| Fixture count | 5 | 5 minimum |
| Total RGBA output bytes | 122496 | 524288 |
| First smoke output | 96x51 | 96 max edge |

The low-memory WASM suite stayed within all configured binary, startup,
rendering, and output-size thresholds.

## Failure Typing

The browser-compatible Node WebAssembly harness exits with reproducible failure
classes:

- Exit `2`: invalid harness invocation.
- Exit `3`: required WASM export missing.
- Exit `4`: renderer returned a zero smoke status.
- Exit `5`: rendered output byte export returned zero.
- Exit `1`: one or more budget checks failed.

The Rust suite maps fixture render failures to stable fixture labels, so a
regression identifies the workflow slice that failed before it reaches the
budget layer.

## Constraints And Follow-Ups

- This is not a full browser viewer UI and does not attempt to cover all
  server-side renderer features in WASM.
- The harness uses the standard WebAssembly API from Node, which keeps the
  package smoke deterministic; a Playwright or device-browser harness can be
  added later if client-side thumbnail shipping becomes a primary product path.
- PDFium and native dynamic libraries remain out of scope for browser delivery.
- Future optimization backlog: per-fixture timing export, wasm-opt comparison,
  and a larger client-thumbnail fixture matrix if WASM moves from secondary
  signal to supported runtime.

## Validation

Commands run:

```sh
cargo fmt --check
cargo check --workspace --no-default-features
cargo test -p ferrugo-wasm-smoke --no-default-features -- --nocapture
FERRUGO_WASM_REPORT=target/wasm-0196-mobile-smoke.json bash scripts/check_wasm_smoke.sh
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-default-features
```
