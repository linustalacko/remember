//! Import an existing Anki collection (`collection.anki2`) into our database.
//!
//! We read the Anki SQLite file directly and translate its decks, notes, cards
//! (with their SM-2 scheduling state) and full review history into our schema.
//! Field order is preserved and HTML content is kept verbatim, so vocabulary
//! definitions and formatting survive the trip.

use std::collections::HashMap;

use libsql::{Builder, Connection};

use crate::models::ImportSummary;
use crate::util::{es, new_id, now_ms};

fn classify(name: &str, field_names: &[String]) -> String {
    let lower = name.to_lowercase();
    if lower.contains("cloze") {
        return "cloze".into();
    }
    let has = |f: &str| field_names.iter().any(|n| n.eq_ignore_ascii_case(f));
    if has("Word") && has("Definition") {
        return "vocab".into();
    }
    if lower.contains("reversed") {
        return "basic_reversed".into();
    }
    "basic".into()
}

pub async fn import_anki(target: &Connection, anki_path: &str) -> Result<ImportSummary, String> {
    let adb = Builder::new_local(anki_path.to_string())
        .build()
        .await
        .map_err(es)?;
    let a = adb.connect().map_err(es)?;
    let now = now_ms();

    // Collection creation time (seconds) — used to convert Anki's day-based due
    // values into absolute timestamps.
    let crt: i64 = {
        let mut rows = a.query("SELECT crt FROM col LIMIT 1", ()).await.map_err(es)?;
        match rows.next().await.map_err(es)? {
            Some(r) => r.get(0).map_err(es)?,
            None => now / 1000,
        }
    };
    let crt_ms = crt * 1000;

    // Field names per notetype, ordered by ord.
    let mut field_names: HashMap<i64, Vec<(i64, String)>> = HashMap::new();
    {
        let mut rows = a
            .query("SELECT ntid, ord, name FROM fields", ())
            .await
            .map_err(es)?;
        while let Some(r) = rows.next().await.map_err(es)? {
            let ntid: i64 = r.get(0).map_err(es)?;
            let ord: i64 = r.get(1).map_err(es)?;
            let name: String = r.get(2).map_err(es)?;
            field_names.entry(ntid).or_default().push((ord, name));
        }
    }
    let field_names: HashMap<i64, Vec<String>> = field_names
        .into_iter()
        .map(|(k, mut v)| {
            v.sort_by_key(|(ord, _)| *ord);
            (k, v.into_iter().map(|(_, n)| n).collect())
        })
        .collect();

    // Notetype names.
    let mut notetype_names: HashMap<i64, String> = HashMap::new();
    {
        let mut rows = a.query("SELECT id, name FROM notetypes", ()).await.map_err(es)?;
        while let Some(r) = rows.next().await.map_err(es)? {
            let id: i64 = r.get(0).map_err(es)?;
            let name: String = r.get(1).map_err(es)?;
            notetype_names.insert(id, name);
        }
    }

    target.execute("BEGIN", ()).await.map_err(es)?;
    let result = import_inner(
        target,
        &a,
        crt_ms,
        now,
        &field_names,
        &notetype_names,
    )
    .await;
    match result {
        Ok(summary) => {
            target.execute("COMMIT", ()).await.map_err(es)?;
            Ok(summary)
        }
        Err(e) => {
            let _ = target.execute("ROLLBACK", ()).await;
            Err(e)
        }
    }
}

async fn import_inner(
    target: &Connection,
    a: &Connection,
    crt_ms: i64,
    now: i64,
    field_names: &HashMap<i64, Vec<String>>,
    notetype_names: &HashMap<i64, String>,
) -> Result<ImportSummary, String> {
    let mut summary = ImportSummary::default();

    // Decks.
    let mut deck_map: HashMap<i64, String> = HashMap::new();
    {
        let mut rows = a.query("SELECT id, name FROM decks", ()).await.map_err(es)?;
        let mut pos = 0i64;
        while let Some(r) = rows.next().await.map_err(es)? {
            let did: i64 = r.get(0).map_err(es)?;
            let name: String = r.get(1).map_err(es)?;
            let id = new_id();
            target
                .execute(
                    "INSERT INTO decks (id, name, config, position, created_at, updated_at, deleted) \
                     VALUES (?1, ?2, '{}', ?3, ?4, ?4, 0)",
                    libsql::params![id.clone(), name, pos, now],
                )
                .await
                .map_err(es)?;
            deck_map.insert(did, id);
            pos += 1;
            summary.decks += 1;
        }
    }

    // Notes.
    let mut note_type_for_note: HashMap<i64, String> = HashMap::new();
    let mut note_map: HashMap<i64, String> = HashMap::new();
    {
        let mut rows = a
            .query("SELECT id, mid, tags, flds FROM notes", ())
            .await
            .map_err(es)?;
        while let Some(r) = rows.next().await.map_err(es)? {
            let nid: i64 = r.get(0).map_err(es)?;
            let mid: i64 = r.get(1).map_err(es)?;
            let tags: String = r.get(2).map_err(es)?;
            let flds: String = r.get(3).map_err(es)?;

            let names = field_names.get(&mid);
            let empty: Vec<String> = Vec::new();
            let names = names.unwrap_or(&empty);
            let parts: Vec<&str> = flds.split('\u{1f}').collect();
            let mut arr = Vec::new();
            for (i, val) in parts.iter().enumerate() {
                let name = names
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("Field {}", i + 1));
                arr.push(serde_json::json!({ "name": name, "value": val }));
            }
            let fields_json = serde_json::Value::Array(arr).to_string();

            let nt_name = notetype_names.get(&mid).cloned().unwrap_or_default();
            let note_type = classify(&nt_name, names);
            note_type_for_note.insert(nid, note_type.clone());

            let id = new_id();
            target
                .execute(
                    "INSERT INTO notes (id, note_type, fields, tags, created_at, updated_at, deleted) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?5, 0)",
                    libsql::params![id.clone(), note_type, fields_json, tags.trim(), now],
                )
                .await
                .map_err(es)?;
            note_map.insert(nid, id);
            summary.notes += 1;
        }
    }

    // Cards.
    let mut card_map: HashMap<i64, String> = HashMap::new();
    {
        let mut rows = a
            .query(
                "SELECT id, nid, did, ord, type, queue, due, ivl, factor, reps, lapses FROM cards",
                (),
            )
            .await
            .map_err(es)?;
        while let Some(r) = rows.next().await.map_err(es)? {
            let cid: i64 = r.get(0).map_err(es)?;
            let nid: i64 = r.get(1).map_err(es)?;
            let did: i64 = r.get(2).map_err(es)?;
            let ord: i64 = r.get(3).map_err(es)?;
            let ctype: i64 = r.get(4).map_err(es)?;
            let queue: i64 = r.get(5).map_err(es)?;
            let due: i64 = r.get(6).map_err(es)?;
            let ivl: i64 = r.get(7).map_err(es)?;
            let factor: i64 = r.get(8).map_err(es)?;
            let reps: i64 = r.get(9).map_err(es)?;
            let lapses: i64 = r.get(10).map_err(es)?;

            let (Some(note_id), Some(deck_id)) = (note_map.get(&nid), deck_map.get(&did)) else {
                continue;
            };

            let state: &str = match queue {
                -1 => "suspended",
                2 => "review",
                1 | 3 => {
                    if ctype == 3 {
                        "relearning"
                    } else {
                        "learning"
                    }
                }
                -2 | -3 => match ctype {
                    2 => "review",
                    1 => "learning",
                    3 => "relearning",
                    _ => "new",
                },
                _ => "new",
            };

            // `due`, `ivl` and `factor` come from an untrusted imported file, so
            // use saturating arithmetic to avoid overflow panics (debug) / wraps
            // (release) on absurd values.
            let due_days_ms = || crt_ms.saturating_add(due.saturating_mul(crate::srs::DAY_MS));
            let due_ms = match state {
                "review" | "suspended" => due_days_ms(),
                "learning" | "relearning" => {
                    if queue == 1 {
                        due.saturating_mul(1000)
                    } else {
                        due_days_ms()
                    }
                }
                _ => 0,
            };
            let interval = if ivl > 0 { ivl } else { 0 };
            let ease = if factor > 0 { factor } else { 2500 };
            // Preserve new-card ordering using Anki's queue position.
            let created_at = if state == "new" {
                crt_ms.saturating_add(due.max(0))
            } else {
                crt_ms
            };

            let id = new_id();
            target
                .execute(
                    "INSERT INTO cards \
                     (id, note_id, deck_id, template, state, step, due, interval, ease, reps, lapses, created_at, updated_at, deleted) \
                     VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0)",
                    libsql::params![
                        id.clone(),
                        note_id.clone(),
                        deck_id.clone(),
                        ord,
                        state,
                        due_ms,
                        interval,
                        ease,
                        reps,
                        lapses,
                        created_at,
                        now
                    ],
                )
                .await
                .map_err(es)?;
            card_map.insert(cid, id);
            summary.cards += 1;
        }
    }

    // Review history.
    {
        let mut rows = a
            .query("SELECT id, cid, ease, ivl, factor, time FROM revlog", ())
            .await
            .map_err(es)?;
        while let Some(r) = rows.next().await.map_err(es)? {
            let rid: i64 = r.get(0).map_err(es)?;
            let cid: i64 = r.get(1).map_err(es)?;
            let ease: i64 = r.get(2).map_err(es)?;
            let ivl: i64 = r.get(3).map_err(es)?;
            let factor: i64 = r.get(4).map_err(es)?;
            let time: i64 = r.get(5).map_err(es)?;

            let Some(card_id) = card_map.get(&cid) else {
                continue;
            };
            let rating = ease.clamp(1, 4);
            target
                .execute(
                    "INSERT INTO reviews \
                     (id, card_id, rating, prev_state, interval, ease, duration_ms, reviewed_at, deleted) \
                     VALUES (?1, ?2, ?3, 'review', ?4, ?5, ?6, ?7, 0)",
                    libsql::params![
                        new_id(),
                        card_id.clone(),
                        rating,
                        ivl.max(0),
                        factor,
                        time,
                        rid
                    ],
                )
                .await
                .map_err(es)?;
            summary.reviews += 1;
        }
    }

    Ok(summary)
}
