use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Build documentation with cargo doc
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_doc")]
pub struct CargoDoc {
    /// Optional package name to document (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Open the docs in a browser after building
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub open: Option<bool>,

    /// Don't build documentation for dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub no_deps: Option<bool>,

    /// Include private items in the documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub document_private_items: Option<bool>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoDoc {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Build documentation for the project",
                item: Self {
                    package: None,
                    open: None,
                    no_deps: None,
                    document_private_items: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Build documentation and open in browser",
                item: Self {
                    package: None,
                    open: Some(true),
                    no_deps: None,
                    document_private_items: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Build docs without dependencies",
                item: Self {
                    package: None,
                    open: None,
                    no_deps: Some(true),
                    document_private_items: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Build docs including private items",
                item: Self {
                    package: None,
                    open: None,
                    no_deps: None,
                    document_private_items: Some(true),
                    toolchain: None,
                    cargo_env: None,
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoDoc {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;
        
        // Use toolchain from args, session default, or none
        let toolchain = self.toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let mut args = vec!["doc"];
        
        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        if self.open.unwrap_or(false) {
            args.push("--open");
        }

        if self.no_deps.unwrap_or(false) {
            args.push("--no-deps");
        }

        if self.document_private_items.unwrap_or(false) {
            args.push("--document-private-items");
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo doc")
    }
}