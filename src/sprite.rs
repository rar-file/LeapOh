use crate::config::Config;
use crate::theme::Color;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

const EMBEDDED_FOX: &str = include_str!("assets/fox.toml");
const EMBEDDED_AXOLOTL: &str = include_str!("assets/axolotl.toml");

#[derive(Debug, Deserialize, Clone)]
pub struct Sprite {
    pub name: String,
    #[serde(default = "default_default_pose")]
    pub default_pose: String,
    #[serde(default)]
    pub color: Option<Color>,
    pub poses: BTreeMap<String, Pose>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Pose {
    pub art: String,
    #[serde(default)]
    pub color: Option<Color>,
}

fn default_default_pose() -> String {
    "idle".into()
}

pub fn load(cfg: &Config, cli_override: Option<&str>) -> Sprite {
    let pick = cli_override.unwrap_or(cfg.creature.as_str());
    if let Some(sp) = try_load(pick) {
        return sp;
    }
    parse_embedded(EMBEDDED_FOX)
}

fn try_load(pick: &str) -> Option<Sprite> {
    if pick.contains('/') || pick.ends_with(".toml") {
        let text = std::fs::read_to_string(Path::new(pick)).ok()?;
        return toml::from_str(&text).ok();
    }
    if let Some(home) = dirs::config_dir() {
        let candidate = home.join(format!("leapoh/creatures/{pick}.toml"));
        if let Ok(text) = std::fs::read_to_string(candidate) {
            if let Ok(sp) = toml::from_str(&text) {
                return Some(sp);
            }
        }
    }
    match pick {
        "fox" => Some(parse_embedded(EMBEDDED_FOX)),
        "axolotl" => Some(parse_embedded(EMBEDDED_AXOLOTL)),
        _ => None,
    }
}

fn parse_embedded(text: &str) -> Sprite {
    toml::from_str(text).expect("embedded sprite must parse")
}

pub fn pose_color(sprite: &Sprite, pose: &Pose) -> Option<Color> {
    pose.color.or(sprite.color)
}
