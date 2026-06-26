# Security Fuzz Nightly Loop 2026-06-26

Milestone: 0178

## Summary

Turned the existing fuzz smoke targets into a repeatable local/nightly
maintenance loop. The repository now has one script entry point for the current
target matrix and a security triage policy for crashes, panics, timeouts,
excessive allocation, unsupported input, and malformed input.

No new crash or uncontrolled allocation finding was identified in this slice.
The existing adversarial and malformed regressions remain the active safety
boundary.

## Changes

Added:

- `scripts/check_fuzz_smoke.sh`
- `docs/policies/security-fuzz-triage.md`

Updated:

- `docs/fuzzing.md`
- milestone status and index for 0178

## Fuzz Smoke Matrix

Command:

```sh
bash scripts/check_fuzz_smoke.sh
```

Result:

| Target | Cases | Result |
| --- | ---: | --- |
| `primitive_parse` | 165 | passed |
| `xref_load` | 154 | passed |
| `stream_decode` | 154 | passed |
| `content_tokenize` | 165 | passed |
| `render_setup` | 176 | passed |

## Targeted Regression Checks

Commands:

```sh
cargo test -p pdfrust-syntax excessive_nesting -- --nocapture
cargo test -p pdfrust-content adversarial_unterminated_inline_image -- --nocapture
cargo test -p pdfrust-native adversarial_truncated_header -- --nocapture
cargo test -p pdfrust-native huge_image_dimensions -- --nocapture
cargo test -p pdfrust-render image_resources_should_enforce_declared_image_byte_budget -- --nocapture
cargo test -p pdfrust-render image_resources_should_enforce_image_byte_budget -- --nocapture
```

All targeted checks passed.

## Triage Rules

`docs/policies/security-fuzz-triage.md` defines:

- the active target matrix;
- security, correctness, unsupported-input, and malformed-input classes;
- private crash artifact handling;
- minimization and regression expectations;
- the nightly smoke command set.

Resolved fuzz findings must become a regression test, a minimized adversarial
fixture, a documented unsupported bucket, or a documented malformed boundary.

## Validation

- `bash scripts/check_fuzz_smoke.sh`
- targeted adversarial and memory-budget regression tests
- adversarial corpus classification through existing typed tests
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
