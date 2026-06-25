# Adversarial Regression Corpus

These reduced inputs cover malformed or excessive-work cases used by fuzz smoke
targets and focused regression tests. They are intentionally small so failures
stay easy to minimize and review.

Expected handling:

- `truncated-header.pdf`: native metadata/render setup returns `malformed`.
- `huge-image-dimensions.pdf`: native rendering returns
  `renderer.memory-budget` before allocating declared image samples.
- `deep-primitive-array.input`: primitive parsing hits the nesting budget.
- `unterminated-inline-image.content`: content tokenization returns
  `UnexpectedEof`.
