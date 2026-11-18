use clap::Parser;
use console::style;
use novastack_core::archetypes::{self, get_archetype};
use novastack_core::cli::{Cli, Commands, PluginChoice};
use novastack_core::cli_generate::run_generate;
use novastack_core::doctor::run_doctor;
use novastack_core::error::Result;
use novastack_core::scaffold::{ScaffoldConfig, ScaffoldEngine};
use novastack_core::wizard::run_wizard;
use std::process;

fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        eprintln!("\n{} {}", style("error:").bold().red(), err);
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::New {
            name,
            db,
            plugins,
            out,
            dry_run,
            no_interactive,
            skip_hooks,
            template,
        } => {
            let (project_name, selected_db, selected_plugins, selected_template, target_dir) =
                if !no_interactive && name.is_none() {
                    let answers = run_wizard(name, out)?;
                    let target = answers.out.join(&answers.name);
                    (
                        answers.name,
                        answers.db,
                        answers.plugins,
                        answers.template,
                        target,
                    )
                } else {
                    let proj_name = name.unwrap_or_else(|| "my-api".to_string());
                    let target = out.join(&proj_name);
                    (proj_name, db, plugins, template, target)
                };

            let config = ScaffoldConfig {
                project_name,
                target_dir,
                db: selected_db,
                plugins: selected_plugins,
                dry_run,
                skip_hooks,
                template: selected_template,
            };

            ScaffoldEngine::execute(&config)?;
        }

        Commands::Add {
            plugin,
            project_dir,
        } => {
            ScaffoldEngine::add_plugin(&project_dir, plugin)?;
        }

        Commands::List => {
            print_plugins_list();
        }

        Commands::Doctor => {
            run_doctor();
        }

        Commands::Generate {
            kind,
            name,
            project_dir,
        } => {
            run_generate(kind, &name, &project_dir)?;
        }
    }

    Ok(())
}

fn print_plugins_list() {
    println!(
        "{}",
        style("⚡ NovaStack Available Plugins & Components")
            .bold()
            .cyan()
    );
    println!(
        "{}",
        style("Use `novastack add <plugin>` inside an existing project.\n").dim()
    );

    let plugins = [
        (
            PluginChoice::Auth,
            "JWT token generation/verification, Argon2 password hashing, and user authentication handlers.",
        ),
        (
            PluginChoice::Otel,
            "Prometheus metrics endpoint (`/metrics`) with per-route HTTP request metrics.",
        ),
        (
            PluginChoice::Redis,
            "Redis connection pool (`deadpool-redis`) and caching helper layer.",
        ),
        (
            PluginChoice::Workers,
            "Asynchronous background worker supervisor and task processing queue.",
        ),
    ];

    for (choice, desc) in plugins {
        println!(
            "  {} {}",
            style(format!("{:<10}", format!("{:?}", choice).to_lowercase()))
                .bold()
                .green(),
            desc
        );
    }

    println!(
        "\n{}",
        style("Build Templates (`novastack new -t <template>`):").bold()
    );
    for choice in archetypes::ALL {
        println!(
            "  {} {}",
            style(format!("{:<10}", choice.as_str())).bold().magenta(),
            get_archetype(choice).summary()
        );
    }

    println!("\n{}", style("Supported Database Backends:").bold());
    println!(
        "  {} Production PostgreSQL with SQLx connection pooling and embedded migrations",
        style("postgres  ").bold().yellow()
    );
    println!(
        "  {} Lightweight SQLx file-based or in-memory database",
        style("sqlite    ").bold().yellow()
    );
    println!(
        "  {} Stateless / in-memory state only (no database driver)",
        style("none      ").bold().yellow()
    );
}
