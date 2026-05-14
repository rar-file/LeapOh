use crate::observers::Observer;
use crate::state::{Context, Observation, Reading, Signals};
use std::process::Command;

pub struct System;

impl Observer for System {
    fn name(&self) -> &'static str {
        "system"
    }

    fn read(&self, _ctx: &Context) -> Reading {
        let mut sig = Signals::default();
        let mut obs = Vec::new();

        if let Some((pct, on_ac)) = battery() {
            sig.battery_pct = Some(pct);
            sig.on_ac = Some(on_ac);
            if pct < 20 && !on_ac {
                obs.push(Observation {
                    priority: 80,
                    tag: "bat".into(),
                    line: format!("battery {}%", pct),
                });
            } else if pct < 10 {
                obs.push(Observation {
                    priority: 90,
                    tag: "bat".into(),
                    line: format!("battery {}% — plug in", pct),
                });
            }
        }

        if let Some(used) = disk_used_pct("/") {
            sig.disk_pct_used = Some(used);
            if used >= 85 {
                obs.push(Observation {
                    priority: 75,
                    tag: "dsk".into(),
                    line: format!("disk {}% used", used),
                });
            }
        }

        Reading {
            signals: sig,
            observations: obs,
        }
    }
}

fn battery() -> Option<(u8, bool)> {
    let dir = std::fs::read_dir("/sys/class/power_supply").ok()?;
    for entry in dir.flatten() {
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if !name_s.starts_with("BAT") {
            continue;
        }
        let cap = std::fs::read_to_string(entry.path().join("capacity")).ok()?;
        let pct: u8 = cap.trim().parse().ok()?;
        let status = std::fs::read_to_string(entry.path().join("status"))
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let on_ac = matches!(status.as_str(), "Charging" | "Full" | "Not charging");
        return Some((pct, on_ac));
    }
    None
}

fn disk_used_pct(path: &str) -> Option<u8> {
    let out = Command::new("df")
        .args(["-P", path])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().nth(1)?;
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 5 {
        return None;
    }
    cols[4].trim_end_matches('%').parse().ok()
}
