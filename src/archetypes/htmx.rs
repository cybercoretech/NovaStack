use super::Archetype;
use crate::cli::TemplateChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher as P;
use std::collections::HashMap;

/// Server-rendered UI: Askama templates, HTMX fragment swaps and Tailwind styling.
pub struct HtmxArchetype;

impl Archetype for HtmxArchetype {
    fn choice(&self) -> TemplateChoice {
        TemplateChoice::Htmx
    }

    fn summary(&self) -> &'static str {
        "Server-rendered UI: Askama templates, HTMX fragment swaps, Tailwind via CDN"
    }

    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()> {
        P::patch(files, "src/lib.rs", P::IMPORTS_ANCHOR, "pub mod web;")?;
        P::patch(
            files,
            "src/router.rs",
            P::ROUTE_ANCHOR,
            ".merge(crate::web::routes(crate::web::TodoStore::from_state(&state)))",
        )?;
        P::patch(files, "Cargo.toml", P::DEPS_ANCHOR, "askama = \"0.16\"\n")?;
        P::patch(
            files,
            "Cargo.toml",
            P::DEV_DEPS_ANCHOR,
            r#"tower = { version = "0.5", features = ["util"] }
http-body-util = "0.1"
"#,
        )?;
        Ok(())
    }
}
