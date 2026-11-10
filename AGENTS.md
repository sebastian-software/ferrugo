# Agent guidance

Use Conventional Commits. This repository owns its formatter, toolchain, lint,
and dependency-policy configuration; explain narrow exceptions beside their settings.
Shared infrastructure guidance lives in [project-infra](https://github.com/sebastian-software/project-infra).

Keep the native Rust formatting, all-target/all-feature Clippy, both workspace
test profiles, distribution checks, and homepage verification in CI.
Run `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
and the applicable native gates from `.github/workflows/ci.yml`.

Edit `README.md.src` and use the pinned mdtheme mise tasks; keep README generation
and its check. The homepage owns its Node configuration and runs `pnpm verify`
from `homepage/`. Do not change dependency exceptions or toolchain support merely
as part of an ownership migration.
