use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Automatically fix lint warnings with cargo fix
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_fix")]
pub struct CargoFix {
    /// Optional package name to fix (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Allow dirty working directories (uncommitted changes)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub allow_dirty: Option<bool>,

    /// Allow staged files in git index
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub allow_staged: Option<bool>,

    /// Fix code that may break or change behavior
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub broken_code: Option<bool>,

    /// Apply fixes from the Rust 2018 edition migration
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub edition: Option<bool>,

    /// Apply fixes from idioms lints
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub edition_idioms: Option<bool>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoFix {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Fix lint warnings for the project",
                item: Self {
                    package: None,
                    allow_dirty: None,
                    allow_staged: None,
                    broken_code: None,
                    edition: None,
                    edition_idioms: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Fix warnings allowing dirty working directory",
                item: Self {
                    package: None,
                    allow_dirty: Some(true),
                    allow_staged: None,
                    broken_code: None,
                    edition: None,
                    edition_idioms: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Apply Rust 2018 edition migration fixes",
                item: Self {
                    package: None,
                    allow_dirty: None,
                    allow_staged: None,
                    broken_code: None,
                    edition: Some(true),
                    edition_idioms: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Fix code that may break or change behavior",
                item: Self {
                    package: None,
                    allow_dirty: None,
                    allow_staged: None,
                    broken_code: Some(true),
                    edition: None,
                    edition_idioms: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoFix {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;
        
        // Use toolchain from args, session default, or none
        let toolchain = self.toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let mut args = vec!["fix"];
        
        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        if self.allow_dirty.unwrap_or(false) {
            args.push("--allow-dirty");
        }

        if self.allow_staged.unwrap_or(false) {
            args.push("--allow-staged");
        }

        if self.broken_code.unwrap_or(false) {
            args.push("--broken-code");
        }

        if self.edition.unwrap_or(false) {
            args.push("--edition");
        }

        if self.edition_idioms.unwrap_or(false) {
            args.push("--edition-idioms");
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo fix")
    }
}