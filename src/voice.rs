use crate::config::Config;
use crate::sprite::Sprite;
use crate::state::{Context, Signals};

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

pub fn synthesize(ctx: &Context, sig: &Signals, pose: &str, cfg: &Config, _sprite: &Sprite) -> String {
    let name_owned;
    let name: &str = match cfg.name.as_deref() {
        Some(n) => n,
        None => {
            name_owned = capitalize(&ctx.user);
            &name_owned
        }
    };

    // The pose is the dominant mood. The voice line reinforces the *why*
    // by picking the single most salient signal.
    match pose {
        "sick" => {
            if matches!(sig.battery_pct, Some(p) if p < 10) {
                "i'm running on fumes. plug me in?".into()
            } else if matches!(sig.disk_pct_used, Some(d) if d > 95) {
                "i can't breathe. clear some disk space.".into()
            } else {
                "something's off. take a look.".into()
            }
        }
        "adorned" => format!(
            "day {} — look at you go, {}.",
            sig.streak_days, name
        ),
        "shivering" => match sig.temp_c {
            Some(t) => format!("brrr, {}°. wear something warm.", t.round() as i32),
            None => "it's cold out. bundle up.".into(),
        },
        "excited" => "something good just landed.".into(),
        "concerned" => {
            if sig.gh_review_requests >= 3 {
                format!(
                    "{} review{} waiting on you.",
                    sig.gh_review_requests,
                    if sig.gh_review_requests == 1 { "" } else { "s" }
                )
            } else if matches!(sig.dirty_since, Some(d) if d.as_secs() > 24 * 3600) {
                "your tree has been dirty for a while. commit or stash?".into()
            } else if matches!(sig.disk_pct_used, Some(d) if d > 85) {
                "disk is getting tight.".into()
            } else {
                "a few things want your attention.".into()
            }
        }
        "sleeping" => format!("g'night, {}. sleep well.", name),
        "working" => {
            if sig.dirty_count > 0 {
                format!("you've got {} change{} in flight.", sig.dirty_count, if sig.dirty_count == 1 { "" } else { "s" })
            } else if sig.in_git_repo {
                "clean tree. nice place to start.".into()
            } else {
                "settling in.".into()
            }
        }
        _ => match sig.streak_days {
            0 => format!("hello, {}.", name),
            1 => format!("welcome back, {}.", name),
            n => format!("day {}, {}. one glance at a time.", n, name),
        },
    }
}
