use crate::config::Config;
use crate::state::{Context, Reading};

pub mod git;
pub mod github;
pub mod streak;
pub mod system;
pub mod weather;

pub trait Observer: Send + Sync {
    fn name(&self) -> &'static str;
    fn read(&self, ctx: &Context) -> Reading;
}

pub fn active(cfg: &Config) -> Vec<Box<dyn Observer>> {
    let mut out: Vec<Box<dyn Observer>> = Vec::new();
    for n in &cfg.observers {
        match n.as_str() {
            "git" => out.push(Box::new(git::Git)),
            "github" => out.push(Box::new(github::Github)),
            "weather" => out.push(Box::new(weather::Weather)),
            "system" => out.push(Box::new(system::System)),
            "streak" => out.push(Box::new(streak::Streak)),
            _ => {}
        }
    }
    out
}
