# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/bearcove/marq/compare/v0.0.0...v1.0.0) - 2026-03-03

### Added

- wrap code blocks in .code-block container div
- add LinkResolver trait for custom link resolution
- add configurable language header to ArboriumHandler
- add CompareHandler for side-by-side code comparison blocks
- add InlineCodeHandler for custom inline code rendering
- *(reqs)* add marker_span for precise LSP positioning
- *(reqs)* add text field for plain markdown content

### Fixed

- preserve nested blockquotes inside rule blockquotes ([#11](https://github.com/bearcove/marq/pull/11))
- use explicit mermaid.run() instead of startOnLoad for dynamic injection
- resolve @/ links in list items and other non-paragraph contexts
- *(render)* strip req markers correctly when pulldown-cmark splits them
- *(reqs)* correct marker_span offset for blockquote requirements
- *(highlight)* fall back to plain text for unsupported languages

### Other

- Add release-plz workflow
- Stable pikru, marq v1.0.0
- Add markdown AST and diff API ([#9](https://github.com/bearcove/marq/pull/9))
- Support arbitrary requirement marker prefixes
- Fix RuleId string comparisons for clippy
- Use structured RuleId across requirement parsing
- Fix mermaid live-reload: apply opaque content and re-render
- Add auto dark/light mode for Mermaid.js diagrams
- Add integration tests for head injection collection and deduplication
- Replace mermaid-rs-renderer with client-side Mermaid.js
- Add optional mermaid-rs-renderer
- Migrate captain config to styx
- Upgrade deps
- Make default-langs optional
- Upgrade arborium
- HCL support
- Upgrade arborium
- Show language header by default, add data-lang attribute for CSS targeting
- Add TermHandler for passthrough HTML rendering
- Workaround arborium trailing newline (bearcove/arborium#128)
- Add styx highlighting
- No tracey warnings left
- Update pikru
- Fix img tag rendering
- add integration tests for CompareHandler
- ignore .tracey/
- Remove plain text extraction
- Strip trailing newline from code blocks before rendering
- Add gitignore and improve test coverage for render and requirements
- simpler req.id rendering
- Initial import
