//! Machine-readable catalogue of everything NovaStack can generate
//! (`novastack list --json`). The project website is built from this output, so it never
//! drifts from the code.

use crate::archetypes::{self, get_archetype};
use crate::cli::{DbChoice, GenerateKind, PluginChoice, TemplateChoice};
use crate::plugins::get_plugin;
use crate::scaffold::engine::GENERATED_RUST_VERSION;
use clap::ValueEnum;
use serde_json::{json, Value};

/// The full catalogue as JSON.
pub fn catalog() -> Value {
    let templates: Vec<Value> = archetypes::ALL
        .iter()
        .map(|&choice| {
            json!({
                "name": choice.as_str(),
                "summary": get_archetype(choice).summary(),
                "default": choice == TemplateChoice::default(),
            })
        })
        .collect();

    let plugins: Vec<Value> = PluginChoice::value_variants()
        .iter()
        .map(|&choice| {
            let plugin = get_plugin(choice);
            json!({
                "name": plugin.name(),
                "description": plugin.description(),
                "dependencies": crate_names(plugin.dependencies()),
                "files": plugin
                    .template_files()
                    .into_iter()
                    .map(|(_, target)| target)
                    .collect::<Vec<_>>(),
            })
        })
        .collect();

    let databases: Vec<Value> = DbChoice::value_variants()
        .iter()
        .map(|choice| {
            json!({
                "name": value_name(choice),
                "description": choice.to_string(),
            })
        })
        .collect();

    let generate: Vec<Value> = GenerateKind::value_variants()
        .iter()
        .map(|kind| {
            json!({
                "kind": kind.as_str(),
                "description": value_help(kind),
            })
        })
        .collect();

    json!({
        "novastack_version": env!("CARGO_PKG_VERSION"),
        "generated_rust_version": GENERATED_RUST_VERSION,
        "templates": templates,
        "plugins": plugins,
        "databases": databases,
        "generate": generate,
    })
}

/// Crate names from a Cargo.toml dependency snippet (`name = ...` lines).
fn crate_names(snippet: &str) -> Vec<&str> {
    snippet
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('=').map(|(name, _)| name.trim()))
        .collect()
}

fn value_name(value: &impl ValueEnum) -> String {
    value
        .to_possible_value()
        .map(|v| v.get_name().to_string())
        .unwrap_or_default()
}

fn value_help(value: &impl ValueEnum) -> String {
    value
        .to_possible_value()
        .and_then(|v| v.get_help().map(ToString::to_string))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_lists_everything() {
        let c = catalog();
        assert_eq!(
            c["templates"].as_array().unwrap().len(),
            archetypes::ALL.len()
        );
        assert_eq!(
            c["plugins"].as_array().unwrap().len(),
            PluginChoice::value_variants().len()
        );
        assert_eq!(c["databases"][0]["name"], "postgres");
        assert!(c["generate"][0]["description"]
            .as_str()
            .unwrap()
            .contains("module"));
    }

    #[test]
    fn crate_names_skip_comments_and_blank_lines() {
        let snippet = "argon2 = \"0.5\"\n# comment\n\npassword-hash = { version = \"0.5\" }\n";
        assert_eq!(crate_names(snippet), vec!["argon2", "password-hash"]);
    }
}
