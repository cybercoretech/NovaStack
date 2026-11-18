use super::Plugin;
use crate::cli::PluginChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher as P;
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
"#
    }

    fn template_files(&self) -> Vec<(&'static str, &'static str)> {
        vec![("plugins/redis/src/cache.rs.jinja", "src/cache.rs")]
    }

    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()> {
        P::patch(files, "src/lib.rs", P::IMPORTS_ANCHOR, "pub mod cache;")?;

        P::patch(
            files,
            "src/config.rs",
            P::CONFIG_FIELDS_ANCHOR,
            "pub redis_url: String,",
        )?;
        P::patch(
            files,
            "src/config.rs",
            P::CONFIG_LOAD_ANCHOR,
            r#"let redis_url = std::env::var("REDIS_URL")
    .unwrap_or_else(|_| "redis://localhost:6379".to_string());
"#,
        )?;
        P::patch(files, "src/config.rs", P::CONFIG_INIT_ANCHOR, "redis_url,")?;

        P::patch(
            files,
            "src/state.rs",
            P::STATE_ANCHOR,
            "pub redis: deadpool_redis::Pool,",
        )?;
        P::patch(
            files,
            "src/state.rs",
            P::STATE_SETUP_ANCHOR,
            "let redis = crate::cache::create_redis_pool(&config.redis_url)?;",
        )?;
        P::patch(files, "src/state.rs", P::STATE_INIT_ANCHOR, "redis,")?;

        P::patch_optional(
            files,
            ".env.example",
            P::ENV_ANCHOR,
            "# Redis Configuration\nREDIS_URL=redis://localhost:6379\n\n",
        );

        P::patch(files, "Cargo.toml", P::DEPS_ANCHOR, self.dependencies())?;
        Ok(())
    }
}
