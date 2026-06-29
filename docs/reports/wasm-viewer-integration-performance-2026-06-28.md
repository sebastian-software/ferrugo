# WASM Viewer Integration Performance 2026-06-28

Milestone: 0176.
Status: done.

## Summary

The existing `ferrugo-wasm-smoke` package remains PDFium-free and validates the
Rust-native low-memory thumbnail path as a secondary viewer-profile signal.
This milestone does not introduce a production browser viewer. It confirms that
the current WASM package can compile, instantiate, and render a supported first
thumbnail through the exported smoke entrypoint without pulling in PDFium,
native plugins, or the CLI surface.

WASM remains secondary to the server-side PDFium replacement path. Any future
browser-only API, viewer shell, or size optimization work should stay in the
WASM follow-up backlog unless it exposes a shared renderer correctness or
safety defect.

## Gate Artifacts

WASM artifact:
`target/wasm32-unknown-unknown/release/ferrugo_wasm_smoke.wasm`

Script report:
`target/wasm-0132-smoke.json`

0176 report copy:
`target/wasm-0176-smoke.json`

| Metric | Measured | Gate |
| --- | ---: | ---: |
| Artifact size bytes | 723687 | 4194304 |
| WebAssembly compile ms | 2.231 | 250 |
| WebAssembly instantiate ms | 0.096 | 100 |
| Smoke render ms | 5.970 | 250 |
| Smoke width | 96 | 96 max edge |
| Smoke height | 51 | 96 max edge |

The 0176 report copy measured the same artifact immediately after the gate run:
compile 1.028 ms, instantiate 0.076 ms, smoke render 5.687 ms, 96x51 output.
Both measurements are comfortably within the configured viewer-profile budgets.

## Unsupported WASM Surface

- PDFium is not part of the WASM package path.
- `ferrugo-cli` remains a native command-line surface and is not exported to
  WASM.
- Browser viewer bindings are still deferred; the smoke export is the current
  integration boundary.
- WASM package or latency findings are profile-specific unless they reveal a
  shared native renderer correctness, memory, or safety defect.

## Validation

Commands run:

```text
sh scripts/check_wasm_smoke.sh
node scripts/wasm_smoke.mjs target/wasm32-unknown-unknown/release/ferrugo_wasm_smoke.wasm target/wasm-0176-smoke.json
cargo test -p ferrugo-wasm-smoke --no-default-features -- --nocapture
```

Results:

- `scripts/check_wasm_smoke.sh` passed all size, compile, instantiate, and smoke
  render budgets.
- `ferrugo_wasm_smoke_status` returned a non-zero status encoding a 96x51
  thumbnail.
- `cargo test -p ferrugo-wasm-smoke --no-default-features` passed 2 tests and
  0 doctests.
