use super::Plugin;
use crate::cli::PluginChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher as P;
use std::collections::HashMap;

pub struct OtelPlugin;

impl Plugin for OtelPlugin {
    fn choice(&self) -> PluginChoice {
        PluginChoice::Otel
    }

    fn name(&self) -> &'static str {
        "otel"
    }

    fn description(&self) -> &'static str {
        "Prometheus /metrics endpoint with per-route HTTP request metrics (OTLP trace export planned)"
    }

    fn dependencies(&self) -> &'static str {
        r#"metrics = "0.24"
metrics-exporter-prometheus = { version = "0.16", default-features = false }
"#
    }

    fn template_files(&self) -> Vec<(&'static str, &'static str)> {
        vec![("plugins/otel/src/telemetry.rs.jinja", "src/telemetry.rs")]
    }

    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()> {
        P::patch(files, "src/lib.rs", P::IMPORTS_ANCHOR, "pub mod telemetry;")?;
        P::patch(
            files,
            "src/router.rs",
            P::ROUTE_ANCHOR,
            ".route(\"/metrics\", axum::routing::get(crate::telemetry::metrics_handler))",
        )?;
        P::patch(
            files,
            "src/router.rs",
            P::LAYERS_ANCHOR,
            ".layer(axum::middleware::from_fn(crate::telemetry::track_metrics))",
        )?;
        P::patch(
            files,
            "src/main.rs",
            P::STARTUP_ANCHOR,
            "project::telemetry::init_metrics()?;",
        )?;
        P::patch(files, "Cargo.toml", P::DEPS_ANCHOR, self.dependencies())?;
        Ok(())
    }
}
