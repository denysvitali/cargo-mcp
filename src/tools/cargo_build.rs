use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Build the project with cargo build
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_build")]
pub struct CargoBuild {
    /// Optional package name to build (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Build in release mode
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub release: Option<bool>,

    /// Space-separated list of features to activate
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub features: Option<String>,

    /// Activate all available features
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub all_features: Option<bool>,

    /// Do not activate the `default` feature
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub no_default_features: Option<bool>,

    /// Build for the target triple
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub target: Option<String>,

    /// Number of parallel jobs, defaults to # of CPUs
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub jobs: Option<u32>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoBuild {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Build the project in debug mode",
                item: Self {
                    package: None,
                    release: None,
                    features: None,
                    all_features: None,
                    no_default_features: None,
                    target: None,
                    jobs: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Build the project in release mode",
                item: Self {
                    package: None,
                    release: Some(true),
                    features: None,
                    all_features: None,
                    no_default_features: None,
                    target: None,
                    jobs: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Build a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    release: None,
                    features: None,
                    all_features: None,
                    no_default_features: None,
                    target: None,
                    jobs: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Build with nightly toolchain",
                item: Self {
                    package: None,
                    release: None,
                    features: None,
                    all_features: None,
                    no_default_features: None,
                    target: None,
                    jobs: None,
                    toolchain: Some("nightly".into()),
                    cargo_env: None,
                },
            },
            Example {
                description: "Build with specific features",
                item: Self {
                    package: None,
                    release: None,
                    features: Some("serde json".into()),
                    all_features: None,
                    no_default_features: None,
                    target: None,
                    jobs: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Build for specific target",
                item: Self {
                    package: None,
                    release: None,
                    features: None,
                    all_features: None,
                    no_default_features: None,
                    target: Some("x86_64-pc-windows-gnu".into()),
                    jobs: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoBuild {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;
        
        // Use toolchain from args, session default, or none
        let toolchain = self.toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));


        let mut args = vec!["build"];
        
        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        if self.release.unwrap_or(false) {
            args.push("--release");
        }

        if let Some(ref features) = self.features {
            args.extend_from_slice(&["--features", features]);
        }

        if self.all_features.unwrap_or(false) {
            args.push("--all-features");
        }

        if self.no_default_features.unwrap_or(false) {
            args.push("--no-default-features");
        }

        if let Some(ref target) = self.target {
            args.extend_from_slice(&["--target", target]);
        }

        let jobs_str;
        if let Some(jobs) = self.jobs {
            jobs_str = jobs.to_string();
            args.extend_from_slice(&["--jobs", &jobs_str]);
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo build")
    }
}
