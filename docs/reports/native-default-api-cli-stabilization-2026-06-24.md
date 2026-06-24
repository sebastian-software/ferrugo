# Native Default API And CLI Stabilization 2026-06-24

Milestone: 0082.

## Decision

`pdfrust-cli render` and `render-auto` are native-default. PDFium is no longer
selected silently for unsupported native documents, even in a PDFium-enabled
build. Callers must opt in with `--allow-pdfium-fallback`.

## Behavior

Supported documents:

- `render` and `render-auto` run the Rust-native backend first.
- If native succeeds, PDFium is not loaded or selected.
- `render-native` remains the explicit native-only command.

Unsupported native documents:

- Default behavior returns a stable `unsupported` render error with the native
  fallback reason.
- `--allow-pdfium-fallback` permits retrying through PDFium in a build compiled
  with `--features pdfium`.
- `--deny-fallback-reason <bucket>` can still block specific fallback buckets
  after fallback is enabled.
- `--native-only` and `--no-pdfium-fallback` force fallback denial.

PDFium-specific workflows:

- `render-pdfium`, `render-isolated`, `compare-metadata`, and
  `benchmark-pdfium` still require `--features pdfium`.
- PDFium remains the comparison oracle and emergency fallback, not the default
  rendering path.

## Smoke Results

Native default supported fixture:

```text
cargo run -p pdfrust-cli --no-default-features -- render fixtures/generated/vector-paths.pdf --max-edge 120 --output target/0082-default-native-vector.png
```

Result: `render backend: native`.

Unsupported fixture without fallback opt-in:

```text
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- render-auto fixtures/generated/optional-content-ocmd.pdf --max-edge 120 --output target/0082-should-require-fallback.png
```

Result: failed with `PDFium fallback not enabled for graphics.optional-content`.

Unsupported fixture with explicit fallback opt-in:

```text
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- render-auto fixtures/generated/optional-content-ocmd.pdf --allow-pdfium-fallback --max-edge 120 --output target/0082-explicit-fallback.png
```

Result: `render backend: pdfium fallback_reason=graphics.optional-content`.

## Validation

```text
cargo fmt --check
cargo check
cargo test
cargo test -p pdfrust-cli --no-default-features
cargo test -p pdfrust-cli --features pdfium
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.
