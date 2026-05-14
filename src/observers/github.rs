use crate::observers::Observer;
use crate::state::{Context, Observation, Reading, Signals};
use std::path::PathBuf;
use std::process::Command;

pub struct Github;

const CACHE_TTL_SECS: u64 = 60;

impl Observer for Github {
    fn name(&self) -> &'static str {
        "github"
    }

    fn read(&self, _ctx: &Context) -> Reading {
        let mut sig = Signals::default();
        let mut obs = Vec::new();

        let (reviews, notifs) = match read_cache() {
            Some(v) => v,
            None => {
                if !has_gh() {
                    return Reading { signals: sig, observations: obs };
                }
                let r = gh_search_count("is:open is:pr review-requested:@me").unwrap_or(0);
                let n = gh_notifications_count().unwrap_or(0);
                write_cache(r, n);
                (r, n)
            }
        };

        sig.gh_review_requests = reviews;
        sig.gh_notifications = notifs;

        if reviews > 0 {
            obs.push(Observation {
                priority: 75,
                tag: "gh".into(),
                line: format!("{} review request{}", reviews, if reviews == 1 { "" } else { "s" }),
            });
        }
        if notifs > 0 {
            obs.push(Observation {
                priority: 50,
                tag: "gh".into(),
                line: format!("{} notification{}", notifs, if notifs == 1 { "" } else { "s" }),
            });
        }

        Reading { signals: sig, observations: obs }
    }
}

fn cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("leapoh/github.txt"))
}

fn read_cache() -> Option<(u32, u32)> {
    let p = cache_path()?;
    let meta = std::fs::metadata(&p).ok()?;
    let age = meta.modified().ok()?.elapsed().ok()?.as_secs();
    if age > CACHE_TTL_SECS {
        return None;
    }
    let text = std::fs::read_to_string(&p).ok()?;
    let mut parts = text.split_whitespace();
    let r: u32 = parts.next()?.parse().ok()?;
    let n: u32 = parts.next()?.parse().ok()?;
    Some((r, n))
}

fn write_cache(r: u32, n: u32) {
    let Some(p) = cache_path() else { return };
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&p, format!("{} {}", r, n));
}

fn has_gh() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn gh_search_count(query: &str) -> Option<u32> {
    let out = Command::new("gh")
        .args(["search", "prs", query, "--json", "url", "--limit", "100"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    // count occurrences of "url":
    Some(text.matches("\"url\"").count() as u32)
}

fn gh_notifications_count() -> Option<u32> {
    let out = Command::new("gh")
        .args(["api", "notifications", "--jq", ". | length"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}
