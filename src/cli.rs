use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "novastack",
    author = "Cybercore Ecosystem",
    version,
    about = "⚡ Zero-friction production-ready Rust web backend scaffolding",
    long_about = "NovaStack scaffolds modular, highly opinionated, yet customizable Rust web backends.\nEquipped with Axum, SQLx, Tower middleware, structured tracing, and Docker containerization."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scaffold a new production-ready Rust backend project
    New {
        /// Project name or path
        name: Option<String>,

        /// Architectural build template archetype
        #[arg(short = 't', long, value_enum, default_value = "rest")]
        template: TemplateChoice,

        /// Database backend choice
        #[arg(short = 'd', long, value_enum, default_value = "postgres")]
        db: DbChoice,

        /// List of optional plugins to enable (e.g. auth, otel, redis, workers)
        #[arg(short = 'p', long, value_delimiter = ',')]
        plugins: Vec<PluginChoice>,

        /// Destination directory (defaults to current dir)
        #[arg(short = 'o', long, default_value = ".")]
        out: PathBuf,

        /// Simulate scaffolding without writing files to disk
        #[arg(long)]
        dry_run: bool,

        /// Non-interactive mode (uses CLI defaults directly)
        #[arg(long)]
        no_interactive: bool,

        /// Skip post-scaffold hooks (git init, cargo check)
        #[arg(long)]
        skip_hooks: bool,
    },

    /// Add a feature plugin to an existing NovaStack project
    Add {
        /// Plugin to install
        #[arg(value_enum)]
        plugin: PluginChoice,

        /// Path to target project (defaults to current dir)
        #[arg(short = 'p', long, default_value = ".")]
        project_dir: PathBuf,
    },

    /// List all available templates, plugins, and stack components
    List,

    /// Check system dependencies (rustc, cargo, docker, sqlx, just)
    Doctor,

    /// Generate a component inside an existing NovaStack project
    Generate {
        /// Type of component to generate
        #[arg(value_enum)]
        kind: GenerateKind,

        /// Name of the component (e.g. `user` or `UserProfile`)
        name: String,

        /// Path to target project (defaults to current dir)
        #[arg(short = 'p', long, default_value = ".")]
        project_dir: PathBuf,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerateKind {
    /// A new top-level module (`src/<name>/mod.rs`)
    Module,
    /// An Axum handler with its own router, nested at `/<name>`
    Handler,
    /// A service struct (`src/services/<name>.rs`)
    Service,
    /// A serde request/response DTO (`src/dtos/<name>.rs`)
    Dto,
    /// An Axum `from_fn` middleware (`src/middleware/<name>.rs`)
    Middleware,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateChoice {
    /// Production REST API with OpenAPI/Swagger UI docs (utoipa)
    Rest,
    /// Full-stack SSR with Askama templates, HTMX 2.0 & Tailwind CSS
    Htmx,
    /// High-throughput Tonic gRPC microservice with Protocol Buffers
    Grpc,
    /// Realtime WebSocket & Server-Sent Events (SSE) streaming server
    Realtime,
    /// Minimalist barebones nano-service with fast compile times
    Minimal,
}

impl fmt::Display for TemplateChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateChoice::Rest => {
                write!(f, "rest (Production REST API with OpenAPI / Swagger UI)")
            }
            TemplateChoice::Htmx => {
                write!(f, "htmx (Full-Stack SSR with Askama + HTMX + Tailwind)")
            }
            TemplateChoice::Grpc => write!(f, "grpc (High-throughput Tonic gRPC microservice)"),
            TemplateChoice::Realtime => {
                write!(f, "realtime (WebSockets & Server-Sent Events streaming)")
            }
            TemplateChoice::Minimal => write!(f, "minimal (Barebones lightweight nano-service)"),
        }
    }
}


#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DbChoice {
    Postgres,
    Sqlite,
    None,
}

impl fmt::Display for DbChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbChoice::Postgres => write!(f, "PostgreSQL (SQLx + connection pooling)"),
            DbChoice::Sqlite => write!(f, "SQLite (SQLx + embedded file/memory)"),
            DbChoice::None => write!(f, "None (in-memory state only)"),
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginChoice {
    Auth,
    Otel,
    Redis,
    Workers,
}

impl fmt::Display for PluginChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginChoice::Auth => write!(f, "auth (JWT authentication & password hashing)"),
            PluginChoice::Otel => write!(f, "otel (OpenTelemetry traces & Prometheus metrics)"),
            PluginChoice::Redis => write!(f, "redis (Redis connection pool & cache layers)"),
            PluginChoice::Workers => write!(f, "workers (Async background job runner)"),
        }
    }
}
