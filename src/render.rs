use crate::config::Config;
use crate::sprite::Pose;
use crate::state::{Context, Observation};

const GUTTER: usize = 3;

pub fn render(
    ctx: &Context,
    art: &Pose,
    observations: &[Observation],
    voice: &str,
    cfg: &Config,
) -> String {
    let left_lines: Vec<String> = art
        .art
        .lines()
        .map(|l| l.trim_end_matches('\n').to_string())
        .filter(|l| !l.is_empty() || true)
        .collect();
    let left_width = left_lines
        .iter()
        .map(|l| visual_width(l))
        .max()
        .unwrap_or(0);

    let right_lines = build_right_lines(ctx, observations, voice, cfg);

    let height = left_lines.len().max(right_lines.len());
    let mut out = String::new();
    for i in 0..height {
        let left = left_lines.get(i).map(String::as_str).unwrap_or("");
        let right = right_lines.get(i).map(String::as_str).unwrap_or("");
        let pad = left_width.saturating_sub(visual_width(left)) + GUTTER;
        out.push_str(left);
        for _ in 0..pad {
            out.push(' ');
        }
        out.push_str(right);
        out.push('\n');
    }
    out
}

fn build_right_lines(
    ctx: &Context,
    observations: &[Observation],
    voice: &str,
    cfg: &Config,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(header(ctx, cfg));
    lines.push(String::new());
    for obs in observations {
        lines.push(format!("{:<4} {}", obs.tag, obs.line));
    }
    // Always end with the voice line, after a blank.
    lines.push(String::new());
    let speaker = cfg.creature.chars().take(3).collect::<String>();
    lines.push(format!("{} says: \"{}\"", speaker, voice));
    lines
}

fn header(ctx: &Context, cfg: &Config) -> String {
    use chrono::Timelike;
    let when = ctx.now.format("%A · %H:%M").to_string();
    let who = cfg
        .name
        .clone()
        .unwrap_or_else(|| capitalize(&ctx.user));
    let h = ctx.now.hour();
    let greeting = match h {
        5..=11 => "good morning",
        12..=16 => "good afternoon",
        17..=21 => "good evening",
        _ => "still up",
    };
    format!("{greeting}, {who} — {when}")
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn visual_width(s: &str) -> usize {
    // Best-effort: count chars, ignoring ANSI escapes. Sprites are ASCII for now.
    let mut w = 0;
    let mut in_esc = false;
    for c in s.chars() {
        if in_esc {
            if c.is_ascii_alphabetic() {
                in_esc = false;
            }
            continue;
        }
        if c == '\x1b' {
            in_esc = true;
            continue;
        }
        w += 1;
    }
    w
}
