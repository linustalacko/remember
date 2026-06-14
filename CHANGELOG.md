# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-13

### Added

- Initial public release.
- SM-2 spaced-repetition scheduler with configurable learning steps, graduating/easy
  intervals, and lapse handling.
- Study, browse, stats, and settings views.
- Anki `.anki2` import (decks, notes, cards, scheduling state, and full review history).
- Note types: basic, basic + reverse, vocabulary, and cloze deletion.
- Optional multi-device sync via a user-owned Turso (libSQL) database.
- Configurable daily new-card and review limits.

### Security

- Card/note HTML is sanitized with DOMPurify before rendering, and the webview runs
  under a strict Content-Security-Policy.
- Imported files are validated as SQLite before being opened; the unused `opener`
  plugin was removed; the Turso auth token is stored owner-only and never synced.

[Unreleased]: https://github.com/linustalacko/remember/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/linustalacko/remember/releases/tag/v0.1.0
