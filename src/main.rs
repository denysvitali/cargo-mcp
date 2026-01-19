mod state;
mod tools;

#[cfg(test)]
mod tests;

use anyhow::Result;
use mcplease::server_info;
use state::CargoTools;

const INSTRUCTIONS: &str = "Cargo operations for Rust projects.

Use set_working_directory to set the project directory first, then run cargo commands.

For embedded projects using espflash for flashing/monitoring, commands are
automatically wrapped with `script` to provide a PTY required by espflash's
terminal handling (crossterm requires a TTY for monitor mode).";

fn main() -> Result<()> {
    let mut state = CargoTools::new()?;

    mcplease::run::<tools::Tools, _>(&mut state, server_info!(), Some(INSTRUCTIONS))
}
