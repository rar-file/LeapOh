use crate::config::Config;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

const EMBEDDED_AXOLOTL: &str = include_str!("assets/axolotl.toml");

#[derive(Debug, Deserialize, Clone)]
pub struct Sprite {
    pub name: String,
    #[serde(default = "default_default_pose")]
    pub default_pose: String,
    pub poses: BTreeMap<String, Pose>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Pose {
    pub art: String,
}

fn default_default_pose() -> String {
    "idle".into()
}

pub fn load(cfg: &Config, cli_override: Option<&str>) -> Sprite {
    let pick = cli_override.unwrap_or(cfg.creature.as_str());
    if let Some(sp) = try_load(pick) {
        return sp;
    }
    // Fallback: embedded axolotl
    parse_embedded()
}

fn try_load(pick: &str) -> Option<Sprite> {
    // Path-like? Load from file.
    if pick.contains('/') || pick.ends_with(".toml") {
        let text = std::fs::read_to_string(Path::new(pick)).ok()?;
        return toml::from_str(&text).ok();
    }
    // Named creature in user dir?
    if let Some(home) = dirs::config_dir() {
        let candidate = home.join(format!("leapoh/creatures/{pick}.toml"));
        if let Ok(text) = std::fs::read_to_string(candidate) {
            if let Ok(sp) = toml::from_str(&text) {
                return Some(sp);
            }
        }
    }
    // Built-in named creature
    match pick {
        "axolotl" => Some(parse_embedded()),
        _ => None,
    }
}

fn parse_embedded() -> Sprite {
    toml::from_str(EMBEDDED_AXOLOTL).expect("embedded axolotl sprite must parse")
}

pub fn fallback_art() -> Pose {
    Pose {
        art: "    .---.\n   /     \\\n *( o   o )*\n   \\  _  /\n    '---'\n    /   \\\n   /__|__\\\n    |   |\n".into(),
    }
}
