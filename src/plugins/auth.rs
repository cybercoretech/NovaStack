use super::Plugin;
use crate::cli::PluginChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher as P;
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
        "JWT authentication, Argon2 password hashing, and an AuthUser extractor that guards routes"
    }

    fn dependencies(&self) -> &'static str {
        r#"jsonwebtoken = "9.3"
argon2 = "0.5"
password-hash = { version = "0.5", features = ["getrandom"] }
"#
    }

    fn template_files(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("plugins/auth/src/auth/mod.rs.jinja", "src/auth/mod.rs"),
            ("plugins/auth/src/auth/jwt.rs.jinja", "src/auth/jwt.rs"),
            (
                "plugins/auth/src/auth/password.rs.jinja",
                "src/auth/password.rs",
            ),
            (
                "plugins/auth/src/auth/extractor.rs.jinja",
                "src/auth/extractor.rs",
            ),
            (
                "plugins/auth/src/handlers/auth.rs.jinja",
                "src/handlers/auth.rs",
            ),
        ]
    }

    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()> {
        P::patch(files, "src/lib.rs", P::IMPORTS_ANCHOR, "pub mod auth;")?;
        P::patch(
            files,
            "src/handlers/mod.rs",
            P::HANDLERS_ANCHOR,
            "pub mod auth;",
        )?;
        P::patch(
            files,
            "src/router.rs",
            P::ROUTE_ANCHOR,
            ".nest(\"/auth\", crate::handlers::auth::auth_router())",
        )?;

        P::patch(
            files,
            "src/config.rs",
            P::CONFIG_FIELDS_ANCHOR,
            "pub jwt_secret: String,\npub jwt_expiration_hours: i64,",
        )?;
        P::patch(
            files,
            "src/config.rs",
            P::CONFIG_LOAD_ANCHOR,
            r#"let jwt_secret = std::env::var("JWT_SECRET")
    .context("JWT_SECRET environment variable must be set")?;
let jwt_expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
    .unwrap_or_else(|_| "24".to_string())
    .parse::<i64>()
    .context("JWT_EXPIRATION_HOURS must be an integer")?;
"#,
        )?;
        P::patch(
            files,
            "src/config.rs",
            P::CONFIG_INIT_ANCHOR,
            "jwt_secret,\njwt_expiration_hours,",
        )?;

        P::patch_optional(
            files,
            ".env.example",
            P::ENV_ANCHOR,
            "# JWT Authentication (generate a secret with: openssl rand -hex 32)\nJWT_SECRET=change-me-to-a-long-random-secret\nJWT_EXPIRATION_HOURS=24\n\n",
        );

        P::patch(files, "Cargo.toml", P::DEPS_ANCHOR, self.dependencies())?;
        Ok(())
    }
}
