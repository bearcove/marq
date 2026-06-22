# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [5.0.0-rc.1](https://github.com/bearcove/marq/compare/v5.0.0-rc.0...v5.0.0-rc.1) - 2026-06-22

### Other

- add resolved flag to NoteMeta (data-resolved on aside)
- add id + created to NoteMeta; link mark<->note via data-note-id
- Add note highlight marks (<dodeca-mark>), stripped in production
- Fix to_comment dropping metadata when a field is None
- Add marq::to_comment to serialize notes (round-trips with parse_note)
- Add inline note rendering (<!-- note --> comments)
- Point readers to Dodeca

## [4.0.1](https://github.com/bearcove/marq/compare/v4.0.0...v4.0.1) - 2026-05-25

### Fixed

- Build the `pikru` feature with `pikru` 1.2.1.

## [4.0.0](https://github.com/bearcove/marq/compare/v3.0.0...v4.0.0) - 2026-05-25

### Other

- Add wiki-style link support

## [3.0.0](https://github.com/bearcove/marq/compare/v2.2.2...v3.0.0) - 2026-05-25

### Other

- Add opt-in source maps for rendered elements ([#15](https://github.com/bearcove/marq/pull/15))

## [2.2.1](https://github.com/bearcove/marq/compare/v2.2.0...v2.2.1) - 2026-05-19

### Other

- Upgrade facet 0.44 → 0.46 (source-compatible, builds clean)
