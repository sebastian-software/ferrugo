# Fuzzing And Adversarial Checks

The repository keeps a fast deterministic smoke gate for pull requests and real
coverage-guided fuzz targets for scheduled or local hardening runs. The `fuzz/`
package is a standalone Cargo project that supports both modes:

- normal `cargo run --manifest-path fuzz/Cargo.toml --bin <target> -- --smoke`
  replays deterministic seeds and mutations without external fuzzing tools;
- `cargo fuzz run <target>` uses libFuzzer against the same target entrypoint
  and committed seed corpus under `fuzz/corpus/<target>/`.

Run all current smoke targets:

```sh
bash scripts/check_fuzz_smoke.sh
```

For the scoped 1.0 release train, this command is part of
`scripts/check_native_only_release.sh`. A passing release-candidate run must
produce:

- one successful smoke line for each target in the matrix below;
- `target/fuzz-smoke-summary.txt` with the target list and smoke-case counts;
- the final `Fuzz smoke gate passed` line.

The smoke gate runs quickly from committed fixtures and seeds, needs no PDFium,
Poppler, network, or private corpus files, and keeps pull-request coverage
reproducible from a clean native-only checkout.

Run one target against saved inputs:

```sh
cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- fixtures/adversarial/truncated-header.pdf
cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- fixtures/adversarial/huge-image-dimensions.pdf
```

Run the bounded coverage-guided campaign used by scheduled CI:

```sh
cargo install cargo-fuzz --version 0.13.2 --locked
bash scripts/run_fuzz_campaign.sh
```

The default campaign runs each target for a bounded number of libFuzzer
iterations and seconds. Override locally with:

```sh
FUZZ_RUNS=4096 FUZZ_MAX_TOTAL_TIME=120 bash scripts/run_fuzz_campaign.sh
```

The committed fuzz seed corpus lives under `fuzz/corpus/<target>/` and is seeded
from generated fixtures plus `fixtures/adversarial/`. The adversarial fixtures
are intentionally reduced and reviewable; add a minimized input there when a
fuzz run finds a panic, excessive-work case, or unstable error mapping. Add
target-specific seed bytes under `fuzz/corpus/<target>/` when they should guide
future libFuzzer exploration but are not themselves regression fixtures.

Current targets:

| Target | Covered path | Security focus |
| --- | --- | --- |
| `primitive_parse` | PDF primitive parsing and prefix parsing | nesting, malformed scalars, offset accounting |
| `xref_load` | indirect object parsing, classic xref loading, modern xref loading | object graph corruption, offset drift, expansion limits |
| `stream_decode` | stream object parsing and bounded filter decoding | decode expansion and malformed filter data |
| `content_tokenize` | decoded content stream tokenization and inline-image parsing | unterminated data and operand/operator ambiguity |
| `render_setup` | native metadata inspection and first-page render setup | page setup, declared image dimensions, renderer budgets |
| `render` | native first-page thumbnail rendering | end-to-end parser, resource, raster, and budget interaction |

Current minimized adversarial inputs:

| Input | Expected boundary |
| --- | --- |
| `truncated-header.pdf` | malformed native metadata/render setup |
| `huge-image-dimensions.pdf` | `renderer.memory-budget` before sample allocation |
| `deep-primitive-array.input` | primitive parser nesting budget |
| `unterminated-inline-image.content` | content tokenizer `UnexpectedEof` |

Panics are not caught by the harness. A panic or abort fails the smoke command
and should be minimized into `fixtures/adversarial/` before the code path is
hardened.

`.github/workflows/scheduled-fuzz.yml` runs the deterministic smoke gate and
then the bounded cargo-fuzz campaign on a nightly schedule and through manual
dispatch. Crash artifacts are uploaded from `target/fuzz-artifacts/` and
`fuzz/artifacts/` for triage.

See `docs/policies/security-fuzz-triage.md` for finding classification,
private crash artifact handling, minimization rules, and nightly gate guidance.
