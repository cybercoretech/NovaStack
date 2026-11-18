use crate::cli::PluginChoice;
use crate::error::Result;
use std::collections::HashMap;

pub mod auth;
pub mod otel;
pub mod redis;
pub mod workers;

pub trait Plugin: Send + Sync {
    fn choice(&self) -> PluginChoice;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn dependencies(&self) -> &'static str;
    fn template_files(&self) -> Vec<(&'static str, &'static str)>;
    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()>;
}

pub fn get_plugin(choice: PluginChoice) -> Box<dyn Plugin> {
    match choice {
        PluginChoice::Auth => Box::new(auth::AuthPlugin),
        PluginChoice::Otel => Box::new(otel::OtelPlugin),
        PluginChoice::Redis => Box::new(redis::RedisPlugin),
        PluginChoice::Workers => Box::new(workers::WorkersPlugin),
    }
}
