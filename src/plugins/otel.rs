use super::Plugin;
use crate::cli::PluginChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher;
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
        "OpenTelemetry distributed tracing exporter & Prometheus /metrics endpoint"
    }

    fn dependencies(&self) -> &'static str {
        r#"metrics = "0.24"
metrics-exporter-prometheus = "0.16"
"#
    }

    fn template_files(&self) -> Vec<(&'static str, &'static str)> {
        vec![("plugins/otel/src/telemetry.rs.jinja", "src/telemetry.rs")]
    }

    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()> {
        if let Some(router_content) = files.get_mut("src/router.rs") {
            *router_content = AnchorPatcher::inject_before(
                router_content,
                AnchorPatcher::ROUTE_ANCHOR,
                ".route(\"/metrics\", axum::routing::get(crate::telemetry::metrics_handler))",
            );
        }

        if let Some(lib_content) = files.get_mut("src/lib.rs") {
            *lib_content = AnchorPatcher::inject_before(
                lib_content,
                AnchorPatcher::IMPORTS_ANCHOR,
                "pub mod telemetry;",
            );
        }

        if let Some(cargo_content) = files.get_mut("Cargo.toml") {
            *cargo_content =
                AnchorPatcher::append_to_dependencies(cargo_content, self.dependencies());
        }

        Ok(())
    }
}
