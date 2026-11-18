use super::Archetype;
use crate::cli::TemplateChoice;
use crate::error::Result;
use std::collections::HashMap;

/// The shared base with a trimmed dependency set (see the `template == "minimal"`
/// conditionals in `templates_shared/`); adds no files of its own.
pub struct MinimalArchetype;

impl Archetype for MinimalArchetype {
    fn choice(&self) -> TemplateChoice {
        TemplateChoice::Minimal
    }

    fn summary(&self) -> &'static str {
        "Bare Axum service: health route, tracing and timeouts, trimmed dependency set"
    }

    fn apply_patches(&self, _files: &mut HashMap<String, String>) -> Result<()> {
        Ok(())
    }
}
