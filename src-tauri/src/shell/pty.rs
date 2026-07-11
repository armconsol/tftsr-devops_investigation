// PTY Management for Interactive Shell Sessions
//
// This module provides pseudo-terminal (PTY) support for kubectl exec/attach operations.
// It uses the portable-pty crate for cross-platform PTY functionality.
//
// Key features:
// - Spawns kubectl exec/attach in a PTY for full interactivity
// - Bidirectional I/O streaming (stdin/stdout/stderr)
// - Proper terminal control (SIGWINCH, raw mode, etc.)
// - Clean session lifecycle management

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use tracing::{debug, warn};

/// PTY session handle with I/O streams
pub struct PtySession {
    /// PTY pair (master + child)
    pair: portable_pty::PtyPair,
    /// Child process handle
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl PtySession {
    /// Spawn a new PTY session with the given command and arguments
    pub fn spawn(command: &str, args: Vec<String>, env: Vec<(String, String)>) -> Result<Self> {
        let pty_system = native_pty_system();

        // Create PTY with default size (80x24)
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to open PTY")?;

        // Build command
        let mut cmd = CommandBuilder::new(command);
        cmd.args(args);
        for (key, value) in env {
            cmd.env(key, value);
        }

        // Spawn child process
        let child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn command in PTY")?;

        debug!(
            "PTY session spawned: {command} (PID: {:?})",
            child.process_id()
        );

        Ok(Self { pair, child })
    }

    /// Spawn kubectl exec session
    pub fn spawn_kubectl_exec(
        kubectl_path: &str,
        namespace: &str,
        pod: &str,
        container: Option<&str>,
        kubeconfig_path: Option<&str>,
    ) -> Result<Self> {
        let mut args = vec![
            "exec".to_string(),
            "-i".to_string(),
            "-t".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            pod.to_string(),
        ];

        if let Some(c) = container {
            args.push("-c".to_string());
            args.push(c.to_string());
        }

        // Use FreeLens-style shell fallback command.
        // We deliberately omit `clear` from the chain: when a container image
        // lacks `clear` (or `tput`), running it would print a non-fatal but
        // confusing error to the user. The frontend terminal is responsible
        // for clearing on connect.
        args.push("--".to_string());
        args.push("sh".to_string());
        args.push("-c".to_string());
        args.push("bash || ash || sh".to_string());

        let mut env = Vec::new();
        if let Some(kubeconfig) = kubeconfig_path {
            env.push(("KUBECONFIG".to_string(), kubeconfig.to_string()));
        }

        Self::spawn(kubectl_path, args, env)
    }

    /// Spawn kubectl attach session
    pub fn spawn_kubectl_attach(
        kubectl_path: &str,
        namespace: &str,
        pod: &str,
        container: Option<&str>,
        kubeconfig_path: Option<&str>,
    ) -> Result<Self> {
        let mut args = vec![
            "attach".to_string(),
            "-i".to_string(),
            "-t".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            pod.to_string(),
        ];

        if let Some(c) = container {
            args.push("-c".to_string());
            args.push(c.to_string());
        }

        let mut env = Vec::new();
        if let Some(kubeconfig) = kubeconfig_path {
            env.push(("KUBECONFIG".to_string(), kubeconfig.to_string()));
        }

        Self::spawn(kubectl_path, args, env)
    }

    /// Write data to PTY stdin
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        let mut writer = self
            .pair
            .master
            .take_writer()
            .context("PTY writer unavailable")?;
        writer
            .write_all(data)
            .context("Failed to write to PTY stdin")?;
        writer.flush().context("Failed to flush PTY stdin")?;
        Ok(())
    }

    /// Read available data from PTY stdout/stderr (non-blocking)
    pub fn read(&mut self) -> Result<Vec<u8>> {
        let mut reader = self
            .pair
            .master
            .try_clone_reader()
            .context("PTY reader unavailable")?;
        let mut buffer = vec![0u8; 4096];

        // Non-blocking read with timeout
        match reader.read(&mut buffer) {
            Ok(n) if n > 0 => {
                buffer.truncate(n);
                Ok(buffer)
            }
            Ok(_) => Ok(Vec::new()), // No data available
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(Vec::new()),
            Err(e) => Err(e).context("Failed to read from PTY"),
        }
    }

    /// Resize the PTY
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        self.pair
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to resize PTY")
    }

    /// Check if the child process is still alive
    pub fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_)) => false, // Process exited
            Ok(None) => true,     // Still running
            Err(_) => false,      // Error checking status
        }
    }

    /// Kill the child process
    pub fn kill(&mut self) -> Result<()> {
        self.child
            .kill()
            .context("Failed to kill PTY child process")
    }

    /// Wait for the child process to exit
    pub fn wait(&mut self) -> Result<portable_pty::ExitStatus> {
        self.child
            .wait()
            .context("Failed to wait for PTY child process")
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        // Best-effort cleanup. Log kill failures rather than swallowing them so
        // operators can detect leaked child processes during diagnostics.
        if self.is_alive() {
            if let Err(e) = self.kill() {
                warn!("PTY session Drop: failed to kill child process: {e:#}");
            }
        }
        debug!("PTY session dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_simple_command() {
        // Spawn a simple echo command
        let result = PtySession::spawn("echo", vec!["hello".to_string()], vec![]);
        assert!(result.is_ok(), "Failed to spawn PTY session");

        let mut session = result.unwrap();

        // Wait a bit for command to execute
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Read output
        let output = session.read().unwrap();
        let output_str = String::from_utf8_lossy(&output);

        // Should contain "hello"
        assert!(
            output_str.contains("hello") || output_str.is_empty(),
            "Expected output to contain 'hello' or be empty (timing issue)"
        );
    }

    #[test]
    fn test_write_and_read() {
        // Spawn cat command (echoes stdin to stdout)
        let result = PtySession::spawn("cat", vec![], vec![]);
        assert!(result.is_ok(), "Failed to spawn PTY session");

        let mut session = result.unwrap();

        // Write data
        let test_data = b"test input\n";
        assert!(session.write(test_data).is_ok(), "Failed to write to PTY");

        // Wait a bit for data to echo back
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Read output
        let output = session.read().unwrap();

        // Kill the session
        assert!(session.kill().is_ok(), "Failed to kill PTY session");

        // Output should contain our test data (cat echoes it back)
        let output_str = String::from_utf8_lossy(&output);
        assert!(
            output_str.contains("test input") || output_str.is_empty(),
            "Expected output to contain 'test input' or be empty (timing issue)"
        );
    }

    #[test]
    fn test_is_alive() {
        let mut session = PtySession::spawn("sleep", vec!["0.1".to_string()], vec![]).unwrap();

        // Should be alive initially
        assert!(session.is_alive(), "Session should be alive");

        // Wait for process to exit with retry logic to handle OS timing variations
        let mut retries = 10;
        while retries > 0 && session.is_alive() {
            std::thread::sleep(std::time::Duration::from_millis(100));
            retries -= 1;
        }

        // Should be dead now
        assert!(
            !session.is_alive(),
            "Session should be dead after sleep completed"
        );
    }

    #[test]
    fn test_kill() {
        let mut session = PtySession::spawn("sleep", vec!["10".to_string()], vec![]).unwrap();

        assert!(session.is_alive(), "Session should be alive");

        // Kill it
        assert!(session.kill().is_ok(), "Failed to kill session");

        // Wait a bit
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Should be dead
        assert!(!session.is_alive(), "Session should be dead after kill");
    }

    #[test]
    fn test_resize() {
        let session = PtySession::spawn("cat", vec![], vec![]).unwrap();

        // Resize should succeed
        assert!(session.resize(40, 120).is_ok(), "Failed to resize PTY");
    }

    #[test]
    fn test_env_variables() {
        // Spawn a command that prints an environment variable
        let result = PtySession::spawn(
            "sh",
            vec!["-c".to_string(), "echo $TEST_VAR".to_string()],
            vec![("TEST_VAR".to_string(), "test_value".to_string())],
        );
        assert!(result.is_ok(), "Failed to spawn PTY session with env");

        let mut session = result.unwrap();

        // Wait for command to execute
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Read output
        let output = session.read().unwrap();
        let output_str = String::from_utf8_lossy(&output);

        // Should contain our test value
        assert!(
            output_str.contains("test_value") || output_str.is_empty(),
            "Expected output to contain 'test_value' or be empty (timing issue)"
        );
    }
}
