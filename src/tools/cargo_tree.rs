use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Display a tree visualization of a crate's dependencies
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_tree")]
pub struct CargoTree {
    /// Optional package name to show tree for (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Show features for each package
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub features: Option<bool>,

    /// Show dependencies as inverted tree (what depends on what)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub invert: Option<bool>,

    /// Show duplicated dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub duplicates: Option<bool>,

    /// Prune the given package from the tree
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub prune: Option<String>,

    /// Maximum display depth of the dependency tree
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub depth: Option<u32>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoTree {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Show dependency tree for the project",
                item: Self {
                    package: None,
                    features: None,
                    invert: None,
                    duplicates: None,
                    prune: None,
                    depth: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Show dependency tree with features",
                item: Self {
                    package: None,
                    features: Some(true),
                    invert: None,
                    duplicates: None,
                    prune: None,
                    depth: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Show inverted dependency tree",
                item: Self {
                    package: None,
                    features: None,
                    invert: Some(true),
                    duplicates: None,
                    prune: None,
                    depth: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Show duplicated dependencies only",
                item: Self {
                    package: None,
                    features: None,
                    invert: None,
                    duplicates: Some(true),
                    prune: None,
                    depth: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoTree {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;
        
        // Use toolchain from args, session default, or none
        let toolchain = self.toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let mut args = vec!["tree"];
        
        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        if self.features.unwrap_or(false) {
            args.push("--features");
        }

        if self.invert.unwrap_or(false) {
            args.push("--invert");
        }

        if self.duplicates.unwrap_or(false) {
            args.push("--duplicates");
        }

        if let Some(ref prune_pkg) = self.prune {
            args.extend_from_slice(&["--prune", prune_pkg]);
        }

        let depth_str;
        if let Some(depth) = self.depth {
            depth_str = depth.to_string();
            args.extend_from_slice(&["--depth", &depth_str]);
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo tree")
    }
}