# PDFium Comparison Tooling Quarantine

Date: 2026-06-26
Milestone: 0142

## Summary

Milestone 0142 keeps PDFium available only as explicit maintainer oracle
tooling. Normal runtime rendering remains native-only, and the remaining
PDFium comparison commands are guarded by the `pdfium` feature plus a
regression scan that catches accidental runtime reintroduction.

The private `render-worker` entry point is no longer directly callable as a
normal CLI command. It requires the internal `FERRUGO_PDFIUM_RENDER_WORKER`
environment marker, which `render-isolated` sets only for its single-use child
process.

## Quarantine Boundary

PDFium-enabled maintainer commands remain:

- `render-pdfium`
- `render-isolated`
- `compare-metadata`
- `benchmark-pdfium`
- `visual-diff`

Runtime commands remain native-only:

- `render`
- `render-auto`
- `render-native`
- `summarize-fallbacks`
- `extract-corpus-metadata`
- `benchmark-native`

The `render-worker` command is private child-process plumbing. Direct
invocation now fails with:

```text
usage error: render-worker is private maintainer tooling; use render-isolated
```

## Regression Check

Added:

```sh
bash scripts/check_pdfium_quarantine.sh
```

The script verifies:

- `cargo tree -p ferrugo-cli --no-default-features` has no `ferrugo-pdfium`
  dependency edge.
- Runtime crates do not contain forbidden PDFium integration symbols such as
  `ferrugo_pdfium`, `PdfiumBackend`, or `FERRUGO_PDFIUM`.

The checked runtime crates are:

- `ferrugo-content`
- `ferrugo-native`
- `ferrugo-object`
- `ferrugo-render`
- `ferrugo-syntax`
- `ferrugo-thumbnail`
- `ferrugo-wasm-smoke`

Result:

```text
PDFium quarantine check passed
```

## Maintainer Oracle Probe

Command:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated/vector-paths.pdf --max-edge 120 --output target/pdfium-quarantine-0142-visual-diff.json
```

Result:

| Fixture | Status | Blockers | Native errors | PDFium errors |
| --- | --- | ---: | ---: | ---: |
| `fixtures/generated/vector-paths.pdf` | `accepted_drift` | 0 | 0 | 0 |

Metrics:

- `changed_ratio`: `0.036054`
- `mean_abs_error`: `1.067`
- `p95_channel_delta`: `0`
- `max_channel_delta`: `229`

This confirms that the PDFium comparison path still works when explicitly
enabled, without becoming part of normal runtime rendering.

## Validation

Commands run:

```sh
cargo fmt --check
bash scripts/check_pdfium_quarantine.sh
cargo check --workspace --no-default-features
cargo check --workspace --all-features
cargo test --workspace --no-default-features
cargo test -p ferrugo-cli --features pdfium render_worker_should_reject_direct_cli_invocation -- --nocapture
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated/vector-paths.pdf --max-edge 120 --output target/pdfium-quarantine-0142-visual-diff.json
```

All commands passed.

During validation, `cargo test --workspace --no-default-features` first exposed
an unused constant warning in the no-feature test build. The constant was
narrowed to the `pdfium` feature path and the native-only test suite was rerun
successfully.

## Follow-Up

The next milestone can use this quarantine boundary as the baseline for native
renderer conformance triage. Any new unsupported bucket should remain a typed
native outcome unless a maintainer-only oracle job is explicitly requested.
