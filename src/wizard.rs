use crate::cli::{DbChoice, PluginChoice};
use crate::error::{NovaError, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use std::path::PathBuf;

pub struct WizardAnswers {
    pub name: String,
    pub db: DbChoice,
    pub plugins: Vec<PluginChoice>,
    pub out: PathBuf,
}

pub fn run_wizard(initial_name: Option<String>, default_out: PathBuf) -> Result<WizardAnswers> {
    println!(
        "{}",
        style("⚡ NovaStack Interactive Scaffolder").bold().cyan()
    );
    println!(
        "{}",
        style("Let's configure your production-ready Rust web backend.\n").dim()
    );

    let theme = ColorfulTheme::default();

    // 1. Project name prompt
    let name: String = match initial_name {
        Some(n) => n,
        None => Input::with_theme(&theme)
            .with_prompt("Project name")
            .default("my-api".to_string())
            .validate_with(|input: &String| validate_project_name(input))
            .interact_text()
            .map_err(|_| NovaError::PromptCancelled)?,
    };

    // 2. Database prompt
    let db_options = [
        "PostgreSQL (Recommended: SQLx + connection pooling & migrations)",
        "SQLite (Lightweight file-based or in-memory SQLx)",
        "None (Stateless / in-memory only)",
    ];

    let db_idx = Select::with_theme(&theme)
        .with_prompt("Select primary database backend")
        .items(&db_options)
        .default(0)
        .interact()
        .map_err(|_| NovaError::PromptCancelled)?;

    let db = match db_idx {
        0 => DbChoice::Postgres,
        1 => DbChoice::Sqlite,
        _ => DbChoice::None,
    };

    // 3. Plugins prompt
    let plugin_options = [
        "Authentication (JWT, password hashing via argon2, auth middleware)",
        "Observability (OpenTelemetry OTLP traces & Prometheus /metrics)",
        "Redis Cache (Connection pool & caching layers via deadpool-redis)",
        "Background Workers (Async task queue runner & supervisor)",
    ];

    let plugin_selections = MultiSelect::with_theme(&theme)
        .with_prompt("Select feature plugins (Space to toggle, Enter to confirm)")
        .items(&plugin_options)
        .interact()
        .map_err(|_| NovaError::PromptCancelled)?;

    let mut plugins = Vec::new();
    for idx in plugin_selections {
        match idx {
            0 => plugins.push(PluginChoice::Auth),
            1 => plugins.push(PluginChoice::Otel),
            2 => plugins.push(PluginChoice::Redis),
            3 => plugins.push(PluginChoice::Workers),
            _ => {}
        }
    }

    Ok(WizardAnswers {
        name,
        db,
        plugins,
        out: default_out,
    })
}

pub fn validate_project_name(name: &str) -> std::result::Result<(), String> {
    if name.is_empty() {
        return Err("Project name cannot be empty".to_string());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "Project name can only contain letters, numbers, hyphens, and underscores".to_string(),
        );
    }
    if name.starts_with('-') || name.starts_with('_') {
        return Err("Project name cannot start with a hyphen or underscore".to_string());
    }
    Ok(())
}
