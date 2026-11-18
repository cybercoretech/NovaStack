use super::Archetype;
use crate::cli::TemplateChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher as P;
use std::collections::HashMap;

/// Tonic gRPC service (with health and reflection) served on the same port as the HTTP
/// routes, so `/health`, `/metrics` and the plugins keep working.
pub struct GrpcArchetype;

impl Archetype for GrpcArchetype {
    fn choice(&self) -> TemplateChoice {
        TemplateChoice::Grpc
    }

    fn summary(&self) -> &'static str {
        "Tonic gRPC service + health + reflection, sharing one port with the HTTP routes"
    }

    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()> {
        P::patch(files, "src/lib.rs", P::IMPORTS_ANCHOR, "pub mod grpc;")?;
        P::patch(
            files,
            "src/router.rs",
            P::ROUTE_ANCHOR,
            ".merge(crate::grpc::routes())",
        )?;
        P::patch(
            files,
            "Cargo.toml",
            P::DEPS_ANCHOR,
            r#"tonic = "0.14"
tonic-prost = "0.14"
tonic-health = "0.14"
tonic-reflection = "0.14"
prost = "0.14"
"#,
        )?;
        P::append(
            files,
            "Cargo.toml",
            r#"[build-dependencies]
tonic-prost-build = "0.14"
# Bundled protoc: no system Protocol Buffers install needed
protoc-bin-vendored = "3"
"#,
        )?;
        Ok(())
    }
}
