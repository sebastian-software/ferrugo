# ferrugo

`ferrugo` is a Rust-native PDF preview engine for server-side thumbnails,
document intake, and automation workflows.

It is built for the practical job most applications need first: take an
untrusted PDF, render a bounded first-page preview, and do it without shipping a
large native renderer in the normal runtime path. The target documents are the
ones teams actually pass around: office exports, browser print PDFs, invoices,
reports, scans, forms, presentations, and mixed text-image documents.

The project combines a native renderer, a CLI, corpus tooling, benchmark gates,
and explicit diagnostics. It is already useful for scoped preview workloads, but
it is intentionally honest about the boundary: broad pixel-perfect PDF
compatibility is still a measured goal, not a claim.

## Current status

The native Rust path is the default development and packaging target.

- `ferrugo-cli` builds without PDFium by default.
- `render` and `render-auto` use the Rust-native backend.
- Runtime fallback to external PDF renderers has been removed from normal
  rendering.
- Reference-renderer comparison commands remain available for maintainers behind
  explicit Cargo features.
- The 1.4 readiness gate supports a scoped native server/runtime path.
- A broad "drop-in PDF renderer replacement" claim is still deferred.

The current renderer handles a useful slice of typical preview documents, but
PDF is a large format. Some visual-fidelity gaps and typed unsupported feature
boundaries are tracked in reports, policies, and backlogs.

## What you can use it for today

Good fit:

- Generate preview thumbnails in a Rust service or local CLI workflow.
- Test a Rust-native renderer against a generated PDF corpus.
- Compare native output against reference renderers during maintainer work.
- Study a staged approach to replacing a C/C++ PDF renderer with Rust modules.
- Run bounded server-side rendering experiments with explicit memory and
  timeout budgets.

Not a good fit yet:

- A full interactive PDF viewer.
- PDF editing, signing, full JavaScript execution, or dynamic XFA support.
- A guaranteed pixel-perfect replacement for every PDFium-supported document.
- A browser-first WASM product. WASM is tested as a compatibility profile, not
  the main runtime.

## Quick start

Requirements:

- Rust 1.81 or newer.
- A normal Cargo toolchain.
- No PDFium library for the native-only path.

Run the native test suite:

```sh
cargo test --workspace --no-default-features
```

Render a generated fixture with the native backend:

```sh
cargo run -p ferrugo-cli --no-default-features -- \
  render fixtures/generated/text-page.pdf \
  --max-edge 256 \
  --output target/text-page.png
```

Force the native backend explicitly:

```sh
cargo run -p ferrugo-cli --no-default-features -- \
  render-native fixtures/generated/text-page.pdf \
  --max-edge 256 \
  --output target/text-page-native.png
```

Run the local native-only release gate:

```sh
bash scripts/check_native_only_release.sh
```

That gate checks the native build, tests, plugin-free packaging boundary,
PDFium quarantine, package file lists, and all-features Clippy.

## How it works

The workspace is split into small crates so each layer can be tested on its own.

| Crate | Role |
| --- | --- |
| `ferrugo-thumbnail` | Public thumbnail facade, shared errors, options, and output types. |
| `ferrugo-native` | Rust-native PDF backend for metadata inspection and thumbnail rendering. |
| `ferrugo-syntax` | Low-level PDF byte parsing. |
| `ferrugo-object` | Object graph, xref, streams, and document structure. |
| `ferrugo-content` | Content stream tokenization and operator handling. |
| `ferrugo-render` | Display-list and raster rendering pieces. |
| `ferrugo-cli` | Local CLI for rendering, corpus analysis, benchmarks, and reports. |
| `ferrugo-pdfium` | Optional PDFium backend for maintainer comparison workflows. |
| `ferrugo-wasm-smoke` | Small WASM smoke crate for secondary compatibility checks. |

The public boundary is the thumbnail facade and native backend. PDFium handles,
fallback state, and comparison-only commands are not part of the normal runtime
API.

## Performance snapshot

Recent local smoke numbers on macOS/aarch64, `max_edge=160`:

| Profile | Result |
| --- | ---: |
| Low-memory corpus | 5/5 rendered natively, 0 fallbacks, 0 errors |
| Low-memory common docs | 4.815 ms mean |
| Low-memory scan fixture | 41.876 ms mean |
| Low-memory vector-stress fixture | 139.301 ms mean |
| Server batch | 16/16 jobs rendered natively, 0 budget failures |
| Server batch throughput | 38.025 jobs/sec |
| Server batch latency | 28.381 ms mean, 139.118 ms p95 |

The serverless profile is size-oriented and has a recent readiness measurement
of about 1.0 MB for the native CLI binary, with first-render p95 under 6 ms for
the small text fixture. Treat these as local gate numbers, not universal
hardware-independent benchmarks.

Against mature native renderers, the honest picture is mixed. Ferrugo is already
attractive for small, bounded, server-side preview jobs because the supported
runtime is compact, Rust-native, and explicitly budgeted. It is not generally
faster than PDFium today: archived same-corpus smoke runs show PDFium ahead on
many vector, form, presentation, and report cases, while Ferrugo is competitive
on simple text and scan-style thumbnails. The next performance work uses the
new `benchmark-matrix` harness to compare Ferrugo, PDFium, and Poppler across
cold-process time, hot-render distributions, output size, and RSS where the host
can expose it. See [Renderer benchmarks](docs/benchmarks.md) for the current
comparison state, the data-first optimization loop, and the
[performance claims policy](docs/policies/performance-claims.md) that applies
before README or release copy strengthens speed or memory statements.

## Reference renderers

External renderers are treated as behavior oracles, not as the architecture to
copy.

That distinction matters. `ferrugo` uses Rust ownership, typed errors, explicit
budgets, and narrow unsafe boundaries. When PDFium or Poppler are used, they
answer "what should this document look like?" or "where did the native renderer
drift?", not "what should the runtime depend on?"

To build and run PDFium comparison commands, enable the feature explicitly:

```sh
cargo build -p ferrugo-cli --features pdfium
cargo test -p ferrugo-cli --features pdfium
```

Then point the CLI at a local PDFium dynamic library:

```sh
export FERRUGO_PDFIUM_LIBRARY="/path/to/libpdfium.dylib"
export DYLD_LIBRARY_PATH="/path/to/pdfium/lib"
```

See [PDFium checkout recipe](docs/build/pdfium-checkout.md) for the local
source-build path used by maintainers.

## Safety and resource limits

PDF input is treated as untrusted. The native path uses typed public errors for
malformed input, encryption boundaries, unsupported features, and budget
exhaustion. Parser, font, image, raster, transparency, cache, and text paths all
have explicit limits.

Default thumbnail behavior is intentionally bounded:

- page index: `0`;
- max edge: `1024` pixels;
- timeout: `5s`;
- output: RGBA internally, PNG for CLI artifacts.

The server and low-memory gates track binary size, startup time, render latency,
in-flight pixel budgets, and cache behavior. See
[Packaging](docs/packaging.md) and [Rust-native backend](docs/backend/native.md)
for the details.

## Development commands

Common checks:

```sh
cargo fmt --check
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Useful project gates:

```sh
bash scripts/check_pdfium_quarantine.sh
bash scripts/check_performance_claims.sh
bash scripts/check_performance_matrix_smoke.sh
bash scripts/check_plugin_free_distribution.sh
bash scripts/check_native_only_release.sh
bash scripts/check_wasm_smoke.sh
```

Generated fixtures live in `fixtures/generated/`. Reports usually write JSON or
PNG output under `target/` so normal runs do not dirty the repository.

## Documentation map

Start here:

- [Documentation guide](docs/README.md) for a reader-friendly map of the docs.
- [Rust-native backend](docs/backend/native.md) for the current renderer
  contract and limits.
- [Packaging](docs/packaging.md) for native-only, serverless, plugin-free, and
  PDFium-enabled builds.
- [Renderer benchmarks](docs/benchmarks.md) and the
  [initial performance matrix report](docs/reports/performance-matrix-initial-2026-06-29.md)
  for comparative speed, memory, and reference-renderer measurement.
- [Native renderer conformance backlog](docs/backlogs/native-renderer-conformance-backlog.md)
  for follow-up renderer work.
- [PDFium-free 1.4 readiness report](docs/reports/pdfium-free-1-4-readiness-2026-06-29.md)
  for the current release decision.

Historical and planning docs:

- [Rendering landscape](docs/research/2026-06-24-rendering-landscape.md)
- [Rust-first, PDFium-guided decision](docs/decisions/0001-rust-first-pdfium-guided-porting.md)
- [Phase 0 decisions](docs/plans/phase-0-decisions.md)
- [Roadmap](docs/roadmap.md)
- [Attribution policy](docs/policies/attribution.md)

## Licensing

Repository code and documentation are licensed under either MIT or Apache-2.0,
at your option.

PDFium, Poppler, and other renderers may be used as behavioral references under
the project's attribution policy. Their source code is not vendored here.
