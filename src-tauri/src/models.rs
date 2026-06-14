use serde::{Deserialize, Serialize};

/// A deck (what Anki calls a deck / list).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Deck {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// A deck plus the due/new/learning counts shown on the home screen.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeckWithCounts {
    pub id: String,
    pub name: String,
    pub new_count: i64,
    pub learning_count: i64,
    pub review_count: i64,
    pub total: i64,
}

/// Remaining-today counts for a study session header.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StudyCounts {
    pub new_count: i64,
    pub learning_count: i64,
    pub review_count: i64,
}

/// A single card ready to be studied, with everything the UI needs to render
/// the question, the answer, and the four grading buttons.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StudyCard {
    pub id: String,
    pub note_id: String,
    pub deck_id: String,
    pub deck_name: String,
    pub note_type: String,
    pub template: i64,
    pub fields: serde_json::Value,
    pub tags: String,
    pub state: String,
    pub interval: i64,
    pub ease: i64,
    pub reps: i64,
    pub lapses: i64,
    pub due: i64,
    /// Human labels for the Again / Hard / Good / Easy buttons, e.g. "10m", "1d", "4d".
    pub previews: [String; 4],
}

/// A row in the browse table.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CardRow {
    pub id: String,
    pub note_id: String,
    pub deck_id: String,
    pub deck_name: String,
    pub note_type: String,
    pub template: i64,
    pub fields: serde_json::Value,
    pub tags: String,
    pub state: String,
    pub due: i64,
    pub interval: i64,
    pub reps: i64,
    pub lapses: i64,
}

/// One day's review count, for the stats chart.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DayCount {
    pub day: String,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Stats {
    pub total_cards: i64,
    pub new_cards: i64,
    pub learning_cards: i64,
    pub review_cards: i64,
    pub suspended_cards: i64,
    pub reviews_today: i64,
    pub due_today: i64,
    pub due_tomorrow: i64,
    pub streak_days: i64,
    pub mature_cards: i64,
    pub history: Vec<DayCount>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ImportSummary {
    pub decks: i64,
    pub notes: i64,
    pub cards: i64,
    pub reviews: i64,
}
