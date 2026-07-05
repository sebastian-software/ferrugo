# Coverage-Guided Fuzzing

Date: 2026-07-05
Issue: #104

## Summary

The fuzz package now supports both the fast deterministic PR smoke gate and real
`cargo-fuzz`/libFuzzer runs. Each target keeps a normal `cargo run` smoke mode
and compiles to `libfuzzer_sys::fuzz_target!` when built by `cargo fuzz`.

The target matrix is:

- `primitive_parse`
- `xref_load`
- `stream_decode`
- `content_tokenize`
- `render_setup`
- `render`

The new `render` target exercises native first-page thumbnail rendering with
tight max-edge and timeout budgets. The existing `render_setup` target continues
to exercise metadata inspection plus first-page setup.

## Corpus

Committed seed corpora live under `fuzz/corpus/<target>/`. They include:

- parser and content seeds for primitive, xref, stream, and content targets;
- adversarial fixtures from `fixtures/adversarial/`;
- generated image/filter fixtures including LZW, RunLength, CCITT, predictor,
  inline image, image XObject, and scan-page samples for render targets.

Crash artifacts stay out of git through `fuzz/.gitignore`. Minimized regression
inputs that should remain durable still belong in `fixtures/adversarial/`.

## Scheduled Gate

`.github/workflows/scheduled-fuzz.yml` runs nightly and by manual dispatch. It:

- installs pinned `cargo-fuzz` 0.13.2 on nightly Rust;
- runs `scripts/check_fuzz_smoke.sh`;
- runs `scripts/run_fuzz_campaign.sh` with bounded `FUZZ_RUNS` and
  `FUZZ_MAX_TOTAL_TIME`;
- uploads `target/fuzz-artifacts/` and `fuzz/artifacts/` on failure.

## Validation

Commands run for this slice:

```sh
cargo check --manifest-path fuzz/Cargo.toml --bins --locked
cargo clippy --manifest-path fuzz/Cargo.toml --all-targets --locked -- -D warnings
bash scripts/check_fuzz_smoke.sh
cargo fuzz list
FUZZ_RUNS=1 FUZZ_MAX_TOTAL_TIME=5 bash scripts/run_fuzz_campaign.sh
```
