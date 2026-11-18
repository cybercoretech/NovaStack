use super::Plugin;
use crate::cli::PluginChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher;
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
        if let Some(lib_content) = files.get_mut("src/lib.rs") {
            *lib_content = AnchorPatcher::inject_before(
                lib_content,
                AnchorPatcher::IMPORTS_ANCHOR,
                "pub mod workers;",
            );
        }

        if let Some(cargo_content) = files.get_mut("Cargo.toml") {
            *cargo_content =
                AnchorPatcher::append_to_dependencies(cargo_content, self.dependencies());
        }

        Ok(())
    }
}
