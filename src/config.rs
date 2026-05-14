use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const HELP: &str = "leapoh — a calm CLI creature that compresses your morning into a glance.

Usage:
  leapoh [options]

Options:
  --creature <name|path>   Pick a creature (default from config, or 'axolotl')
  --pose <name>            Force a specific pose (for debugging sprites)
  --config <path>          Path to config file (default ~/.config/leapoh/config.toml)
  --no-color               Suppress color output (color is off by default for now)
  -h, --help               Show this help

Built-in poses: working, sleeping, excited, concerned, sick, shivering, adorned, idle
";

#[derive(Debug, Default)]
pub struct Args {
    pub creature: Option<String>,
    pub force_pose: Option<String>,
    pub config: Option<PathBuf>,
    pub no_color: bool,
    pub help: bool,
}

impl Args {
    pub fn parse<I: Iterator<Item = String>>(mut it: I) -> Result<Self, String> {
        let mut a = Args::default();
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "-h" | "--help" => a.help = true,
                "--no-color" => a.no_color = true,
                "--creature" => {
                    a.creature = Some(it.next().ok_or("--creature needs a value")?);
                }
                "--pose" => {
                    a.force_pose = Some(it.next().ok_or("--pose needs a value")?);
                }
                "--config" => {
                    a.config = Some(PathBuf::from(it.next().ok_or("--config needs a value")?));
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }
        Ok(a)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_creature")]
    pub creature: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub location: Option<Location>,
    #[serde(default = "default_max_obs")]
    pub max_observations: u8,
    #[serde(default = "default_observers")]
    pub observers: Vec<String>,
    #[serde(default)]
    pub workday_start: Option<u32>,
    #[serde(default)]
    pub workday_end: Option<u32>,
    #[serde(default)]
    pub no_color: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    #[serde(default)]
    pub label: Option<String>,
}

fn default_creature() -> String {
    "axolotl".into()
}
fn default_max_obs() -> u8 {
    5
}
fn default_observers() -> Vec<String> {
    vec![
        "git".into(),
        "github".into(),
        "weather".into(),
        "system".into(),
        "streak".into(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Config {
            creature: default_creature(),
            name: None,
            location: None,
            max_observations: default_max_obs(),
            observers: default_observers(),
            workday_start: None,
            workday_end: None,
            no_color: false,
        }
    }
}

impl Config {
    pub fn load(explicit: Option<&Path>) -> Option<Self> {
        let path = explicit
            .map(|p| p.to_path_buf())
            .or_else(|| dirs::config_dir().map(|d| d.join("leapoh/config.toml")))?;
        let text = std::fs::read_to_string(&path).ok()?;
        toml::from_str(&text).ok()
    }
}
