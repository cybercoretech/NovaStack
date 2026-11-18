use super::Archetype;
use crate::cli::TemplateChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher as P;
use std::collections::HashMap;

/// JSON REST API with an OpenAPI document (utoipa) and Swagger UI, plus an example
/// `/api/items` CRUD resource.
pub struct RestArchetype;

impl Archetype for RestArchetype {
    fn choice(&self) -> TemplateChoice {
        TemplateChoice::Rest
    }

    fn summary(&self) -> &'static str {
        "JSON REST API: OpenAPI spec (utoipa), Swagger UI at /docs, example /api/items CRUD"
    }

    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()> {
        P::patch(files, "src/lib.rs", P::IMPORTS_ANCHOR, "pub mod api;")?;
        P::patch(
            files,
            "src/router.rs",
            P::ROUTE_ANCHOR,
            ".merge(crate::api::routes())",
        )?;
        P::patch(
            files,
            "Cargo.toml",
            P::DEPS_ANCHOR,
            r#"utoipa = "6"
utoipa-swagger-ui = { version = "10", features = ["axum", "vendored"] }
"#,
        )?;
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
