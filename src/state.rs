use anyhow::{Result, anyhow};
use fieldwork::Fieldwork;
use mcplease::session::SessionStore;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug, Formatter},
    path::PathBuf,
};

/// Session data specific to cargo operations
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CargoSessionData {
    /// Default toolchain to use for cargo commands (e.g., "stable", "nightly", "1.70.0")
    default_toolchain: Option<String>,
}

/// Cargo tools with session support
#[derive(Fieldwork)]
#[fieldwork(get, get_mut)]
pub struct CargoTools {
    /// Private session store for cargo-specific state
    session_store: SessionStore<CargoSessionData>,
}

impl Debug for CargoTools {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("CargoTools")
            .field("session_store", &self.session_store)
            .finish()
    }
}

impl CargoTools {
    /// Create a new CargoTools instance
    pub fn new() -> Result<Self> {
        // Private session store for cargo-specific state
        let mut private_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        private_path.push(".ai-tools");
        private_path.push("sessions");
        private_path.push("cargo-mcp.json");
        let session_store = SessionStore::new(Some(private_path))?;

        let mut tools = Self { session_store };

        // Check for default toolchain from environment variable
        if let Ok(toolchain) = std::env::var("CARGO_MCP_DEFAULT_TOOLCHAIN")
            && !toolchain.is_empty()
        {
            log::info!("Setting default toolchain from CARGO_MCP_DEFAULT_TOOLCHAIN: {toolchain}");
            tools.set_default_toolchain(Some(toolchain))?;
        }

        Ok(tools)
    }

    /// Get the default toolchain
    pub fn get_default_toolchain(&mut self) -> Result<Option<String>> {
        let session_data = self.session_store.get_or_create("default")?;
        Ok(session_data.default_toolchain.clone())
    }

    /// Set the default toolchain
    pub fn set_default_toolchain(&mut self, toolchain: Option<String>) -> Result<()> {
        self.session_store.update("default", |data| {
            data.default_toolchain = toolchain;
        })
    }

    /// Check if the current working directory is a Rust project
    pub fn ensure_rust_project(&mut self) -> Result<PathBuf> {
        let context = std::env::current_dir()
            .map_err(|e| anyhow!("Failed to get current working directory: {}", e))?;

        let cargo_toml = context.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(anyhow!(
                "Not a Rust project: Cargo.toml not found in {}",
                context.display()
            ));
        }

        Ok(context)
    }
}
