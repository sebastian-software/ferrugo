# 0003: Ferrugo Project Name

Date: 2026-06-24.
Status: accepted.

## Context

The repository started under the working name `pdfrust` while the project was
still validating its first product slice: reliable PDF thumbnail generation
backed by PDFium and, over time, a Rust-native PDF engine.

That working name is descriptive, but it is also narrow and generic. It ties
the project closely to Rust and PDF implementation mechanics, while the intended
library surface may eventually cover more than thumbnail rendering:

- PDF parsing,
- page and document inspection,
- high-quality rendering,
- thumbnail generation,
- and potentially PDF creation.

The project therefore needs a public name that can carry a broader PDF library
and engine over time. The name should be internationally pronounceable, usable
for both Rust and Node.js packages, and compatible with the existing `ferro` /
`ferri` naming family used for related Rust projects.

The `ferrugo` name was checked and successfully claimed on npm as a public
package name before this decision was recorded.

## Decision

Use **Ferrugo** as the public project and package name.

The exact timing of repository, crate, module, and package renames is deferred.
This decision records the naming direction only; it does not require immediate
changes to existing Rust crate names, code paths, documentation links, or GitHub
repository metadata while parallel implementation work is in progress.

## Rationale

Ferrugo is a good fit for this project because it connects several useful ideas
without naming a single implementation detail:

- It starts with `ferr`, matching the established `ferro` / `ferri` family.
- It is derived from Latin usage around iron rust and iron-colored patina,
  giving it a direct but understated Rust association.
- It is not limited to rendering, thumbnails, parsing, or writing, so it can
  still fit if the project grows into a broader PDF engine.
- It is short enough for package names and crate prefixes.
- It is pronounceable in English and German without unusual punctuation,
  casing, or diacritics.
- It is distinctive enough to work as a project name rather than only a
  descriptive implementation label.

Compared with more literal names such as `pdfrust`, Ferrugo gives the project a
durable identity while still preserving a quiet technical hint: iron, rust, and
the Rust ecosystem.

Compared with document-specific names such as `charta` or `scribe` variants,
Ferrugo is less tied to PDF creation and therefore remains suitable for parsing,
rendering, validation, and future generation APIs.

Compared with Greek iron-derived names such as `sidera` or `sideron`, Ferrugo
fits the existing project-family prefix better and was available for the npm
package path that the future Node.js API is expected to need.

## Consequences

Positive:

- The project has a stable public name before API and package surfaces are
  finalized.
- The name can cover both Rust-native internals and a future Node.js package.
- The name avoids overcommitting the project to thumbnails or rendering only.
- The npm package name is already claimed, reducing the risk of losing the
  intended JavaScript distribution name.

Tradeoffs:

- The name does not directly say "PDF", so package descriptions and README
  copy must make the domain clear.
- The current repository and crate names still use `pdfrust` until a separate
  rename plan is accepted.
- A future rename should be handled deliberately to avoid disrupting ongoing
  work, docs, crate paths, and package metadata in one uncontrolled change.

## Follow-Up

When parallel implementation work has settled, create a separate rename plan for
the concrete surfaces that should move from `pdfrust` to `ferrugo`, including:

- Rust crate names and crate prefixes,
- Node.js package structure,
- README and documentation references,
- GitHub repository metadata,
- CI and release configuration,
- and migration notes for early users.
