use crate::state::{Context, Signals};
use chrono::Timelike;

pub fn select(ctx: &Context, sig: &Signals) -> &'static str {
    let h = ctx.now.hour();

    if matches!(sig.battery_pct, Some(p) if p < 10 && sig.on_ac != Some(true))
        || matches!(sig.disk_pct_used, Some(d) if d > 95)
    {
        return "sick";
    }

    if sig.streak_milestone {
        return "adorned";
    }

    if matches!(sig.temp_c, Some(t) if t < 5.0) {
        return "shivering";
    }

    if sig.recent_merge {
        return "excited";
    }

    let dirty_long = matches!(sig.dirty_since, Some(d) if d.as_secs() > 24 * 3600);
    let low_disk = matches!(sig.disk_pct_used, Some(d) if d > 85);
    if dirty_long || low_disk || sig.gh_review_requests >= 3 {
        return "concerned";
    }

    if h < 6 || h >= 23 {
        return "sleeping";
    }

    let workday_start = ctx.cfg.workday_start.unwrap_or(8);
    let workday_end = ctx.cfg.workday_end.unwrap_or(18);
    if h >= workday_start && h < workday_end && sig.in_git_repo {
        return "working";
    }

    "idle"
}
