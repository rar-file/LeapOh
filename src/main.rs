mod config;
mod observers;
mod pose;
mod render;
mod sprite;
mod state;
mod theme;
mod voice;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args = match config::Args::parse(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("leapoh: {e}");
            return ExitCode::from(2);
        }
    };

    if args.help {
        print!("{}", config::HELP);
        return ExitCode::SUCCESS;
    }

    let cfg = config::Config::load(args.config.as_deref()).unwrap_or_default();
    let ctx = state::Context::gather(&cfg);

    let color_mode = match std::env::var("LEAPOH_COLOR").as_deref() {
        Ok("always") => theme::ColorMode::Always,
        Ok("never") => theme::ColorMode::Never,
        _ if args.no_color => theme::ColorMode::Never,
        _ => theme::ColorMode::Auto,
    };
    let active_theme = theme::Theme::new(color_mode.resolve());

    let mut signals = state::Signals::default();
    let mut observations = Vec::new();

    let readings: Vec<state::Reading> = std::thread::scope(|s| {
        let handles: Vec<_> = observers::active(&cfg)
            .into_iter()
            .map(|obs| {
                let ctx_ref = &ctx;
                s.spawn(move || obs.read(ctx_ref))
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_default())
            .collect()
    });
    for reading in readings {
        signals.merge(reading.signals);
        observations.extend(reading.observations);
    }

    let pose_name = args
        .force_pose
        .clone()
        .unwrap_or_else(|| pose::select(&ctx, &signals).to_string());

    let sprite = sprite::load(&cfg, args.creature.as_deref());
    let art = sprite
        .poses
        .get(&pose_name)
        .or_else(|| sprite.poses.get(&sprite.default_pose))
        .cloned()
        .unwrap_or_else(|| sprite::Pose {
            art: "?".into(),
            color: None,
        });

    let voice_line = voice::synthesize(&ctx, &signals, &pose_name, &cfg, &sprite);

    observations.sort_by(|a, b| b.priority.cmp(&a.priority));
    observations.truncate(cfg.max_observations as usize);

    let output = render::render(&ctx, &sprite, &art, &observations, &voice_line, &cfg, &active_theme);
    print!("{output}");

    observers::streak::record_invocation(&ctx);

    ExitCode::SUCCESS
}
