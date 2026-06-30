# 0008: Reference Renderers As Maintainer Oracles

Date: 2026-06-30.
Status: accepted.

## Context

Ferrugo uses mature PDF engines to understand behavior, compare output, and
triage visual drift. PDFium has been the primary comparison engine because it
is permissively licensed and Chrome-adjacent. Poppler, MuPDF, PDF.js, and
Ghostscript are also useful when engines disagree or when a document family is
better represented by another ecosystem.

At the same time, the normal runtime path is intentionally Rust-native and
native-only. Runtime fallback to external PDF renderers has been removed from
the supported path.

## Decision

Treat external PDF renderers as explicit maintainer oracles, not runtime
dependencies.

This means:

- PDFium, Poppler, MuPDF, PDF.js, and Ghostscript may inform behavior,
  conformance analysis, research, and differential test strategy.
- Maintainer comparison tooling may use PDFium behind explicit Cargo features
  and local environment configuration.
- The default CLI and library runtime must not package, download, or silently
  invoke external renderer libraries.
- Active corpus expectations should describe native outcomes such as
  `expected:native`, `expected:native-unsupported`, or typed error/fallback
  buckets, not hidden PDFium fallback behavior.
- External source code must not be copied into Ferrugo. Research findings
  should be recorded as design patterns and validated independently.

## Rationale

The Rust-native renderer needs mature references because PDF rendering is full
of underspecified behavior, malformed real-world files, and cross-engine
differences. Removing all references would make fidelity work weaker.

But a hidden runtime dependency would undermine the project's purpose. Ferrugo
should be able to say clearly what its Rust-native path supports, where it is
typed unsupported, and which comparisons were only maintainer evidence.

## Consequences

Positive:

- Default builds stay native-only and easier to reason about.
- Maintainers still have strong comparison tools for hard visual questions.
- Performance and compatibility claims can distinguish runtime evidence from
  oracle evidence.
- License boundaries stay cleaner because external engines are not vendored or
  copied.

Tradeoffs:

- Some maintainer workflows require local setup of reference renderers.
- PDFium-specific comparison commands remain in the repository until native
  golden and multi-oracle workflows cover the same diagnostic value.
- Public docs must be careful not to imply that reference-renderer results are
  runtime guarantees.

## Follow-Up

- Keep `ferrugo-pdfium` quarantined behind explicit features.
- Keep reference-renderer research in `docs/research/`.
- Promote durable runtime policy into `docs/policies/` and ADRs, not completed
  milestone plans.
