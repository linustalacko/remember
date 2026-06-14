# Architecture

Remember is a [Tauri 2](https://tauri.app) desktop app: a SvelteKit front end
running in the system webview, talking to a Rust backend over Tauri's IPC. All
state lives in a local [libSQL](https://github.com/tursodatabase/libsql) (SQLite)
database that can optionally sync to a user-owned [Turso](https://turso.tech)
database.

```
┌────────────────────────────┐       Tauri IPC        ┌────────────────────────────┐
│ SvelteKit frontend (src/)  │ ───── invoke() ──────▶ │  Rust backend (src-tauri/)  │
│ Svelte 5 · adapter-static  │ ◀──── results ───────  │  commands · scheduler · db  │
└────────────────────────────┘                        └─────────────┬──────────────┘
                                                                     │
                                                     libSQL (SQLite) ── optional ──▶ Turso
```

## Frontend (`src/`)

A static SvelteKit SPA (`@sveltejs/adapter-static`, `fallback: index.html`) — no
SSR, no Node server at runtime. Svelte 5 runes (`$state`, `$derived`, `$effect`)
provide reactivity.

- `src/routes/` — the pages: home (`+page.svelte`), `study/`, `browse/`, `stats/`,
  `settings/`.
- `src/lib/api.ts` — the single typed wrapper around Tauri's `invoke()`. Every
  backend call goes through here.
- `src/lib/render.ts` — renders note fields to HTML and **sanitizes every string
  with DOMPurify** before it reaches `{@html}`. This is the one choke point for
  untrusted card content.
- `src/lib/app.svelte.ts` — small shared UI state (toasts, the editor modal, a
  data-version counter that components watch to refetch).

## Backend (`src-tauri/src/`)

| File | Responsibility |
|------|----------------|
| `lib.rs` | App setup: open the DB, run migrations, register commands. Also the headless `--import` entry point. |
| `commands.rs` | The IPC surface — every `#[tauri::command]` the UI can call. |
| `db.rs` | libSQL connection, schema/migrations, and the optional Turso embedded replica. |
| `srs.rs` | The SM-2 spaced-repetition scheduler. |
| `importer.rs` | Reads an Anki `.anki2` SQLite collection and maps it into Remember's schema. |
| `models.rs` | The Rust ⇄ TypeScript data contracts (serde). |
| `util.rs` | IDs, timestamps, and the 4am day-rollover boundary. |

## Data model

Four core tables (plus a `settings` key/value table). Rows are soft-deleted via a
`deleted` tombstone so deletions can sync.

- **decks** — `id`, `name`, `position`.
- **notes** — `id`, `note_type`, `fields` (JSON array of `{name, value}`), `tags`.
  A note is the content; cards are generated from it.
- **cards** — `id`, `note_id`, `deck_id`, `template`, and the scheduling state:
  `state` (`new` / `learning` / `review` / `relearning` / `suspended`), `step`,
  `due` (epoch ms), `interval` (days), `ease` (per-mille), `reps`, `lapses`.
- **reviews** — an append-only log: `card_id`, `rating` (1–4), `prev_state`,
  resulting `interval`/`ease`, `duration_ms`, `reviewed_at`.

## Scheduling (SM-2)

`srs.rs` implements Anki's classic "v2" SM-2 algorithm. Grades are **Again (1),
Hard (2), Good (3), Easy (4)**. Cards move through learning steps (default
`[1m, 10m]`) before graduating to the review queue; review intervals scale by the
card's ease factor (default 2.5×, floored at 1.3×); lapses apply a penalty and
send the card to relearning. `SchedConfig` holds all the tunables, and
`previews()` precomputes the next-interval label shown on each grading button.

The "day" boundary is local 4am (`util::day_start_ms`), matching Anki, so late
-night reviews count toward the right day.

## Sync (optional)

`db.rs` can open the local database as a Turso **embedded replica**: reads/writes
hit the local SQLite file, and changes sync to the remote in the background. Sync
is entirely opt-in and uses the user's **own** Turso database — there is no
Remember server. The auth token is stored in a device-local `sync.json`
(owner-only) and is never written into synced tables or returned to the frontend.

## Import

`importer.rs` opens an Anki `.anki2` (which is just SQLite), validates it, and
maps Anki's `notes`/`cards`/`revlog`/notetypes into Remember's schema inside a
single transaction (rolled back on any error). Anki's day-relative due dates are
converted to absolute timestamps using the collection's creation time, and review
history is preserved.

## Build & packaging

`pnpm tauri dev` runs SvelteKit's dev server and the Rust app together with hot
reload. `pnpm tauri build` builds the static frontend, embeds it, and produces
native installers under `src-tauri/target/release/bundle/`. CI builds the front
end and Rust on every PR; tagged `v*` pushes build installers for macOS, Windows,
and Linux via `tauri-action` (see `.github/workflows/`).
