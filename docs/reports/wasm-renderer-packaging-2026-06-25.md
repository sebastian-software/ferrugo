# WASM Renderer Packaging 2026-06-25

Milestone: 0132.

## Decision

The Rust-native renderer now has a dedicated WASM packaging smoke gate. The
new `ferrugo-wasm-smoke` crate is publish-disabled, depends only on
`ferrugo-native` and `ferrugo-thumbnail`, and builds a `cdylib` for
`wasm32-unknown-unknown`. It intentionally excludes `ferrugo-cli` and
`ferrugo-pdfium`, so PDFium dynamic-library fallback is not part of the WASM
package path.

The smoke crate renders `text-page.pdf` through `NativeBackend::low_memory()`
at `max_edge 96`, exposes `ferrugo_wasm_smoke_status` for a browser or JS
harness, and has a host unit test for the same render path.

## Gate Artifacts

WASM artifact:
`target/wasm32-unknown-unknown/release/ferrugo_wasm_smoke.wasm`

Smoke report:
`target/wasm-0132-smoke.json`

| Metric | Measured | Gate |
| --- | ---: | ---: |
| Artifact size bytes | 716082 | 4194304 |
| WebAssembly compile ms | 0.967 | 250 |
| WebAssembly instantiate ms | 0.072 | 100 |
| Smoke render ms | 5.502 | 250 |
| Smoke width | 96 | 96 max edge |
| Smoke height | 51 | 96 max edge |

The gate is intentionally generous in this first packaging slice. It catches
large accidental dependency pulls, missing exports, runtime instantiation
failures, and smoke-render failures without pretending to be final bundle-size
optimization.

## Excluded Features

- `ferrugo-pdfium` is not a dependency of the smoke crate and is not included
  in the default workspace members.
- `ferrugo-cli` remains a native command-line surface and is not the WASM
  package entrypoint.
- PDFium fallback commands remain feature-gated behind the native CLI
  `pdfium` feature and are outside this WASM build.

## Smoke Harness

Run the full local gate with:

```text
sh scripts/check_wasm_smoke.sh
```

The script runs:

```text
cargo check -p ferrugo-wasm-smoke --target wasm32-unknown-unknown --no-default-features
cargo build -p ferrugo-wasm-smoke --target wasm32-unknown-unknown --release
node scripts/wasm_smoke.mjs target/wasm32-unknown-unknown/release/ferrugo_wasm_smoke.wasm target/wasm-0132-smoke.json
```

The default budgets are configurable with:

```text
FERRUGO_WASM_MAX_BYTES
FERRUGO_WASM_MAX_COMPILE_MS
FERRUGO_WASM_MAX_INSTANTIATE_MS
FERRUGO_WASM_MAX_SMOKE_MS
```

## Follow-Up Backlog

- Add a browser-driven WASM gate once a viewer integration exists.
- Add `wasm-opt` or equivalent size optimization only after the package shape
  is stable.
- Split larger browser API bindings from this smoke crate so the smoke gate
  stays small and deterministic.

## Validation Commands

```text
rustup target add wasm32-unknown-unknown
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p ferrugo-wasm-smoke -- --nocapture
sh scripts/check_wasm_smoke.sh
cargo test --workspace
cargo test --workspace --no-default-features
git diff --check -- Cargo.toml Cargo.lock crates/ferrugo-wasm-smoke/Cargo.toml crates/ferrugo-wasm-smoke/src/lib.rs scripts/check_wasm_smoke.sh scripts/wasm_smoke.mjs
```
