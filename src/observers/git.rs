use crate::observers::Observer;
use crate::state::{Context, Observation, Reading, Signals};
use std::process::Command;
use std::time::Duration;

pub struct Git;

impl Observer for Git {
    fn name(&self) -> &'static str {
        "git"
    }

    fn read(&self, ctx: &Context) -> Reading {
        let mut sig = Signals::default();
        let mut obs = Vec::new();

        if !is_repo(&ctx.cwd) {
            return Reading {
                signals: sig,
                observations: obs,
            };
        }
        sig.in_git_repo = true;

        let branch = run(&ctx.cwd, &["symbolic-ref", "--short", "HEAD"])
            .or_else(|| run(&ctx.cwd, &["rev-parse", "--abbrev-ref", "HEAD"]));
        let dirty = run(&ctx.cwd, &["status", "--porcelain"]);
        let upstream =
            run(&ctx.cwd, &["rev-list", "--left-right", "--count", "@{u}...HEAD"]);

        let has_commits = run(&ctx.cwd, &["rev-parse", "HEAD"]).is_some();
        let branch_s = branch
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| if has_commits { "detached".into() } else { "no commits yet".into() });
        sig.branch = Some(branch_s.clone());

        let dirty_count = dirty
            .as_deref()
            .map(|s| s.lines().filter(|l| !l.is_empty()).count() as u32)
            .unwrap_or(0);
        sig.dirty_count = dirty_count;

        // Age of the dirtiest tracked change
        if dirty_count > 0 {
            sig.dirty_since = oldest_change_age(&ctx.cwd);
        }

        let (behind, ahead) = upstream
            .as_deref()
            .and_then(parse_lr)
            .unwrap_or((0, 0));
        sig.unpushed = ahead;

        let mut parts = Vec::new();
        parts.push(branch_s.clone());
        if dirty_count > 0 {
            parts.push(format!("{} changed", dirty_count));
        } else {
            parts.push("clean".into());
        }
        if ahead > 0 {
            parts.push(format!("{} ahead", ahead));
        }
        if behind > 0 {
            parts.push(format!("{} behind", behind));
        }

        let priority = if dirty_count > 0 || ahead > 0 || behind > 0 {
            70
        } else {
            40
        };

        obs.push(Observation {
            priority,
            tag: "git".into(),
            line: parts.join(" · "),
        });

        // Recent merge → excite the creature
        if let Some(out) = run(
            &ctx.cwd,
            &["log", "-1", "--merges", "--since=1.hour", "--format=%s"],
        ) {
            if !out.trim().is_empty() {
                sig.recent_merge = true;
                obs.push(Observation {
                    priority: 60,
                    tag: "git".into(),
                    line: format!("merged: {}", truncate(out.trim(), 40)),
                });
            }
        }

        Reading {
            signals: sig,
            observations: obs,
        }
    }
}

fn is_repo(cwd: &std::path::Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false)
}

fn run(cwd: &std::path::Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).current_dir(cwd).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

fn parse_lr(s: &str) -> Option<(u32, u32)> {
    let mut it = s.split_whitespace();
    let behind = it.next()?.parse().ok()?;
    let ahead = it.next()?.parse().ok()?;
    Some((behind, ahead))
}

fn oldest_change_age(cwd: &std::path::Path) -> Option<Duration> {
    // Use git diff to find the oldest unstaged/staged change timestamp.
    // Heuristic: stat mtime of modified files.
    let out = run(cwd, &["status", "--porcelain"])?;
    let mut oldest: Option<std::time::SystemTime> = None;
    for line in out.lines() {
        if line.len() < 4 {
            continue;
        }
        let path = &line[3..];
        let full = cwd.join(path.trim_matches('"'));
        if let Ok(meta) = std::fs::metadata(&full) {
            if let Ok(mt) = meta.modified() {
                if oldest.map_or(true, |o| mt < o) {
                    oldest = Some(mt);
                }
            }
        }
    }
    oldest.and_then(|t| std::time::SystemTime::now().duration_since(t).ok())
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n - 1).collect();
        out.push('…');
        out
    }
}
