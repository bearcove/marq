# marq

## Moved to Dodeca

This project has been absorbed into [Dodeca](https://github.com/bearcove/dodeca).
Future development happens in Dodeca's `libs/marq` tree; this repository is kept
for history and crate metadata.

Markdown processing (based on pulldown-cmark), recognizes
[tracey](https://github.com/bearcove/tracey) rules, uses
[arborium](https://github.com/bearcove/arborium) for syntax highlighting,
[aasvg-rs](https://github.com/bearcove/aasvg-rs) and
[pikru](https://github.com/bearcove/pikru) for diagrams.

Supports wiki-style links (`[[Target]]`, `[[Target|label]]`) through a
`WikiLinkResolver`, so applications can resolve them against their own page
index while marq handles the Markdown syntax.

Used by [dodeca](https://github.com/bearcove/dodeca) and
[tracey](https://github.com/bearcove/tracey)
