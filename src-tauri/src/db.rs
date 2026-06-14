//! Database layer: libSQL connection + schema.
//!
//! Every device keeps a full local copy of the database (so the app works fully
//! offline and instantly), and — once Turso sync credentials are configured —
//! that local copy is an *embedded replica* that syncs in the background. Every
//! row carries a stable text id, `created_at`/`updated_at` timestamps, and a
//! `deleted` tombstone flag so single-user multi-device sync stays conflict-free.

use libsql::{Builder, Connection, Database};

pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS decks (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    config      TEXT NOT NULL DEFAULT '{}',
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    deleted     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS notes (
    id          TEXT PRIMARY KEY,
    note_type   TEXT NOT NULL,
    fields      TEXT NOT NULL,           -- JSON object: { fieldName: value }
    tags        TEXT NOT NULL DEFAULT '',
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    deleted     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS cards (
    id          TEXT PRIMARY KEY,
    note_id     TEXT NOT NULL,
    deck_id     TEXT NOT NULL,
    template    INTEGER NOT NULL DEFAULT 0,   -- which generated card (0 = front, 1 = reverse, ...)
    state       TEXT NOT NULL DEFAULT 'new',  -- new | learning | review | relearning | suspended
    step        INTEGER NOT NULL DEFAULT 0,
    due         INTEGER NOT NULL DEFAULT 0,    -- epoch ms when next due (new cards: queue position)
    interval    INTEGER NOT NULL DEFAULT 0,    -- days
    ease        INTEGER NOT NULL DEFAULT 2500, -- per-mille
    reps        INTEGER NOT NULL DEFAULT 0,
    lapses      INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    deleted     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS reviews (
    id          TEXT PRIMARY KEY,
    card_id     TEXT NOT NULL,
    rating      INTEGER NOT NULL,             -- 1 again .. 4 easy
    prev_state  TEXT NOT NULL,                -- card state before this answer
    interval    INTEGER NOT NULL DEFAULT 0,
    ease        INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    reviewed_at INTEGER NOT NULL,
    deleted     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cards_deck   ON cards(deck_id, state, due);
CREATE INDEX IF NOT EXISTS idx_cards_note   ON cards(note_id);
CREATE INDEX IF NOT EXISTS idx_reviews_when ON reviews(reviewed_at);
CREATE INDEX IF NOT EXISTS idx_reviews_card ON reviews(card_id);
"#;

/// Build the database. When `sync` is `Some((url, token))` we open a Turso
/// embedded replica; otherwise a purely local file.
pub async fn build_database(
    local_path: &str,
    sync: Option<(String, String)>,
) -> Result<Database, libsql::Error> {
    match sync {
        Some((url, token)) => {
            Builder::new_remote_replica(local_path.to_string(), url, token)
                .build()
                .await
        }
        None => Builder::new_local(local_path.to_string()).build().await,
    }
}

pub async fn migrate(conn: &Connection) -> Result<(), libsql::Error> {
    conn.execute_batch(SCHEMA).await?;
    Ok(())
}
