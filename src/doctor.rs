use console::{style, Emoji};
use std::process::Command;

static CHECKMARK: Emoji<'_, '_> = Emoji("✔ ", "[x] ");
static CROSS: Emoji<'_, '_> = Emoji("✖ ", "[ ] ");
static WARN: Emoji<'_, '_> = Emoji("⚠ ", "[!] ");

pub struct ToolStatus {
    pub name: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub required: bool,
    pub help: &'static str,
}

pub const TOOLS: &[ToolStatus] = &[
    ToolStatus {
        name: "Rust compiler (rustc)",
        command: "rustc",
        args: &["--version"],
        required: true,
        help: "Install via https://rustup.rs",
    },
    ToolStatus {
        name: "Cargo package manager",
        command: "cargo",
        args: &["--version"],
        required: true,
        help: "Included with Rust toolchain (https://rustup.rs)",
    },
    ToolStatus {
        name: "Git version control",
        command: "git",
        args: &["--version"],
        required: true,
        help: "Install git via your system package manager",
    },
    ToolStatus {
        name: "Docker container engine",
        command: "docker",
        args: &["--version"],
        required: false,
        help: "Install Docker for local PostgreSQL and containerization",
    },
    ToolStatus {
        name: "Docker Compose",
        command: "docker",
        args: &["compose", "version"],
        required: false,
        help: "Included in modern Docker releases (Docker Compose v2)",
    },
    ToolStatus {
        name: "SQLx CLI (cargo-sqlx)",
        command: "cargo-sqlx",
        args: &["sqlx", "--version"],
        required: false,
        help:
            "Install via `cargo install sqlx-cli --no-default-features --features postgres,sqlite`",
    },
    ToolStatus {
        name: "Just task runner",
        command: "just",
        args: &["--version"],
        required: false,
        help: "Install via `cargo install just` or system package manager",
    },
];

pub fn run_doctor() {
    println!("{}", style("🔍 NovaStack System Diagnostics").bold().cyan());
    println!(
        "{}",
        style("Checking development environment dependencies...\n").dim()
    );

    let mut all_required_ok = true;

    for tool in TOOLS {
        match which::which(tool.command) {
            Ok(_) => {
                let output = Command::new(tool.command).args(tool.args).output();
                match output {
                    Ok(out) if out.status.success() => {
                        let version = String::from_utf8_lossy(&out.stdout);
                        let first_line = version.lines().next().unwrap_or("ok").trim();
                        println!(
                            "  {} {:<30} {}",
                            style(CHECKMARK).green(),
                            style(tool.name).bold(),
                            style(first_line).dim()
                        );
                    }
                    _ => {
                        print_warning(tool, &mut all_required_ok);
                    }
                }
            }
            Err(_) => {
                print_warning(tool, &mut all_required_ok);
            }
        }
    }

    println!();
    if all_required_ok {
        println!(
            "{}",
            style("✨ Environment is fully primed for NovaStack development!")
                .bold()
                .green()
        );
    } else {
        println!(
            "{}",
            style("⚠️  Some required tools are missing. Please install them before proceeding.")
                .bold()
                .yellow()
        );
    }
}

fn print_warning(tool: &ToolStatus, all_required_ok: &mut bool) {
    if tool.required {
        *all_required_ok = false;
        println!(
            "  {} {:<30} {}",
            style(CROSS).red(),
            style(tool.name).bold().red(),
            style(format!("MISSING (Required) -> {}", tool.help)).yellow()
        );
    } else {
        println!(
            "  {} {:<30} {}",
            style(WARN).yellow(),
            style(tool.name).bold(),
            style(format!("Not found (Optional) -> {}", tool.help)).dim()
        );
    }
}
