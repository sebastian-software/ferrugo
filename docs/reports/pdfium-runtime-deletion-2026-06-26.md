# PDFium Runtime Deletion Execution

Date: 2026-06-26
Milestone: 0141

## Summary

Milestone 0141 removes PDFium-backed runtime fallback dispatch from
`render` / `render-auto`. Normal rendering now uses the Rust-native backend
only. Unsupported native categories remain typed `unsupported` outcomes and are
tracked through corpus tooling instead of being retried through PDFium.

PDFium remains available behind the explicit `pdfium` feature for maintainer
comparison commands:

- `render-pdfium`
- `render-isolated`
- `compare-metadata`
- `benchmark-pdfium`
- `visual-diff`

This keeps PDFium as an oracle and probe backend without making it a normal
runtime dependency.

## Runtime Changes

- Removed `render_auto_thumbnail`'s PDFium fallback branch.
- Removed the runtime fallback policy and environment-driven fallback opt-ins.
- `--allow-pdfium-fallback` is now rejected for `render` / `render-auto` with a
  usage error.
- `--native-only` and `--no-pdfium-fallback` remain accepted for script
  compatibility; the render paths are already native-only.
- `--deny-fallback-reason <bucket>` remains accepted as a compatibility no-op
  for old scripts that pass it, but there is no runtime fallback to deny.
- Explicit PDFium commands still compile and run only with `--features pdfium`.

## Native-Only Gate

Command:

```sh
env -u PDFRUST_PDFIUM_LIBRARY -u DYLD_LIBRARY_PATH cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/pdfium-delete-0141-supported-gate.json
```

Result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 8 | 8 | 0 | 0 |
| `office-export` | 44 | 44 | 0 | 0 |
| `form` | 15 | 15 | 0 | 0 |
| **Core total** | **67** | **67** | **0** | **0** |

The gate ran in a native-only CLI build with PDFium environment variables
removed.

## Runtime Behavior Checks

Native runtime render without PDFium env:

```sh
env -u PDFRUST_PDFIUM_LIBRARY -u DYLD_LIBRARY_PATH cargo run -p pdfrust-cli --no-default-features -- render fixtures/generated/vector-paths.pdf --max-edge 120 --output target/pdfium-delete-0141-render.png
```

Result: succeeded and reported `render backend: native`.

Removed fallback flag:

```sh
env -u PDFRUST_PDFIUM_LIBRARY -u DYLD_LIBRARY_PATH cargo run -p pdfrust-cli --no-default-features -- render fixtures/generated/optional-content-ocmd.pdf --allow-pdfium-fallback --max-edge 120 --output target/pdfium-delete-0141-rejected.png
```

Result: expected non-zero usage error:

```text
PDFium runtime fallback has been removed from render/render-auto; use render-pdfium or maintainer comparison commands with --features pdfium
```

Explicit maintainer PDFium render:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- render-pdfium fixtures/generated/vector-paths.pdf --max-edge 120 --output target/pdfium-delete-0141-pdfium-render.png
```

Result: succeeded.

## Documentation Updates

- `docs/backend/native.md` now describes `render` / `render-auto` as native-only
  runtime rendering paths.
- `docs/backend/pdfium.md` now describes PDFium as explicit maintainer tooling.
- `docs/packaging.md` no longer documents runtime fallback as a PDFium-enabled
  CLI behavior.
- `docs/errors.md` now points CI users to native corpus gates instead of
  runtime fallback denial flags.
- `docs/backlogs/pdfium-free-maintenance-backlog.md` marks runtime fallback and
  environment fallback opt-ins as removed in 0141.

## Validation

Commands run:

```sh
cargo fmt --check
cargo test -p pdfrust-cli render_auto -- --nocapture
cargo test -p pdfrust-cli render_config_should_reject_explicit_pdfium_fallback_flag -- --nocapture
cargo check --workspace --no-default-features
cargo test -p pdfrust-cli --features pdfium
env -u PDFRUST_PDFIUM_LIBRARY -u DYLD_LIBRARY_PATH cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/pdfium-delete-0141-supported-gate.json
env -u PDFRUST_PDFIUM_LIBRARY -u DYLD_LIBRARY_PATH cargo run -p pdfrust-cli --no-default-features -- render fixtures/generated/vector-paths.pdf --max-edge 120 --output target/pdfium-delete-0141-render.png
env -u PDFRUST_PDFIUM_LIBRARY -u DYLD_LIBRARY_PATH cargo run -p pdfrust-cli --no-default-features -- render fixtures/generated/optional-content-ocmd.pdf --allow-pdfium-fallback --max-edge 120 --output target/pdfium-delete-0141-rejected.png
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- render-pdfium fixtures/generated/vector-paths.pdf --max-edge 120 --output target/pdfium-delete-0141-pdfium-render.png
cargo package -p pdfrust-syntax --allow-dirty --no-verify
cargo package -p pdfrust-thumbnail --allow-dirty --no-verify
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Expected non-zero validation:

- The `--allow-pdfium-fallback` runtime render check now fails by design.
