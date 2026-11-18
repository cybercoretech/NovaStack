use super::Plugin;
use crate::cli::PluginChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher as P;
use std::collections::HashMap;

pub struct WorkersPlugin;

impl Plugin for WorkersPlugin {
    fn choice(&self) -> PluginChoice {
        PluginChoice::Workers
    }

    fn name(&self) -> &'static str {
        "workers"
    }

    fn description(&self) -> &'static str {
        "Asynchronous background worker supervisor and task processing queue"
    }

    fn dependencies(&self) -> &'static str {
        r#"async-trait = "0.1"
"#
    }

    fn template_files(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "plugins/workers/src/workers/mod.rs.jinja",
                "src/workers/mod.rs",
            ),
            (
                "plugins/workers/src/workers/tasks.rs.jinja",
                "src/workers/tasks.rs",
            ),
        ]
    }

    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()> {
        P::patch(files, "src/lib.rs", P::IMPORTS_ANCHOR, "pub mod workers;")?;
        P::patch(
            files,
            "src/main.rs",
            P::STARTUP_ANCHOR,
            "tokio::spawn(project::workers::start_workers());",
        )?;
        P::patch(files, "Cargo.toml", P::DEPS_ANCHOR, self.dependencies())?;
        Ok(())
    }
}
