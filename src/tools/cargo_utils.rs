use anyhow::{Result, bail};
use std::{
    collections::HashMap,
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

/// Helper to create a cargo command with optional toolchain and environment variables
pub fn create_cargo_command(
    cargo_args: &[&str],
    toolchain: Option<&str>,
    env_vars: Option<&HashMap<String, String>>,
) -> Command {
    let mut cmd = if let Some(toolchain) = toolchain {
        let mut cmd = Command::new("rustup");
        cmd.args(["run", toolchain, "cargo"]);
        cmd.args(cargo_args);
        cmd
    } else {
        let mut cmd = Command::new("cargo");
        cmd.args(cargo_args);
        cmd
    };

    // Apply environment variables if provided
    if let Some(env_map) = env_vars {
        for (key, value) in env_map {
            cmd.env(key, value);
        }
    }

    cmd
}

/// Wrap a command with `script` to provide a PTY, required for espflash monitor mode
/// which uses crossterm for terminal input handling
pub fn wrap_command_for_pty(cmd: &mut Command, project_path: &PathBuf) {
    let program = cmd.get_program().to_string_lossy().to_string();
    let args = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ");

    let combined = format!("{} {}", program, args);
    let combined_lower = combined.to_lowercase();

    // Check if this command involves espflash operations
    // espflash uses crossterm which requires a TTY for monitor mode
    // espflash is typically invoked via cargo runner for embedded projects
    let needs_pty = combined_lower.contains("espflash")
        || combined_lower.contains("flash")
        || combined_lower.contains("monitor")
        || (combined_lower.contains("cargo") && combined_lower.contains("run"));

    if needs_pty {
        // Check if project has a .cargo/config.toml with runner configured
        let cargo_config = project_path.join(".cargo/config.toml");
        let cargo_config_exists = cargo_config.exists();

        if cargo_config_exists {
            *cmd = Command::new("script");
            cmd.args(&["-q", "-c", &combined, "/dev/null"]);
        }
    }
}

/// Execute a cargo command and format the output for MCP response
pub fn execute_cargo_command(
    mut cmd: Command,
    project_path: &PathBuf,
    command_name: &str,
    timeout_secs: Option<u64>,
) -> Result<String> {
    cmd.current_dir(project_path);

    // Capture output for display
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let timeout_duration = timeout_secs.map(Duration::from_secs);

    let output = match timeout_duration {
        Some(timeout) => {
            // Wait for child with timeout
            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => break Ok(status),
                    Ok(None) => {
                        if start.elapsed() > timeout {
                            // Kill the child and return timeout error
                            let _ = child.kill();
                            let _ = child.wait();
                            bail!(
                                "❌ Command timed out after {} seconds\n",
                                timeout_secs.unwrap()
                            );
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => break Err(e),
                }
            }
        }
        None => child.wait(),
    }?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Read output from pipes
    let mut stdout_reader = std::io::BufReader::new(stdout);
    let mut stdout_bytes = Vec::new();
    stdout_reader.read_to_end(&mut stdout_bytes)?;

    let mut stderr_reader = std::io::BufReader::new(stderr);
    let mut stderr_bytes = Vec::new();
    stderr_reader.read_to_end(&mut stderr_bytes)?;

    let stdout_str = String::from_utf8_lossy(&stdout_bytes);
    let stderr_str = String::from_utf8_lossy(&stderr_bytes);

    let mut result = format!("=== {command_name} ===\n");
    result.push_str(&format!(
        "📁 Working directory: {}\n",
        project_path.display()
    ));
    result.push_str(&format!("🔧 Command: {}\n\n", format_command(&cmd)));

    if output.success() {
        result.push_str("✅ Command completed successfully\n\n");
    } else {
        result.push_str(&format!(
            "❌ Command failed with exit code: {}\n\n",
            output.code().unwrap_or(-1)
        ));
    }

    if !stdout_str.is_empty() {
        result.push_str("📤 STDOUT:\n");
        result.push_str(&stdout_str);
        if !stdout_str.ends_with('\n') {
            result.push('\n');
        }
        result.push('\n');
    }

    if !stderr_str.is_empty() {
        result.push_str("📤 STDERR:\n");
        result.push_str(&stderr_str);
        if !stderr_str.ends_with('\n') {
            result.push('\n');
        }
        result.push('\n');
    }

    if stdout_str.is_empty() && stderr_str.is_empty() {
        result.push_str("ℹ️  No output produced\n");
    }

    Ok(result)
}

/// Format a command for display
fn format_command(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy();
    let args = cmd
        .get_args()
        .map(|arg| shell_escape(&arg.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ");

    if args.is_empty() {
        program.to_string()
    } else {
        format!("{program} {args}")
    }
}

/// Simple shell escaping for display purposes
fn shell_escape(arg: &str) -> String {
    if arg.contains(' ') || arg.contains('"') || arg.contains('\'') || arg.contains('\\') {
        format!("{arg:?}") // Uses Rust's debug escaping
    } else {
        arg.to_string()
    }
}
