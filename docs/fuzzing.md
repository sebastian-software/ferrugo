# Fuzzing And Adversarial Smoke Checks

The repository keeps fuzzing optional for local development. The `fuzz/`
package is a small standalone Cargo project with deterministic smoke targets
that exercise parser and render setup paths without requiring network access or
external fuzzing tools.

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

The smoke gate is intentionally local-release-gate coverage rather than a
scheduled CI campaign for this slice. It runs quickly from committed fixtures
and seeds, needs no PDFium, Poppler, network, or private corpus files, and keeps
the scoped release claim reproducible from a clean native-only checkout.
Long-running scheduled fuzz campaigns remain post-scoped-release hardening until
the project has a broader CI workflow surface than the current publish workflow.

Run one target against saved inputs:

```sh
cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- fixtures/adversarial/truncated-header.pdf
cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- fixtures/adversarial/huge-image-dimensions.pdf
```

The committed adversarial corpus lives in `fixtures/adversarial/`. These files
are intentionally reduced and reviewable; add a minimized input there when a
fuzz run finds a panic, excessive-work case, or unstable error mapping.

Current targets:

| Target | Covered path | Security focus |
| --- | --- | --- |
| `primitive_parse` | PDF primitive parsing and prefix parsing | nesting, malformed scalars, offset accounting |
| `xref_load` | indirect object parsing, classic xref loading, modern xref loading | object graph corruption, offset drift, expansion limits |
| `stream_decode` | stream object parsing and bounded filter decoding | decode expansion and malformed filter data |
| `content_tokenize` | decoded content stream tokenization and inline-image parsing | unterminated data and operand/operator ambiguity |
| `render_setup` | native metadata inspection and first-page render setup | page setup, declared image dimensions, renderer budgets |

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

See `docs/policies/security-fuzz-triage.md` for finding classification,
private crash artifact handling, minimization rules, and nightly gate guidance.
