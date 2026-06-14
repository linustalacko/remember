//! Spaced-repetition scheduler.
//!
//! This is a faithful implementation of the classic Anki SM-2 scheduler (the
//! "v2" algorithm): learning steps, graduating/easy intervals, ease factors,
//! lapses with relearning steps, hard/good/easy multipliers. It maps directly
//! onto the `interval` (days) and `ease` (per-mille) state that we import from
//! the user's existing Anki collection, so no review progress is lost.

pub const DAY_MS: i64 = 86_400_000;
pub const MIN_MS: i64 = 60_000;

pub const AGAIN: i64 = 1;
pub const HARD: i64 = 2;
pub const GOOD: i64 = 3;
pub const EASY: i64 = 4;

#[derive(Clone, Debug)]
pub struct SchedConfig {
    /// Learning steps in minutes for brand-new cards, e.g. [1, 10].
    pub learning_steps_min: Vec<i64>,
    /// Relearning steps in minutes after a lapse, e.g. [10].
    pub relearning_steps_min: Vec<i64>,
    /// Interval (days) when a card graduates with "Good".
    pub graduating_interval: i64,
    /// Interval (days) when a card graduates with "Easy".
    pub easy_interval: i64,
    /// Starting ease in per-mille (2500 = 2.5x).
    pub starting_ease: i64,
    /// Bonus multiplier applied on "Easy".
    pub easy_bonus: f64,
    /// Multiplier applied on "Hard".
    pub hard_factor: f64,
    /// Global interval modifier.
    pub interval_modifier: f64,
    /// Maximum interval in days.
    pub max_interval: i64,
    /// Minimum ease in per-mille.
    pub min_ease: i64,
    /// Fraction of the old interval kept after a lapse (Anki "new interval %").
    pub new_interval_pct: f64,
    /// Minimum interval (days) after a lapse.
    pub min_lapse_interval: i64,
}

impl Default for SchedConfig {
    fn default() -> Self {
        SchedConfig {
            learning_steps_min: vec![1, 10],
            relearning_steps_min: vec![10],
            graduating_interval: 1,
            easy_interval: 4,
            starting_ease: 2500,
            easy_bonus: 1.3,
            hard_factor: 1.2,
            interval_modifier: 1.0,
            max_interval: 36500,
            min_ease: 1300,
            new_interval_pct: 0.0,
            min_lapse_interval: 1,
        }
    }
}

/// The mutable scheduling state of a card.
#[derive(Clone, Debug)]
pub struct CardSched {
    pub state: String, // new | learning | review | relearning | suspended
    pub step: i64,
    pub interval: i64, // days
    pub ease: i64,     // per-mille
    pub reps: i64,
    pub lapses: i64,
    pub due: i64, // epoch ms
}

fn clamp_ivl(ivl: i64, cfg: &SchedConfig) -> i64 {
    ivl.max(1).min(cfg.max_interval)
}

/// Apply a rating (1=Again .. 4=Easy) and return the new scheduling state.
pub fn answer(card: &CardSched, rating: i64, now: i64, cfg: &SchedConfig) -> CardSched {
    let mut n = card.clone();
    n.reps += 1;
    match card.state.as_str() {
        "new" | "learning" => {
            let steps = cfg.learning_steps_min.clone();
            schedule_learning(&mut n, &steps, rating, now, cfg, false);
        }
        "relearning" => {
            let steps = cfg.relearning_steps_min.clone();
            schedule_learning(&mut n, &steps, rating, now, cfg, true);
        }
        _ => schedule_review(&mut n, rating, now, cfg),
    }
    n
}

fn graduate(n: &mut CardSched, now: i64, cfg: &SchedConfig, is_relearn: bool, easy: bool) {
    n.state = "review".into();
    n.step = 0;
    if is_relearn {
        if n.interval < cfg.min_lapse_interval {
            n.interval = cfg.min_lapse_interval;
        }
        if easy {
            n.interval = clamp_ivl((n.interval as f64 * cfg.easy_bonus).round() as i64, cfg);
        }
    } else {
        if n.ease == 0 {
            n.ease = cfg.starting_ease;
        }
        n.interval = if easy {
            cfg.easy_interval
        } else {
            cfg.graduating_interval
        };
    }
    n.due = now + n.interval * DAY_MS;
}

fn schedule_learning(
    n: &mut CardSched,
    steps: &[i64],
    rating: i64,
    now: i64,
    cfg: &SchedConfig,
    is_relearn: bool,
) {
    if n.ease == 0 {
        n.ease = cfg.starting_ease;
    }
    let learning_state = if is_relearn { "relearning" } else { "learning" };
    match rating {
        AGAIN => {
            n.state = learning_state.into();
            n.step = 0;
            let delay = steps.first().copied().unwrap_or(1);
            n.due = now + delay * MIN_MS;
        }
        HARD => {
            n.state = learning_state.into();
            // Stay on the current step.
            let idx = (n.step as usize).min(steps.len().saturating_sub(1));
            let delay = steps.get(idx).copied().unwrap_or(1);
            n.due = now + delay * MIN_MS;
        }
        GOOD => {
            let next = n.step + 1;
            if next as usize >= steps.len() {
                graduate(n, now, cfg, is_relearn, false);
            } else {
                n.state = learning_state.into();
                n.step = next;
                n.due = now + steps[next as usize] * MIN_MS;
            }
        }
        _ => graduate(n, now, cfg, is_relearn, true), // EASY
    }
}

fn schedule_review(n: &mut CardSched, rating: i64, now: i64, cfg: &SchedConfig) {
    if n.ease == 0 {
        n.ease = cfg.starting_ease;
    }
    let base = n.interval.max(1) as f64;
    match rating {
        AGAIN => {
            n.lapses += 1;
            n.ease = (n.ease - 200).max(cfg.min_ease);
            let lapsed = ((base * cfg.new_interval_pct).round() as i64).max(cfg.min_lapse_interval);
            n.interval = clamp_ivl(lapsed, cfg);
            if cfg.relearning_steps_min.is_empty() {
                n.state = "review".into();
                n.due = now + n.interval * DAY_MS;
            } else {
                n.state = "relearning".into();
                n.step = 0;
                n.due = now + cfg.relearning_steps_min[0] * MIN_MS;
            }
        }
        HARD => {
            let ivl = (base * cfg.hard_factor * cfg.interval_modifier).round() as i64;
            n.interval = clamp_ivl(ivl.max(n.interval + 1), cfg);
            n.ease = (n.ease - 150).max(cfg.min_ease);
            n.due = now + n.interval * DAY_MS;
        }
        GOOD => {
            let ivl = (base * (n.ease as f64 / 1000.0) * cfg.interval_modifier).round() as i64;
            n.interval = clamp_ivl(ivl.max(n.interval + 1), cfg);
            n.due = now + n.interval * DAY_MS;
        }
        _ => {
            // EASY
            let ivl =
                (base * (n.ease as f64 / 1000.0) * cfg.easy_bonus * cfg.interval_modifier).round()
                    as i64;
            n.interval = clamp_ivl(ivl.max(n.interval + 1), cfg);
            n.ease += 150;
            n.due = now + n.interval * DAY_MS;
        }
    }
}

/// Human-readable labels for the four grading buttons, computed by running the
/// scheduler once per rating and formatting the resulting delay.
pub fn previews(card: &CardSched, now: i64, cfg: &SchedConfig) -> [String; 4] {
    let mut out = [String::new(), String::new(), String::new(), String::new()];
    for (i, rating) in [AGAIN, HARD, GOOD, EASY].iter().enumerate() {
        let next = answer(card, *rating, now, cfg);
        out[i] = format_delay(next.due - now);
    }
    out
}

/// Format a millisecond delay the way Anki labels its buttons.
pub fn format_delay(ms: i64) -> String {
    let secs = ms.max(0) / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;
    if mins < 1 {
        format!("{}s", secs.max(1))
    } else if hours < 1 {
        format!("{}m", mins)
    } else if days < 1 {
        format!("{}h", hours)
    } else if days < 30 {
        format!("{}d", days)
    } else if days < 365 {
        format!("{:.1}mo", days as f64 / 30.0)
    } else {
        format!("{:.1}y", days as f64 / 365.0)
    }
}
