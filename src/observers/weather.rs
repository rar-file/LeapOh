use crate::observers::Observer;
use crate::state::{Context, Observation, Reading, Signals};
use serde::Deserialize;
use std::time::Duration;

pub struct Weather;

#[derive(Deserialize)]
struct Resp {
    current: Current,
}
#[derive(Deserialize)]
struct Current {
    temperature_2m: f32,
    weather_code: u32,
}

impl Observer for Weather {
    fn name(&self) -> &'static str {
        "weather"
    }

    fn read(&self, ctx: &Context) -> Reading {
        let mut sig = Signals::default();
        let mut obs = Vec::new();

        let Some(loc) = ctx.cfg.location.clone() else {
            return Reading { signals: sig, observations: obs };
        };

        // Cache file: ~/.cache/leapoh/weather.json — 30 min TTL
        let cache_path = dirs::cache_dir().map(|d| d.join("leapoh/weather.json"));
        if let Some(p) = &cache_path {
            if let Some((t, c)) = read_cache(p, 30 * 60) {
                sig.temp_c = Some(t);
                obs.push(Observation {
                    priority: priority_for_temp(t),
                    tag: "wx".into(),
                    line: format!("{}° {}", t.round() as i32, describe(c)),
                });
                return Reading { signals: sig, observations: obs };
            }
        }

        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code",
            loc.lat, loc.lon
        );
        let resp: Option<Resp> = ureq::AgentBuilder::new()
            .timeout(Duration::from_millis(800))
            .build()
            .get(&url)
            .call()
            .ok()
            .and_then(|r| r.into_json().ok());

        if let Some(r) = resp {
            sig.temp_c = Some(r.current.temperature_2m);
            obs.push(Observation {
                priority: priority_for_temp(r.current.temperature_2m),
                tag: "wx".into(),
                line: format!(
                    "{}° {}",
                    r.current.temperature_2m.round() as i32,
                    describe(r.current.weather_code)
                ),
            });
            if let Some(p) = cache_path {
                let _ = write_cache(&p, r.current.temperature_2m, r.current.weather_code);
            }
        }

        Reading { signals: sig, observations: obs }
    }
}

fn priority_for_temp(t: f32) -> u8 {
    if t < 0.0 {
        65
    } else if t < 5.0 {
        55
    } else {
        30
    }
}

fn describe(code: u32) -> &'static str {
    match code {
        0 => "clear",
        1 | 2 => "partly cloudy",
        3 => "overcast",
        45 | 48 => "fog",
        51..=57 => "drizzle",
        61..=67 => "rain",
        71..=77 => "snow",
        80..=82 => "showers",
        85 | 86 => "snow showers",
        95..=99 => "storm",
        _ => "",
    }
}

fn read_cache(path: &std::path::Path, ttl_secs: u64) -> Option<(f32, u32)> {
    let meta = std::fs::metadata(path).ok()?;
    let age = meta.modified().ok()?.elapsed().ok()?.as_secs();
    if age > ttl_secs {
        return None;
    }
    let text = std::fs::read_to_string(path).ok()?;
    let mut parts = text.split_whitespace();
    let t: f32 = parts.next()?.parse().ok()?;
    let c: u32 = parts.next()?.parse().ok()?;
    Some((t, c))
}

fn write_cache(path: &std::path::Path, t: f32, c: u32) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format!("{} {}", t, c))
}
