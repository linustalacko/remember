<div align="center">

<img src="src-tauri/icons/128x128@2x.png" width="96" height="96" alt="Remember icon" />

# Remember

**A local-first, spaced-repetition flashcard app — an open-source Anki — with optional end-to-end sync.**

[![License: MIT](https://img.shields.io/badge/License-MIT-black.svg)](LICENSE)
[![CI](https://github.com/linustalacko/remember/actions/workflows/ci.yml/badge.svg)](https://github.com/linustalacko/remember/actions/workflows/ci.yml)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB.svg)](https://tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00.svg)](https://svelte.dev)
[![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg)](#install)

</div>

<!-- Drop a screenshot or short GIF of the study screen here, e.g.:
<p align="center"><img src="docs/screenshot.png" width="720" alt="Remember study screen" /></p>
-->

Remember is a small, fast desktop app for studying with spaced repetition. Your
cards live in a local SQLite database — it works fully offline, owns your data,
and can optionally sync across your machines through your own
[Turso](https://turso.tech) database. Import your existing Anki collection and
keep going.

## Features

- **Spaced repetition that works** — a faithful SM-2 scheduler (Anki's classic
  "v2" algorithm) with configurable learning steps, graduating/easy intervals,
  and lapse handling.
- **Import from Anki** — point it at a `collection.anki2` file and it brings over
  your decks, notes, cards, scheduling state, and full review history.
- **Note types** — basic, basic + reverse, vocabulary, and cloze deletion.
- **Study flow** — reveal with `space`, grade with `1`–`4` (Again / Hard / Good /
  Easy); each button shows the next interval before you press it.
- **Browse** — search across card text and filter by deck.
- **Stats** — daily streak, cards reviewed/due today, mature-card count, a 30-day
  review history, and a new/learning/review breakdown of your collection.
- **Daily limits** — cap new cards and reviews per day, per your preference.
- **Optional sync** — multi-device sync via a [Turso](https://turso.tech)
  (libSQL) embedded replica. Your auth token is stored device-locally and is
  **never** part of the synced data.
- **Local-first & private** — no account, no telemetry; SQLite on disk.
- **Cross-platform** — macOS, Windows, and Linux from one Rust + web codebase.

## Install

Pre-built installers are published on the
[Releases](https://github.com/linustalacko/remember/releases) page (`.dmg` for
macOS, `.msi`/`.exe` for Windows, `.deb`/`.AppImage` for Linux).

> macOS note: builds are ad-hoc-signed, so the first launch may need
> **right-click → Open** to get past Gatekeeper.

## Build from source

**Prerequisites**

- [Node.js](https://nodejs.org) 18+ and [pnpm](https://pnpm.io)
- The [Rust toolchain](https://rustup.rs)
- Tauri's platform prerequisites — see the
  [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/)

```bash
git clone https://github.com/linustalacko/remember.git
cd remember
pnpm install

pnpm tauri dev      # run the app in development
pnpm tauri build    # produce a native installer in src-tauri/target/release/bundle
```

You can also import an Anki collection headlessly:

```bash
remember --import /path/to/collection.anki2
```

## Architecture

```
┌────────────────────────────┐       Tauri IPC        ┌────────────────────────────┐
│ SvelteKit frontend (src/)  │ ───── invoke() ──────▶ │  Rust backend (src-tauri/)  │
│ Svelte 5 · adapter-static  │ ◀──── results ───────  │  commands · scheduler · db  │
└────────────────────────────┘                        └─────────────┬──────────────┘
                                                                     │
                                                     libSQL (SQLite) ── optional ──▶ Turso
```

- **`src/`** — the SvelteKit UI (study, browse, stats, settings). Card HTML is
  sanitized with DOMPurify before rendering (see [Security](#security)).
- **`src-tauri/src/commands.rs`** — the Tauri command surface the UI calls.
- **`src-tauri/src/srs.rs`** — the SM-2 spaced-repetition scheduler.
- **`src-tauri/src/db.rs`** — libSQL storage and the optional Turso replica.
- **`src-tauri/src/importer.rs`** — the Anki `.anki2` importer.

For a deeper tour, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Sync (optional)

Remember can sync across devices using **your own** Turso database — there is no
Remember server.

1. Create a database at [turso.tech](https://turso.tech) and copy its URL and an
   auth token.
2. In **Settings → Sync**, paste the URL and token, save, and restart.

The token is written to a device-local file (owner-only on macOS/Linux) and is
never replicated. Only your card data syncs.

## Security

Remember treats card content — especially decks imported from third parties — as
untrusted: all note HTML is sanitized with DOMPurify and the webview runs under a
strict Content-Security-Policy. See [SECURITY.md](SECURITY.md) for the threat
model and how to report a vulnerability.

## Contributing

Contributions are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md). Bug reports
and feature requests go in
[Issues](https://github.com/linustalacko/remember/issues).

## License

[MIT](LICENSE) © Linus Talacko
