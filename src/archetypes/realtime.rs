use super::Archetype;
use crate::cli::TemplateChoice;
use crate::error::Result;
use crate::scaffold::patcher::AnchorPatcher as P;
use std::collections::HashMap;

/// WebSocket + Server-Sent Events streaming backed by a broadcast hub in `AppState`.
pub struct RealtimeArchetype;

impl Archetype for RealtimeArchetype {
    fn choice(&self) -> TemplateChoice {
        TemplateChoice::Realtime
    }

    fn summary(&self) -> &'static str {
        "Realtime streaming: WebSocket /ws, Server-Sent Events /events, POST /publish, demo page"
    }

    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()> {
        P::patch(files, "src/lib.rs", P::IMPORTS_ANCHOR, "pub mod realtime;")?;
        P::patch(
            files,
            "src/state.rs",
            P::STATE_ANCHOR,
            "/// Broadcast hub: `state.hub.publish(..)` reaches every WebSocket and SSE client.\npub hub: crate::realtime::Hub,",
        )?;
        P::patch(
            files,
            "src/state.rs",
            P::STATE_SETUP_ANCHOR,
            "let hub = crate::realtime::Hub::default();",
        )?;
        P::patch(files, "src/state.rs", P::STATE_INIT_ANCHOR, "hub,")?;
        P::patch(
            files,
            "src/router.rs",
            P::ROUTE_ANCHOR,
            ".merge(crate::realtime::routes(state.hub.clone()))",
        )?;
        P::patch(
            files,
            "Cargo.toml",
            P::DEPS_ANCHOR,
            r#"futures-util = { version = "0.3", default-features = false, features = ["sink", "std"] }
tokio-stream = { version = "0.1", features = ["sync"] }
"#,
        )?;
        P::patch(
            files,
            "Cargo.toml",
            P::DEV_DEPS_ANCHOR,
            r#"tokio-tungstenite = "0.30"
tower = { version = "0.5", features = ["util"] }
"#,
        )?;
        Ok(())
    }
}
