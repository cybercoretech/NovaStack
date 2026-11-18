use crate::cli::{DbChoice, PluginChoice, TemplateChoice};
use crate::error::{NovaError, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use std::path::PathBuf;

pub struct WizardAnswers {
    pub name: String,
    pub template: TemplateChoice,
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

    // 1. Template Archetype Prompt
    let template_options = [
        "REST API (Recommended: Standard JSON API with OpenAPI 3.1 & Swagger UI)",
        "Full-Stack HTMX (Axum + Askama compile-time templates + HTMX + Tailwind)",
        "gRPC Microservice (Tonic + Protocol Buffers high-performance RPC)",
        "Realtime & Streaming (WebSockets, Server-Sent Events & broadcast pub/sub)",
        "Minimal / Nano (Barebones lightweight service with sub-second compile times)",
    ];

    let template_idx = Select::with_theme(&theme)
        .with_prompt("What kind of backend are you building?")
        .items(&template_options)
        .default(0)
        .interact()
        .map_err(|_| NovaError::PromptCancelled)?;

    let template = match template_idx {
        0 => TemplateChoice::Rest,
        1 => TemplateChoice::Htmx,
        2 => TemplateChoice::Grpc,
        3 => TemplateChoice::Realtime,
        _ => TemplateChoice::Minimal,
    };

    // 2. Project name prompt
    let default_name = match template {
        TemplateChoice::Rest => "my-api",
        TemplateChoice::Htmx => "my-app",
        TemplateChoice::Grpc => "my-service",
        TemplateChoice::Realtime => "my-hub",
        TemplateChoice::Minimal => "my-nano",
    };

    let name: String = match initial_name {
        Some(n) => n,
        None => Input::with_theme(&theme)
            .with_prompt("Project name")
            .default(default_name.to_string())
            .validate_with(|input: &String| validate_project_name(input))
            .interact_text()
            .map_err(|_| NovaError::PromptCancelled)?,
    };

    // 3. Database prompt
    let db_options = [
        "PostgreSQL (Recommended: SQLx + connection pooling & migrations)",
        "SQLite (Lightweight file-based or in-memory SQLx)",
        "None (Stateless / in-memory only)",
    ];

    let default_db_idx = match template {
        TemplateChoice::Minimal => 2,
        _ => 0,
    };

    let db_idx = Select::with_theme(&theme)
        .with_prompt("Select primary database backend")
        .items(&db_options)
        .default(default_db_idx)
        .interact()
        .map_err(|_| NovaError::PromptCancelled)?;

    let db = match db_idx {
        0 => DbChoice::Postgres,
        1 => DbChoice::Sqlite,
        _ => DbChoice::None,
    };

    // 4. Feature Plugins prompt (skip for minimal unless requested)
    let plugin_options = [
        "Authentication (JWT, password hashing via argon2, auth middleware)",
        "Observability (OpenTelemetry OTLP traces & Prometheus /metrics)",
        "Redis Cache (Connection pool & caching layers via deadpool-redis)",
        "Background Workers (Async task queue runner & supervisor)",
    ];

    let mut plugins = Vec::new();
    if template != TemplateChoice::Minimal {
        let plugin_selections = MultiSelect::with_theme(&theme)
            .with_prompt("Select feature plugins (Space to toggle, Enter to confirm)")
            .items(&plugin_options)
            .interact()
            .map_err(|_| NovaError::PromptCancelled)?;

        for idx in plugin_selections {
            match idx {
                0 => plugins.push(PluginChoice::Auth),
                1 => plugins.push(PluginChoice::Otel),
                2 => plugins.push(PluginChoice::Redis),
                3 => plugins.push(PluginChoice::Workers),
                _ => {}
            }
        }
    }

    Ok(WizardAnswers {
        name,
        template,
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
