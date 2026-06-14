mod commands;
mod db;
mod importer;
mod models;
mod srs;
mod util;

use std::fs;
use tauri::Manager;

use commands::{AppState, SyncConfig};

/// One-shot CLI import used to bring an Anki collection into the app database
/// without the GUI (`remember --import <collection.anki2>`). Writes to the same
/// database the app uses, so the imported decks appear on next launch.
pub fn import_cli(anki_path: &str) {
    let home = std::env::var("HOME").expect("HOME");
    let dir = format!("{home}/Library/Application Support/com.linustalacko.remember");
    std::fs::create_dir_all(&dir).ok();
    let db_path = format!("{dir}/remember.db");

    tauri::async_runtime::block_on(async move {
        let database = db::build_database(&db_path, None)
            .await
            .expect("open database");
        let conn = database.connect().expect("connect");
        db::migrate(&conn).await.expect("migrate");
        match importer::import_anki(&conn, anki_path).await {
            Ok(s) => println!(
                "IMPORTED decks={} notes={} cards={} reviews={}",
                s.decks, s.notes, s.cards, s.reviews
            ),
            Err(e) => {
                eprintln!("IMPORT FAILED: {e}");
                std::process::exit(1);
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("resolve app data dir");
            fs::create_dir_all(&app_dir).ok();
            let db_path = app_dir.join("remember.db");

            // Sync credentials are stored device-locally (never synced).
            let sync_path = app_dir.join("sync.json");
            let sync = fs::read_to_string(&sync_path)
                .ok()
                .and_then(|s| serde_json::from_str::<SyncConfig>(&s).ok())
                .filter(|c| !c.url.trim().is_empty())
                .map(|c| (c.url, c.token));
            let synced = sync.is_some();

            let (db, conn) = tauri::async_runtime::block_on(async move {
                let db = db::build_database(db_path.to_string_lossy().as_ref(), sync)
                    .await
                    .expect("open database");
                if synced {
                    // Pull the latest from Turso before first use; ignore offline failures.
                    let _ = db.sync().await;
                }
                let conn = db.connect().expect("connect to database");
                // Wait out transient locks (e.g. while a sync applies frames).
                let _ = conn.busy_timeout(std::time::Duration::from_millis(5000));
                db::migrate(&conn).await.expect("run migrations");
                (db, conn)
            });

            app.manage(AppState {
                db,
                conn,
                app_dir,
                synced,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_decks,
            commands::create_deck,
            commands::rename_deck,
            commands::delete_deck,
            commands::study_counts,
            commands::next_card,
            commands::answer_card,
            commands::create_note,
            commands::get_note,
            commands::update_note,
            commands::delete_note,
            commands::move_card,
            commands::list_cards,
            commands::stats,
            commands::get_settings,
            commands::set_setting,
            commands::get_sync_status,
            commands::set_sync_config,
            commands::sync_now,
            commands::default_anki_path,
            commands::import_anki,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
