use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, serde::Serialize)]
pub struct SudoOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub exit_code: Option<i32>,
}

/// Execute a command via sudo, passing the password via stdin (never via cmdline args).
/// `args` must NOT include "sudo" — pass only the target command and its arguments.
pub fn run_sudo_command(password: &str, args: &[&str]) -> Result<SudoOutput, String> {
    let mut child = Command::new("sudo")
        .arg("-S") // read password from stdin
        .arg("-p")
        .arg("") // suppress prompt text
        .arg("--") // end of sudo options — prevents injection
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn sudo: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "{password}")
            .map_err(|e| format!("Failed to write password to stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for sudo: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = strip_sudo_password_prompt(&String::from_utf8_lossy(&output.stderr));

    Ok(SudoOutput {
        stdout,
        stderr,
        success: output.status.success(),
        exit_code: output.status.code(),
    })
}

/// Strip "[sudo] password for ..." prompt lines from stderr before logging.
fn strip_sudo_password_prompt(text: &str) -> String {
    text.lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            !lower.contains("[sudo] password") && !lower.starts_with("password:")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Like run_sudo_command but writes a sanitized audit entry first.
/// The password is NEVER included in audit details.
pub fn run_sudo_command_audited(
    password: &str,
    args: &[&str],
    db: &rusqlite::Connection,
) -> Result<SudoOutput, String> {
    let sanitized_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let details = serde_json::json!({
        "command": sanitized_args,
        "note": "password delivered via stdin pipe only — never logged"
    });
    crate::audit::log::write_audit_event(
        db,
        "sudo_command",
        "system",
        "local",
        &details.to_string(),
    )
    .map_err(|e| format!("Audit log failed: {e}"))?;

    run_sudo_command(password, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_sudo_password_prompt_removes_prompt_lines() {
        let stderr = "[sudo] password for alice:\nsome other output\nPassword: bad line";
        let cleaned = strip_sudo_password_prompt(stderr);
        assert!(!cleaned.contains("[sudo] password"));
        assert!(!cleaned.contains("Password:"));
        assert!(cleaned.contains("some other output"));
    }

    #[test]
    fn test_strip_sudo_password_prompt_keeps_clean_output() {
        let stderr = "Error: permission denied\nsome warning";
        let cleaned = strip_sudo_password_prompt(stderr);
        assert_eq!(cleaned, "Error: permission denied\nsome warning");
    }

    #[test]
    fn test_run_sudo_command_audited_does_not_log_password() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();

        let _result = run_sudo_command_audited("my-secret-password", &["true"], &conn);
        // result may fail in test environment, but audit log must exist
        let details: String = conn
            .query_row(
                "SELECT details FROM audit_log WHERE action = 'sudo_command' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or_default();

        assert!(
            !details.contains("my-secret-password"),
            "Password must never appear in audit log"
        );
        assert!(details.contains("true"), "Command args should be logged");
    }
}
