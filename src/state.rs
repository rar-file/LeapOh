use crate::config::Config;
use chrono::{DateTime, Local};
use std::path::PathBuf;
use std::time::Duration;

pub struct Context {
    pub now: DateTime<Local>,
    pub cwd: PathBuf,
    pub hostname: String,
    pub user: String,
    pub cfg: Config,
}

impl Context {
    pub fn gather(cfg: &Config) -> Self {
        Context {
            now: Local::now(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            hostname: std::env::var("HOSTNAME")
                .ok()
                .or_else(read_hostname_file)
                .unwrap_or_else(|| "localhost".into()),
            user: std::env::var("USER").unwrap_or_else(|_| "you".into()),
            cfg: cfg.clone(),
        }
    }
}

fn read_hostname_file() -> Option<String> {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
}

#[derive(Debug, Default, Clone)]
pub struct Signals {
    pub in_git_repo: bool,
    pub dirty_count: u32,
    pub dirty_since: Option<Duration>,
    pub unpushed: u32,
    pub branch: Option<String>,
    pub battery_pct: Option<u8>,
    pub on_ac: Option<bool>,
    pub disk_pct_used: Option<u8>,
    pub temp_c: Option<f32>,
    pub gh_review_requests: u32,
    pub gh_notifications: u32,
    pub recent_merge: bool,
    pub streak_days: u32,
    pub streak_milestone: bool,
    pub event_in_minutes: Option<i64>,
    pub event_title: Option<String>,
}

impl Signals {
    pub fn merge(&mut self, other: Signals) {
        if other.in_git_repo {
            self.in_git_repo = true;
        }
        if other.dirty_count > 0 {
            self.dirty_count = other.dirty_count;
        }
        if other.dirty_since.is_some() {
            self.dirty_since = other.dirty_since;
        }
        if other.unpushed > 0 {
            self.unpushed = other.unpushed;
        }
        if other.branch.is_some() {
            self.branch = other.branch;
        }
        if other.battery_pct.is_some() {
            self.battery_pct = other.battery_pct;
        }
        if other.on_ac.is_some() {
            self.on_ac = other.on_ac;
        }
        if other.disk_pct_used.is_some() {
            self.disk_pct_used = other.disk_pct_used;
        }
        if other.temp_c.is_some() {
            self.temp_c = other.temp_c;
        }
        if other.gh_review_requests > 0 {
            self.gh_review_requests = other.gh_review_requests;
        }
        if other.gh_notifications > 0 {
            self.gh_notifications = other.gh_notifications;
        }
        if other.recent_merge {
            self.recent_merge = true;
        }
        if other.streak_days > 0 {
            self.streak_days = other.streak_days;
        }
        if other.streak_milestone {
            self.streak_milestone = true;
        }
        if other.event_in_minutes.is_some() {
            self.event_in_minutes = other.event_in_minutes;
            self.event_title = other.event_title;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub priority: u8,
    pub tag: String,
    pub line: String,
}

#[derive(Debug, Default, Clone)]
pub struct Reading {
    pub observations: Vec<Observation>,
    pub signals: Signals,
}
