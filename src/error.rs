use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, NovaError>;

#[derive(Error, Debug)]
pub enum NovaError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Template rendering error in '{template}': {source}")]
    TemplateRender {
        template: String,
        #[source]
        source: minijinja::Error,
    },

    #[error("Invalid project name '{name}': {reason}")]
    InvalidProjectName { name: String, reason: String },

    #[error("Target directory '{path}' already exists and is not empty")]
    DirectoryNotEmpty { path: PathBuf },

    #[error("Project root not found in '{path}'. Is this a NovaStack project?")]
    NotANovaStackProject { path: PathBuf },

    #[error("Plugin '{plugin}' is already installed in this project")]
    PluginAlreadyInstalled { plugin: String },

    #[error("Unknown plugin '{plugin}'")]
    UnknownPlugin { plugin: String },

    #[error("Patch anchor '{anchor}' not found in target file '{path}'")]
    AnchorNotFound { anchor: String, path: PathBuf },

    #[error("Hook execution failed for '{command}': {message}")]
    HookFailed { command: String, message: String },

    #[error("Interactive prompt cancelled by user")]
    PromptCancelled,

    #[error("{0}")]
    Other(String),
}
