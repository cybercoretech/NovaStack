use super::Plugin;
use crate::cli::PluginChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher;
use std::collections::HashMap;

pub struct AuthPlugin;

impl Plugin for AuthPlugin {
    fn choice(&self) -> PluginChoice {
        PluginChoice::Auth
    }

    fn name(&self) -> &'static str {
        "auth"
    }

    fn description(&self) -> &'static str {
        "JWT authentication, password hashing with Argon2, and route guard middleware"
    }

    fn dependencies(&self) -> &'static str {
        r#"jsonwebtoken = "9.3"
argon2 = "0.5"
axum-extra = { version = "0.9", features = ["typed-header"] }
"#
    }

    fn template_files(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("plugins/auth/src/auth/mod.rs.jinja", "src/auth/mod.rs"),
            ("plugins/auth/src/auth/jwt.rs.jinja", "src/auth/jwt.rs"),
            (
                "plugins/auth/src/handlers/auth.rs.jinja",
                "src/handlers/auth.rs",
            ),
        ]
    }

    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()> {
        // Patch src/lib.rs or router
        if let Some(router_content) = files.get_mut("src/router.rs") {
            *router_content = AnchorPatcher::inject_before(
                router_content,
                AnchorPatcher::ROUTE_ANCHOR,
                ".nest(\"/auth\", crate::handlers::auth::auth_router())",
            );
        }

        if let Some(lib_content) = files.get_mut("src/lib.rs") {
            *lib_content = AnchorPatcher::inject_before(
                lib_content,
                AnchorPatcher::IMPORTS_ANCHOR,
                "pub mod auth;",
            );
        }

        if let Some(cargo_content) = files.get_mut("Cargo.toml") {
            *cargo_content =
                AnchorPatcher::append_to_dependencies(cargo_content, self.dependencies());
        }

        Ok(())
    }
}
