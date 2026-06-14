pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Map any error to a String, for returning across the Tauri command boundary.
pub fn es<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

/// Start of the current "Anki day" in epoch ms, using a 4am local rollover so
/// late-night reviews still count toward the previous day (matching Anki).
pub fn day_start_ms(now: i64) -> i64 {
    use chrono::{Datelike, Duration, Local, TimeZone};
    let dt = match Local.timestamp_millis_opt(now).single() {
        Some(d) => d,
        None => return now,
    };
    let shifted = dt - Duration::hours(4);
    match Local
        .with_ymd_and_hms(shifted.year(), shifted.month(), shifted.day(), 4, 0, 0)
        .single()
    {
        Some(m) => m.timestamp_millis(),
        None => now,
    }
}
