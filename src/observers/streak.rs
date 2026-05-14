use crate::observers::Observer;
use crate::state::{Context, Observation, Reading, Signals};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct Streak;

#[derive(Debug, Serialize, Deserialize, Default)]
struct StreakFile {
    last_seen: Option<String>, // YYYY-MM-DD
    count: u32,
    longest: u32,
}

impl Observer for Streak {
    fn name(&self) -> &'static str {
        "streak"
    }

    fn read(&self, ctx: &Context) -> Reading {
        let mut sig = Signals::default();
        let mut obs = Vec::new();

        let s = read_file();
        let today = ctx.now.date_naive();

        let (count, _ ) = compute_today_count(&s, today);
        sig.streak_days = count;
        if is_milestone(count) {
            sig.streak_milestone = true;
        }

        if count > 0 {
            let priority = if is_milestone(count) { 65 } else { 25 };
            let line = if is_milestone(count) {
                format!("day {} of your streak — well done", count)
            } else {
                format!("day {} of your streak", count)
            };
            obs.push(Observation {
                priority,
                tag: "◐".into(),
                line,
            });
        }

        Reading {
            signals: sig,
            observations: obs,
        }
    }
}

fn streak_path() -> Option<PathBuf> {
    dirs::state_dir()
        .or_else(dirs::data_dir)
        .map(|d| d.join("leapoh/streak.json"))
}

fn read_file() -> StreakFile {
    let Some(p) = streak_path() else { return StreakFile::default() };
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn write_file(s: &StreakFile) {
    let Some(p) = streak_path() else { return };
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(t) = serde_json::to_string(s) {
        let _ = std::fs::write(&p, t);
    }
}

fn compute_today_count(s: &StreakFile, today: NaiveDate) -> (u32, bool) {
    let last = s.last_seen.as_deref().and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    match last {
        Some(d) if d == today => (s.count.max(1), false),
        Some(d) => {
            let diff = (today - d).num_days();
            if diff == 1 {
                (s.count + 1, true)
            } else {
                (1, true)
            }
        }
        None => (1, true),
    }
}

pub fn record_invocation(ctx: &Context) {
    let mut s = read_file();
    let today = ctx.now.date_naive();
    let last = s.last_seen.as_deref().and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let need_write = match last {
        Some(d) if d == today => false,
        Some(d) => {
            let diff = (today - d).num_days();
            s.count = if diff == 1 { s.count + 1 } else { 1 };
            s.last_seen = Some(today.format("%Y-%m-%d").to_string());
            if s.count > s.longest {
                s.longest = s.count;
            }
            true
        }
        None => {
            s.count = 1;
            s.last_seen = Some(today.format("%Y-%m-%d").to_string());
            s.longest = s.longest.max(1);
            true
        }
    };
    if need_write {
        write_file(&s);
    }
}

fn is_milestone(n: u32) -> bool {
    matches!(n, 7 | 14 | 30 | 60 | 100 | 200 | 365 | 500 | 1000)
}
