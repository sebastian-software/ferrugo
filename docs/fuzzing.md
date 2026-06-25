# Fuzzing And Adversarial Smoke Checks

The repository keeps fuzzing optional for local development. The `fuzz/`
package is a small standalone Cargo project with deterministic smoke targets
that exercise parser and render setup paths without requiring network access or
external fuzzing tools.

Run all current smoke targets:

```sh
cargo run --manifest-path fuzz/Cargo.toml --bin primitive_parse -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin xref_load -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin stream_decode -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin content_tokenize -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- --smoke
```

Run one target against saved inputs:

```sh
cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- fixtures/adversarial/truncated-header.pdf
```

The committed adversarial corpus lives in `fixtures/adversarial/`. These files
are intentionally reduced and reviewable; add a minimized input there when a
fuzz run finds a panic, excessive-work case, or unstable error mapping.

Current targets:

| Target | Covered path |
| --- | --- |
| `primitive_parse` | PDF primitive parsing and prefix parsing |
| `xref_load` | indirect object parsing, classic xref loading, modern xref loading |
| `stream_decode` | stream object parsing and bounded filter decoding |
| `content_tokenize` | decoded content stream tokenization and inline-image parsing |
| `render_setup` | native metadata inspection and first-page render setup |

Panics are not caught by the harness. A panic or abort fails the smoke command
and should be minimized into `fixtures/adversarial/` before the code path is
hardened.
