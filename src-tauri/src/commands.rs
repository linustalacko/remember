use std::path::PathBuf;

use libsql::{named_params, Connection, Database, Row};
use serde_json::Value;
use tauri::State;

use crate::importer;
use crate::models::*;
use crate::srs::{self, CardSched, SchedConfig};
use crate::util::{day_start_ms, es, new_id, now_ms};

pub struct AppState {
    pub db: Database,
    /// A single shared connection. libSQL serializes statements on one
    /// connection, so reads and writes from concurrent commands never collide
    /// with each other (no more "database is locked").
    pub conn: Connection,
    pub app_dir: PathBuf,
    /// True when the DB was opened as a Turso embedded replica (sync available).
    pub synced: bool,
}

impl AppState {
    fn conn(&self) -> Result<Connection, String> {
        Ok(self.conn.clone())
    }
    pub fn sync_config_path(&self) -> PathBuf {
        self.app_dir.join("sync.json")
    }
}

// ---------- small helpers ----------

async fn scalar_i64(conn: &Connection, sql: &str, params: impl libsql::params::IntoParams) -> i64 {
    match conn.query(sql, params).await {
        Ok(mut rows) => match rows.next().await {
            Ok(Some(r)) => r.get::<i64>(0).unwrap_or(0),
            _ => 0,
        },
        Err(_) => 0,
    }
}

async fn get_setting(conn: &Connection, key: &str) -> Option<String> {
    let mut rows = conn
        .query(
            "SELECT value FROM settings WHERE key = ?1",
            libsql::params![key],
        )
        .await
        .ok()?;
    let row = rows.next().await.ok()??;
    row.get::<String>(0).ok()
}

async fn setting_i64(conn: &Connection, key: &str, default: i64) -> i64 {
    get_setting(conn, key)
        .await
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

const CARD_SELECT: &str = "SELECT c.id, c.note_id, c.deck_id, d.name, n.note_type, c.template, \
    n.fields, n.tags, c.state, c.interval, c.ease, c.reps, c.lapses, c.due \
    FROM cards c JOIN notes n ON n.id = c.note_id JOIN decks d ON d.id = c.deck_id";

fn parse_fields(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|_| Value::Array(vec![]))
}

fn row_to_study_card(row: &Row, now: i64, cfg: &SchedConfig) -> Result<StudyCard, String> {
    let interval: i64 = row.get(9).map_err(es)?;
    let ease: i64 = row.get(10).map_err(es)?;
    let reps: i64 = row.get(11).map_err(es)?;
    let lapses: i64 = row.get(12).map_err(es)?;
    let due: i64 = row.get(13).map_err(es)?;
    let state: String = row.get(8).map_err(es)?;
    let sched = CardSched {
        state: state.clone(),
        step: 0,
        interval,
        ease,
        reps,
        lapses,
        due,
    };
    let previews = srs::previews(&sched, now, cfg);
    Ok(StudyCard {
        id: row.get(0).map_err(es)?,
        note_id: row.get(1).map_err(es)?,
        deck_id: row.get(2).map_err(es)?,
        deck_name: row.get(3).map_err(es)?,
        note_type: row.get(4).map_err(es)?,
        template: row.get(5).map_err(es)?,
        fields: parse_fields(&row.get::<String>(6).map_err(es)?),
        tags: row.get(7).map_err(es)?,
        state,
        interval,
        ease,
        reps,
        lapses,
        due,
        previews,
    })
}

fn row_to_card_row(row: &Row) -> Result<CardRow, String> {
    Ok(CardRow {
        id: row.get(0).map_err(es)?,
        note_id: row.get(1).map_err(es)?,
        deck_id: row.get(2).map_err(es)?,
        deck_name: row.get(3).map_err(es)?,
        note_type: row.get(4).map_err(es)?,
        template: row.get(5).map_err(es)?,
        fields: parse_fields(&row.get::<String>(6).map_err(es)?),
        tags: row.get(7).map_err(es)?,
        state: row.get(8).map_err(es)?,
        interval: row.get(9).map_err(es)?,
        due: row.get(13).map_err(es)?,
        reps: row.get(11).map_err(es)?,
        lapses: row.get(12).map_err(es)?,
    })
}

async fn due_counts(
    conn: &Connection,
    deck: &str,
    now: i64,
    new_per_day: i64,
    rev_per_day: i64,
) -> StudyCounts {
    let day = day_start_ms(now);

    let new_available = scalar_i64(
        conn,
        "SELECT COUNT(*) FROM cards c WHERE c.deleted=0 AND c.state='new' \
         AND (:deck='' OR c.deck_id=:deck)",
        named_params! { ":deck": deck, ":now": now, ":day": day },
    )
    .await;
    let new_done = scalar_i64(
        conn,
        "SELECT COUNT(*) FROM reviews r JOIN cards c ON c.id=r.card_id \
         WHERE r.prev_state='new' AND r.reviewed_at>=:day AND (:deck='' OR c.deck_id=:deck)",
        named_params! { ":deck": deck, ":now": now, ":day": day },
    )
    .await;
    let learning = scalar_i64(
        conn,
        "SELECT COUNT(*) FROM cards c WHERE c.deleted=0 AND c.state IN ('learning','relearning') \
         AND c.due<=:now AND (:deck='' OR c.deck_id=:deck)",
        named_params! { ":deck": deck, ":now": now, ":day": day },
    )
    .await;
    let rev_available = scalar_i64(
        conn,
        "SELECT COUNT(*) FROM cards c WHERE c.deleted=0 AND c.state='review' AND c.due<=:now \
         AND (:deck='' OR c.deck_id=:deck)",
        named_params! { ":deck": deck, ":now": now, ":day": day },
    )
    .await;
    let rev_done = scalar_i64(
        conn,
        "SELECT COUNT(*) FROM reviews r JOIN cards c ON c.id=r.card_id \
         WHERE r.prev_state='review' AND r.reviewed_at>=:day AND (:deck='' OR c.deck_id=:deck)",
        named_params! { ":deck": deck, ":now": now, ":day": day },
    )
    .await;

    StudyCounts {
        new_count: new_available.min((new_per_day - new_done).max(0)),
        learning_count: learning,
        review_count: rev_available.min((rev_per_day - rev_done).max(0)),
    }
}

// ---------- deck commands ----------

#[tauri::command]
pub async fn list_decks(state: State<'_, AppState>) -> Result<Vec<DeckWithCounts>, String> {
    let conn = state.conn()?;
    let now = now_ms();
    let new_per_day = setting_i64(&conn, "new_per_day", 20).await;
    let rev_per_day = setting_i64(&conn, "reviews_per_day", 200).await;

    let mut rows = conn
        .query(
            "SELECT id, name FROM decks WHERE deleted=0 ORDER BY position, created_at",
            (),
        )
        .await
        .map_err(es)?;

    let mut out = Vec::new();
    while let Some(r) = rows.next().await.map_err(es)? {
        let id: String = r.get(0).map_err(es)?;
        let name: String = r.get(1).map_err(es)?;
        let counts = due_counts(&conn, &id, now, new_per_day, rev_per_day).await;
        let total = scalar_i64(
            &conn,
            "SELECT COUNT(*) FROM cards WHERE deck_id=?1 AND deleted=0",
            libsql::params![id.clone()],
        )
        .await;
        out.push(DeckWithCounts {
            id,
            name,
            new_count: counts.new_count,
            learning_count: counts.learning_count,
            review_count: counts.review_count,
            total,
        });
    }
    Ok(out)
}

#[tauri::command]
pub async fn create_deck(state: State<'_, AppState>, name: String) -> Result<String, String> {
    let conn = state.conn()?;
    let now = now_ms();
    let id = new_id();
    conn.execute(
        "INSERT INTO decks (id, name, config, position, created_at, updated_at, deleted) \
         VALUES (?1, ?2, '{}', ?3, ?4, ?4, 0)",
        libsql::params![id.clone(), name, now, now],
    )
    .await
    .map_err(es)?;
    Ok(id)
}

#[tauri::command]
pub async fn rename_deck(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<(), String> {
    let conn = state.conn()?;
    conn.execute(
        "UPDATE decks SET name=?2, updated_at=?3 WHERE id=?1",
        libsql::params![id, name, now_ms()],
    )
    .await
    .map_err(es)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_deck(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.conn()?;
    let now = now_ms();
    conn.execute(
        "UPDATE cards SET deleted=1, updated_at=?2 WHERE deck_id=?1",
        libsql::params![id.clone(), now],
    )
    .await
    .map_err(es)?;
    conn.execute(
        "UPDATE decks SET deleted=1, updated_at=?2 WHERE id=?1",
        libsql::params![id, now],
    )
    .await
    .map_err(es)?;
    Ok(())
}

// ---------- study commands ----------

#[tauri::command]
pub async fn study_counts(
    state: State<'_, AppState>,
    deck_id: Option<String>,
) -> Result<StudyCounts, String> {
    let conn = state.conn()?;
    let now = now_ms();
    let new_per_day = setting_i64(&conn, "new_per_day", 20).await;
    let rev_per_day = setting_i64(&conn, "reviews_per_day", 200).await;
    Ok(due_counts(&conn, &deck_id.unwrap_or_default(), now, new_per_day, rev_per_day).await)
}

#[tauri::command]
pub async fn next_card(
    state: State<'_, AppState>,
    deck_id: Option<String>,
) -> Result<Option<StudyCard>, String> {
    let conn = state.conn()?;
    let now = now_ms();
    let day = day_start_ms(now);
    let deck = deck_id.unwrap_or_default();
    let cfg = SchedConfig::default();
    let new_per_day = setting_i64(&conn, "new_per_day", 20).await;
    let rev_per_day = setting_i64(&conn, "reviews_per_day", 200).await;
    let deck = deck.as_str();

    // 1. Learning / relearning cards that are due now.
    let mut rows = conn
        .query(
            &format!(
                "{CARD_SELECT} WHERE c.deleted=0 AND c.state IN ('learning','relearning') \
                 AND c.due<=:now AND (:deck='' OR c.deck_id=:deck) ORDER BY c.due ASC LIMIT 1"
            ),
            named_params! { ":deck": deck, ":now": now, ":day": day },
        )
        .await
        .map_err(es)?;
    if let Some(r) = rows.next().await.map_err(es)? {
        return Ok(Some(row_to_study_card(&r, now, &cfg)?));
    }

    // 2. Review cards that are due, within the daily review limit.
    let rev_done = scalar_i64(
        &conn,
        "SELECT COUNT(*) FROM reviews r JOIN cards c ON c.id=r.card_id \
         WHERE r.prev_state='review' AND r.reviewed_at>=:day AND (:deck='' OR c.deck_id=:deck)",
        named_params! { ":deck": deck, ":now": now, ":day": day },
    )
    .await;
    if rev_done < rev_per_day {
        let mut rows = conn
            .query(
                &format!(
                    "{CARD_SELECT} WHERE c.deleted=0 AND c.state='review' AND c.due<=:now \
                     AND (:deck='' OR c.deck_id=:deck) ORDER BY c.due ASC LIMIT 1"
                ),
                named_params! { ":deck": deck, ":now": now, ":day": day },
            )
            .await
            .map_err(es)?;
        if let Some(r) = rows.next().await.map_err(es)? {
            return Ok(Some(row_to_study_card(&r, now, &cfg)?));
        }
    }

    // 3. New cards, within the daily new limit.
    let new_done = scalar_i64(
        &conn,
        "SELECT COUNT(*) FROM reviews r JOIN cards c ON c.id=r.card_id \
         WHERE r.prev_state='new' AND r.reviewed_at>=:day AND (:deck='' OR c.deck_id=:deck)",
        named_params! { ":deck": deck, ":now": now, ":day": day },
    )
    .await;
    if new_done < new_per_day {
        let mut rows = conn
            .query(
                &format!(
                    "{CARD_SELECT} WHERE c.deleted=0 AND c.state='new' \
                     AND (:deck='' OR c.deck_id=:deck) ORDER BY c.created_at ASC, c.due ASC LIMIT 1"
                ),
                named_params! { ":deck": deck, ":now": now, ":day": day },
            )
            .await
            .map_err(es)?;
        if let Some(r) = rows.next().await.map_err(es)? {
            return Ok(Some(row_to_study_card(&r, now, &cfg)?));
        }
    }

    Ok(None)
}

#[tauri::command]
pub async fn answer_card(
    state: State<'_, AppState>,
    card_id: String,
    rating: i64,
    duration_ms: Option<i64>,
) -> Result<(), String> {
    let conn = state.conn()?;
    let now = now_ms();
    let cfg = SchedConfig::default();

    let mut rows = conn
        .query(
            "SELECT state, step, interval, ease, reps, lapses, due FROM cards WHERE id=?1 AND deleted=0",
            libsql::params![card_id.clone()],
        )
        .await
        .map_err(es)?;
    let row = rows
        .next()
        .await
        .map_err(es)?
        .ok_or_else(|| "card not found".to_string())?;
    let prev_state: String = row.get(0).map_err(es)?;
    let sched = CardSched {
        state: prev_state.clone(),
        step: row.get(1).map_err(es)?,
        interval: row.get(2).map_err(es)?,
        ease: row.get(3).map_err(es)?,
        reps: row.get(4).map_err(es)?,
        lapses: row.get(5).map_err(es)?,
        due: row.get(6).map_err(es)?,
    };

    let next = srs::answer(&sched, rating, now, &cfg);

    conn.execute(
        "UPDATE cards SET state=?2, step=?3, interval=?4, ease=?5, reps=?6, lapses=?7, due=?8, updated_at=?9 \
         WHERE id=?1",
        libsql::params![
            card_id.clone(),
            next.state.clone(),
            next.step,
            next.interval,
            next.ease,
            next.reps,
            next.lapses,
            next.due,
            now
        ],
    )
    .await
    .map_err(es)?;

    conn.execute(
        "INSERT INTO reviews (id, card_id, rating, prev_state, interval, ease, duration_ms, reviewed_at, deleted) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)",
        libsql::params![
            new_id(),
            card_id,
            rating,
            prev_state,
            next.interval,
            next.ease,
            duration_ms.unwrap_or(0),
            now
        ],
    )
    .await
    .map_err(es)?;

    Ok(())
}

// ---------- note / card editing ----------

fn templates_for(note_type: &str) -> Vec<i64> {
    match note_type {
        "basic_reversed" => vec![0, 1],
        _ => vec![0],
    }
}

#[tauri::command]
pub async fn create_note(
    state: State<'_, AppState>,
    deck_id: String,
    note_type: String,
    fields: Value,
    tags: Option<String>,
) -> Result<String, String> {
    let conn = state.conn()?;
    let now = now_ms();
    let note_id = new_id();
    conn.execute(
        "INSERT INTO notes (id, note_type, fields, tags, created_at, updated_at, deleted) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, 0)",
        libsql::params![
            note_id.clone(),
            note_type.clone(),
            fields.to_string(),
            tags.unwrap_or_default(),
            now
        ],
    )
    .await
    .map_err(es)?;

    for (i, template) in templates_for(&note_type).into_iter().enumerate() {
        conn.execute(
            "INSERT INTO cards \
             (id, note_id, deck_id, template, state, step, due, interval, ease, reps, lapses, created_at, updated_at, deleted) \
             VALUES (?1, ?2, ?3, ?4, 'new', 0, 0, 0, 2500, 0, 0, ?5, ?5, 0)",
            libsql::params![new_id(), note_id.clone(), deck_id.clone(), template, now + i as i64],
        )
        .await
        .map_err(es)?;
    }
    Ok(note_id)
}

#[tauri::command]
pub async fn get_note(state: State<'_, AppState>, note_id: String) -> Result<Value, String> {
    let conn = state.conn()?;
    let mut rows = conn
        .query(
            "SELECT id, note_type, fields, tags FROM notes WHERE id=?1 AND deleted=0",
            libsql::params![note_id],
        )
        .await
        .map_err(es)?;
    let row = rows
        .next()
        .await
        .map_err(es)?
        .ok_or_else(|| "note not found".to_string())?;
    Ok(serde_json::json!({
        "id": row.get::<String>(0).map_err(es)?,
        "note_type": row.get::<String>(1).map_err(es)?,
        "fields": parse_fields(&row.get::<String>(2).map_err(es)?),
        "tags": row.get::<String>(3).map_err(es)?,
    }))
}

#[tauri::command]
pub async fn update_note(
    state: State<'_, AppState>,
    note_id: String,
    fields: Value,
    tags: Option<String>,
) -> Result<(), String> {
    let conn = state.conn()?;
    conn.execute(
        "UPDATE notes SET fields=?2, tags=?3, updated_at=?4 WHERE id=?1",
        libsql::params![
            note_id,
            fields.to_string(),
            tags.unwrap_or_default(),
            now_ms()
        ],
    )
    .await
    .map_err(es)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_note(state: State<'_, AppState>, note_id: String) -> Result<(), String> {
    let conn = state.conn()?;
    let now = now_ms();
    conn.execute(
        "UPDATE cards SET deleted=1, updated_at=?2 WHERE note_id=?1",
        libsql::params![note_id.clone(), now],
    )
    .await
    .map_err(es)?;
    conn.execute(
        "UPDATE notes SET deleted=1, updated_at=?2 WHERE id=?1",
        libsql::params![note_id, now],
    )
    .await
    .map_err(es)?;
    Ok(())
}

#[tauri::command]
pub async fn move_card(
    state: State<'_, AppState>,
    card_id: String,
    deck_id: String,
) -> Result<(), String> {
    let conn = state.conn()?;
    conn.execute(
        "UPDATE cards SET deck_id=?2, updated_at=?3 WHERE id=?1",
        libsql::params![card_id, deck_id, now_ms()],
    )
    .await
    .map_err(es)?;
    Ok(())
}

#[tauri::command]
pub async fn list_cards(
    state: State<'_, AppState>,
    deck_id: Option<String>,
    search: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<CardRow>, String> {
    let conn = state.conn()?;
    let deck = deck_id.unwrap_or_default();
    let search = format!("%{}%", search.unwrap_or_default());
    let limit = limit.unwrap_or(200);
    let offset = offset.unwrap_or(0);

    let sql = format!(
        "{CARD_SELECT} WHERE c.deleted=0 AND (:deck='' OR c.deck_id=:deck) \
         AND (:search='%%' OR n.fields LIKE :search) \
         ORDER BY c.created_at DESC LIMIT :limit OFFSET :offset"
    );
    let mut rows = conn
        .query(
            &sql,
            named_params! {
                ":deck": deck,
                ":search": search,
                ":limit": limit,
                ":offset": offset,
            },
        )
        .await
        .map_err(es)?;
    let mut out = Vec::new();
    while let Some(r) = rows.next().await.map_err(es)? {
        out.push(row_to_card_row(&r)?);
    }
    Ok(out)
}

// ---------- stats ----------

#[tauri::command]
pub async fn stats(state: State<'_, AppState>) -> Result<Stats, String> {
    let conn = state.conn()?;
    let now = now_ms();
    let day = day_start_ms(now);
    let end_today = day + srs::DAY_MS;

    let mut s = Stats::default();
    s.total_cards = scalar_i64(&conn, "SELECT COUNT(*) FROM cards WHERE deleted=0", ()).await;
    s.new_cards = scalar_i64(
        &conn,
        "SELECT COUNT(*) FROM cards WHERE deleted=0 AND state='new'",
        (),
    )
    .await;
    s.learning_cards = scalar_i64(
        &conn,
        "SELECT COUNT(*) FROM cards WHERE deleted=0 AND state IN ('learning','relearning')",
        (),
    )
    .await;
    s.review_cards = scalar_i64(
        &conn,
        "SELECT COUNT(*) FROM cards WHERE deleted=0 AND state='review'",
        (),
    )
    .await;
    s.suspended_cards = scalar_i64(
        &conn,
        "SELECT COUNT(*) FROM cards WHERE deleted=0 AND state='suspended'",
        (),
    )
    .await;
    s.mature_cards = scalar_i64(
        &conn,
        "SELECT COUNT(*) FROM cards WHERE deleted=0 AND state='review' AND interval>=21",
        (),
    )
    .await;
    s.reviews_today = scalar_i64(
        &conn,
        "SELECT COUNT(*) FROM reviews WHERE reviewed_at>=?1",
        libsql::params![day],
    )
    .await;
    s.due_today = scalar_i64(
        &conn,
        "SELECT COUNT(*) FROM cards WHERE deleted=0 AND state IN ('review','learning','relearning') AND due<?1",
        libsql::params![end_today],
    )
    .await;
    s.due_tomorrow = scalar_i64(
        &conn,
        "SELECT COUNT(*) FROM cards WHERE deleted=0 AND state IN ('review','learning','relearning') AND due>=?1 AND due<?2",
        libsql::params![end_today, end_today + srs::DAY_MS],
    )
    .await;

    // 30-day review history, bucketed by local day.
    let since = day - 29 * srs::DAY_MS;
    let mut buckets: std::collections::BTreeMap<i64, i64> = std::collections::BTreeMap::new();
    for i in 0..30 {
        buckets.insert(since + i * srs::DAY_MS, 0);
    }
    let mut rows = conn
        .query(
            "SELECT reviewed_at FROM reviews WHERE reviewed_at>=?1",
            libsql::params![since],
        )
        .await
        .map_err(es)?;
    while let Some(r) = rows.next().await.map_err(es)? {
        let at: i64 = r.get(0).map_err(es)?;
        let bucket = day_start_ms(at);
        *buckets.entry(bucket).or_insert(0) += 1;
    }
    s.history = buckets
        .into_iter()
        .map(|(k, v)| {
            let label = chrono::DateTime::from_timestamp_millis(k)
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            DayCount { day: label, count: v }
        })
        .collect();

    // Streak: consecutive days (ending today or yesterday) with >=1 review.
    let mut streak = 0i64;
    let mut d = day;
    loop {
        let c = scalar_i64(
            &conn,
            "SELECT COUNT(*) FROM reviews WHERE reviewed_at>=?1 AND reviewed_at<?2",
            libsql::params![d, d + srs::DAY_MS],
        )
        .await;
        if c > 0 {
            streak += 1;
            d -= srs::DAY_MS;
        } else if d == day {
            // No reviews yet today — keep the streak alive if yesterday had reviews.
            d -= srs::DAY_MS;
        } else {
            break;
        }
    }
    s.streak_days = streak;

    Ok(s)
}

// ---------- settings ----------

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Value, String> {
    let conn = state.conn()?;
    Ok(serde_json::json!({
        "new_per_day": setting_i64(&conn, "new_per_day", 20).await,
        "reviews_per_day": setting_i64(&conn, "reviews_per_day", 200).await,
    }))
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.conn()?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value=?2",
        libsql::params![key, value],
    )
    .await
    .map_err(es)?;
    Ok(())
}

// ---------- sync ----------

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SyncConfig {
    pub url: String,
    pub token: String,
}

#[tauri::command]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<Value, String> {
    let path = state.sync_config_path();
    let configured = path.exists();
    let url = if configured {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<SyncConfig>(&s).ok())
            .map(|c| c.url)
            .unwrap_or_default()
    } else {
        String::new()
    };
    Ok(serde_json::json!({ "configured": configured, "url": url }))
}

#[tauri::command]
pub async fn set_sync_config(
    state: State<'_, AppState>,
    url: String,
    token: String,
) -> Result<(), String> {
    let path = state.sync_config_path();
    if url.trim().is_empty() {
        let _ = std::fs::remove_file(&path);
        return Ok(());
    }
    // Only accept libSQL/HTTP(S) endpoints so a stray value can't point the
    // embedded replica somewhere unexpected.
    let u = url.trim();
    if !(u.starts_with("libsql://") || u.starts_with("https://") || u.starts_with("http://")) {
        return Err("Sync URL must start with libsql:// or https://".into());
    }
    let cfg = SyncConfig { url, token };
    std::fs::write(&path, serde_json::to_string(&cfg).map_err(es)?).map_err(es)?;
    // The auth token is a bearer credential; keep the file readable only by its
    // owner on Unix so other local accounts can't lift it.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

#[tauri::command]
pub async fn sync_now(state: State<'_, AppState>) -> Result<String, String> {
    if !state.synced {
        return Err("Sync is not configured. Add your Turso URL and token in Settings, then restart.".into());
    }
    match state.db.sync().await {
        Ok(_) => Ok("Synced".into()),
        Err(e) => Err(format!("Sync failed: {e}")),
    }
}

// ---------- import ----------

#[tauri::command]
pub async fn default_anki_path() -> Result<Option<String>, String> {
    let home = std::env::var("HOME").map_err(es)?;
    let base = PathBuf::from(home).join("Library/Application Support/Anki2");
    if !base.exists() {
        return Ok(None);
    }
    // Prefer a profile directory that actually contains a collection.
    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            let candidate = entry.path().join("collection.anki2");
            if candidate.exists() {
                return Ok(Some(candidate.to_string_lossy().to_string()));
            }
        }
    }
    Ok(None)
}

/// Guard the import path before we open it: a real, reasonably-sized SQLite file.
/// Anki collections are SQLite databases, so anything that isn't one is rejected
/// up front (defense-in-depth against a crafted path or a non-collection file).
fn validate_anki_file(path: &str) -> Result<(), String> {
    let meta = std::fs::metadata(path).map_err(|_| "File not found.".to_string())?;
    if !meta.is_file() {
        return Err("Not a regular file.".into());
    }
    const MAX_BYTES: u64 = 4 * 1024 * 1024 * 1024; // 4 GiB ceiling
    if meta.len() > MAX_BYTES {
        return Err("File is too large to import (over 4 GB).".into());
    }
    // Every SQLite database begins with the 16-byte string "SQLite format 3\0".
    use std::io::Read;
    let mut header = [0u8; 16];
    std::fs::File::open(path)
        .map_err(es)?
        .read_exact(&mut header)
        .map_err(|_| "That doesn't look like an Anki collection.".to_string())?;
    if &header != b"SQLite format 3\0" {
        return Err("That doesn't look like an Anki collection (.anki2 SQLite file).".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn import_anki(
    state: State<'_, AppState>,
    path: String,
) -> Result<ImportSummary, String> {
    validate_anki_file(&path)?;
    let conn = state.conn()?;
    importer::import_anki(&conn, &path).await
}
