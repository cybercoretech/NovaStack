use super::Plugin;
use crate::cli::PluginChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher;
use std::collections::HashMap;

pub struct RedisPlugin;

impl Plugin for RedisPlugin {
    fn choice(&self) -> PluginChoice {
        PluginChoice::Redis
    }

    fn name(&self) -> &'static str {
        "redis"
    }

    fn description(&self) -> &'static str {
        "Redis connection pool and caching helper layer via deadpool-redis"
    }

    fn dependencies(&self) -> &'static str {
        r#"deadpool-redis = { version = "0.19", features = ["rt_tokio_1"] }
redis = { version = "0.28", features = ["tokio-comp"] }
"#
    }

    fn template_files(&self) -> Vec<(&'static str, &'static str)> {
        vec![("plugins/redis/src/cache.rs.jinja", "src/cache.rs")]
    }

    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()> {
        if let Some(state_content) = files.get_mut("src/state.rs") {
            *state_content = AnchorPatcher::inject_before(
                state_content,
                AnchorPatcher::STATE_ANCHOR,
                "pub redis_pool: Option<deadpool_redis::Pool>,",
            );
        }

        if let Some(lib_content) = files.get_mut("src/lib.rs") {
            *lib_content = AnchorPatcher::inject_before(
                lib_content,
                AnchorPatcher::IMPORTS_ANCHOR,
                "pub mod cache;",
            );
        }

        if let Some(cargo_content) = files.get_mut("Cargo.toml") {
            *cargo_content =
                AnchorPatcher::append_to_dependencies(cargo_content, self.dependencies());
        }

        Ok(())
    }
}
