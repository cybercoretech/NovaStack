//! Build archetypes (`novastack new -t <archetype>`).
//!
//! Every archetype shares the base project in `templates_shared/`. An archetype adds its own
//! files from `templates/<archetype>/` (rendered when they end in `.jinja`, copied verbatim
//! otherwise) and wires them in through the same anchor comments plugins use, so plugins,
//! `novastack add` and `novastack generate` keep working on every archetype.

use crate::cli::TemplateChoice;
use crate::error::Result;
use std::collections::HashMap;

pub mod grpc;
pub mod htmx;
pub mod minimal;
pub mod realtime;
pub mod rest;

pub trait Archetype: Send + Sync {
    fn choice(&self) -> TemplateChoice;
    /// One-line description shown by `novastack list`.
    fn summary(&self) -> &'static str;
    /// Patches the rendered base project (deps, module declarations, routes, state).
    fn apply_patches(&self, files: &mut HashMap<String, String>) -> Result<()>;
}

pub fn get_archetype(choice: TemplateChoice) -> Box<dyn Archetype> {
    match choice {
        TemplateChoice::Rest => Box::new(rest::RestArchetype),
        TemplateChoice::Htmx => Box::new(htmx::HtmxArchetype),
        TemplateChoice::Grpc => Box::new(grpc::GrpcArchetype),
        TemplateChoice::Realtime => Box::new(realtime::RealtimeArchetype),
        TemplateChoice::Minimal => Box::new(minimal::MinimalArchetype),
    }
}

pub const ALL: [TemplateChoice; 5] = [
    TemplateChoice::Rest,
    TemplateChoice::Htmx,
    TemplateChoice::Grpc,
    TemplateChoice::Realtime,
    TemplateChoice::Minimal,
];
