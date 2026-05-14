use crate::config::Config;
use crate::sprite::{Pose, Sprite};
use crate::state::{Context, Observation};
use crate::theme::{self, Theme};

const GUTTER: usize = 4;
const TAG_WIDTH: usize = 6;

pub fn render(
    ctx: &Context,
    sprite: &Sprite,
    art: &Pose,
    observations: &[Observation],
    voice: &str,
    cfg: &Config,
    theme: &Theme,
) -> String {
    let creature_color = crate::sprite::pose_color(sprite, art)
        .unwrap_or(theme::SPEAKER);

    let left_raw_lines: Vec<&str> = art
        .art
        .lines()
        .filter(|l| !(l.is_empty() && true))
        .collect();
    let left_raw_lines: Vec<&str> = if left_raw_lines.first().map_or(false, |l| l.is_empty()) {
        left_raw_lines.into_iter().skip(1).collect()
    } else {
        left_raw_lines
    };
    let left_width = left_raw_lines.iter().map(|l| visual_width(l)).max().unwrap_or(0);

    let right_lines = build_right_lines(ctx, sprite, observations, voice, cfg, theme);

    let height = left_raw_lines.len().max(right_lines.len());
    let mut out = String::new();
    for i in 0..height {
        let raw = left_raw_lines.get(i).copied().unwrap_or("");
        let painted = if raw.is_empty() {
            String::new()
        } else {
            theme.fg(creature_color, raw)
        };
        let pad = left_width.saturating_sub(visual_width(raw)) + GUTTER;
        out.push_str(&painted);
        for _ in 0..pad {
            out.push(' ');
        }
        let right = right_lines.get(i).map(String::as_str).unwrap_or("");
        out.push_str(right);
        out.push('\n');
    }
    out
}

fn build_right_lines(
    ctx: &Context,
    sprite: &Sprite,
    observations: &[Observation],
    voice: &str,
    cfg: &Config,
    theme: &Theme,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(header(ctx, cfg, theme));
    lines.push(String::new());
    for obs in observations {
        let tag = obs.tag.as_str();
        let glyph = theme.fg(theme::tag_color(tag), &format!("{:<width$}", tag, width = TAG_WIDTH));
        lines.push(format!("{}{}", glyph, obs.line));
    }
    // Pad with blank lines so the voice always lands at the bottom
    while lines.len() < 7 {
        lines.push(String::new());
    }
    let speaker_name = speaker(sprite);
    let speaker_color = sprite.color.unwrap_or(theme::SPEAKER);
    let speaker_styled = theme.bold_fg(speaker_color, &speaker_name);
    let quote = theme.italic_fg(theme::VOICE, &format!("\"{}\"", voice));
    lines.push(format!("{} {} {}", speaker_styled, theme.dim("·"), quote));
    lines
}

fn speaker(sprite: &Sprite) -> String {
    // Use first 3 chars of the creature name as a short voice tag.
    let n = sprite.name.chars().take(3).collect::<String>();
    if n.is_empty() {
        "pet".into()
    } else {
        n
    }
}

fn header(ctx: &Context, cfg: &Config, theme: &Theme) -> String {
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
    let core = format!("{greeting}, ");
    let name_styled = theme.bold_fg(theme::HEADER, &who);
    let when_styled = theme.fg(theme::HEADER_DIM, &when);
    let sep = theme.fg(theme::HEADER_DIM, " · ");
    format!("{}{}{}{}", theme.dim(&core), name_styled, sep, when_styled)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Visual width of a string with ANSI escapes stripped. Treats each
/// Unicode scalar as width 1 — acceptable while sprites are BMP only.
pub fn visual_width(s: &str) -> usize {
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
