# README theme composition

## Status

Active. Update this document when the contract changes.

## Decision

The project README is composed by native mdtheme from `README.md.src`. The
outer frame is Sebastian Software; Ferramenta is the inner frame. Footers close
in reverse order. The project content remains the main focus; shared branding
is compact and maintained upstream. The Ferramenta footer excludes this project
and includes sibling descriptions and the family icon.

Pin the CLI with mise and both Git theme revisions in mdtheme.yaml. CI checks
the generated output. Source and output are committed together. The pinned
mdtheme generator owns the complete README, including the company footer.
Published subpackage READMEs keep their compact registry format and existing
generator.

## Consequences

Contributors edit the source, then regenerate. No JavaScript configuration or
Node installation is needed for the root README. Shared theme updates are
reviewable Git diffs. Rendering requires network access to the Git sources.

See [the contributor guide](../readme-theme.md) for commands.

## Repository tooling

The repository owns its formatter, lint, toolchain, and dependency-policy
configuration. Shared infrastructure guidance lives in
[project-infra](https://github.com/sebastian-software/project-infra).
The repository-tooling workflow retains the native README, Rust formatting,
Clippy, and dependency-policy checks; the existing regression workflow remains.
The metadata stamp and retired CLI drift check no longer participate.

The dependency-policy baseline records RUSTSEC-2026-0192 as a narrow exception:
`ttf-parser` is unmaintained, has no patched release, and is used by the current
renderer. Remove the exception when that parser is replaced. Other advisories
remain blocking; the license allow-list is unchanged.

## Theme badge placement

The project pins mdtheme 0.4.0 and uses one `mdtheme:badges:start` /
`mdtheme:badges:end` comment pair in `README.md.src`. Sebastian's
`badges-prepend.md` places its badge before the authored project badges.
Ferramenta's header and footer stay unchanged. Upgrade the CLI and lockfile
before adopting this theme revision. Keep badge markup outside raw HTML blocks.

## Change history

- 2026-11-10: Transfer tooling ownership to this repository while retaining native checks and pinned README generation.
