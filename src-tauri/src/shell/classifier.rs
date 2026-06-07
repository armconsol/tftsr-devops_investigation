// Command Safety Classifier
//
// Classifies shell commands into three safety tiers:
//   Tier 1 — Auto-execute (read-only, no side effects)
//   Tier 2 — User approval required (potentially mutating)
//   Tier 3 — Always deny (destructive / irreversible)
//
// The pub const arrays below are the single source of truth for both the
// classification logic AND the UI tier-rule listing endpoint.

use serde::{Deserialize, Serialize};

// ── Tier 3: Always deny ───────────────────────────────────────────────────────
// Single-token entries only; multi-word strings cannot match (parse_single_component
// extracts only the first whitespace-delimited token as `command`).

pub const TIER3_COMMANDS: &[&str] = &[
    // Linux — filesystem destruction
    "rm",
    "mkfs",
    "mkfs.ext4",
    "mkfs.xfs",
    "mkfs.btrfs",
    "dd",
    "fdisk",
    "parted",
    "cfdisk",
    "sfdisk",
    "gdisk",
    "wipefs",
    "blkdiscard",
    "mkswap",
    // Linux — volume/storage destruction
    "zpool",
    "dmsetup",
    "cryptsetup",
    "vgremove",
    "lvremove",
    "pvremove",
    "mdadm",
    // Linux — process / system control (all destructive without subcommand context)
    "shutdown",
    "reboot",
    "halt",
    "poweroff",
    "kill",
    "pkill",
    "killall",
    "init",
    // Windows — filesystem / disk destruction
    "format",
    "diskpart",
    "del",
    "erase",
    "sdelete",
    "cipher",
    "bcdedit",
    "bootrec",
    "dism",
    "wimlib-imaging",
    // Windows — destructive item removal
    "remove-item",
    "clear-item",
    // Windows PowerShell — system/process control
    "stop-computer",
    "restart-computer",
    "stop-process",
    "stop-service",
    "start-process",
    "start-service",
    "start-computer",
    "suspend-process",
    "suspend-service",
    "resume-process",
    "resume-service",
    "wait-process",
    "wait-service",
    "wait-computer",
    "invoke-item",
    "clear-recyclebin",
    "clear-host",
    // Windows PowerShell — object/task destruction
    "unregister-scheduledtask",
    "remove-scheduledtask",
    "remove-job",
    "remove-runspace",
    "remove-appdomain",
    "remove-pssession",
    "remove-module",
    "uninstall-package",
    "uninstall-module",
    "remove-wmiobject",
    "remove-itemproperty",
];

// ── Tier 1: kubectl read-only subcommands ─────────────────────────────────────

pub const TIER1_KUBECTL_SUBCOMMANDS: &[&str] = &[
    "get",
    "describe",
    "logs",
    "explain",
    "api-resources",
    "api-versions",
    "cluster-info",
    "top",
    "version",
];

// ── Tier 2: kubectl mutating subcommands ──────────────────────────────────────

pub const TIER2_KUBECTL_SUBCOMMANDS: &[&str] = &[
    "apply",
    "delete",
    "edit",
    "scale",
    "rollout",
    "drain",
    "cordon",
    "uncordon",
    "exec",
    "cp",
    "port-forward",
    "patch",
    "create",
    "replace",
    "label",
    "annotate",
    "taint",
    "set",
];

// ── Tier 1: systemctl read-only subcommands ───────────────────────────────────
// (Previously dead code — systemctl was not in tier1_general so the special
//  case check inside that block never executed.)

pub const TIER1_SYSTEMCTL_SUBCOMMANDS: &[&str] = &[
    "status",
    "is-active",
    "is-enabled",
    "list-units",
    "list-unit-files",
];

// ── Tier 2: systemctl mutating subcommands ────────────────────────────────────

pub const TIER2_SYSTEMCTL_SUBCOMMANDS: &[&str] = &[
    "restart",
    "stop",
    "start",
    "enable",
    "disable",
    "reload",
    "daemon-reload",
    "mask",
    "unmask",
    "preset",
];

// ── Tier 1: Proxmox read-only subcommands ─────────────────────────────────────

pub const TIER1_PROXMOX_SUBCOMMANDS: &[&str] = &["status", "get"];

// ── Tier 2: Proxmox mutating subcommands ──────────────────────────────────────

pub const TIER2_PROXMOX_SUBCOMMANDS: &[&str] =
    &["migrate", "create", "set", "delete", "start", "stop"];

// ── Tier 1: General read-only commands ───────────────────────────────────────

pub const TIER1_GENERAL_COMMANDS: &[&str] = &[
    // Linux — file / process inspection
    "cat",
    "grep",
    "ls",
    "find",
    "df",
    "free",
    "ps",
    "ss",
    "netstat",
    "journalctl",
    "echo",
    "pwd",
    "whoami",
    "date",
    "uptime",
    "head",
    "tail",
    "less",
    "more",
    "wc",
    "sort",
    "uniq",
    "cut",
    "tr",
    "test",
    "stat",
    "file",
    "readlink",
    "which",
    "whereis",
    "type",
    "help",
    "man",
    "info",
    // Linux — hardware / storage inspection (read-only)
    "dmidecode",
    "lscpu",
    "lsblk",
    "lshw",
    "lspci",
    "lsusb",
    "hwinfo",
    "smartctl",
    "vgdisplay",
    "lvdisplay",
    "pvdisplay",
    // Linux — network inspection (read-only)
    "dig",
    "host",
    "nslookup",
    "ldapsearch",
    // Windows cmd — read-only
    "dir",
    "findstr",
    "fc",
    "comp",
    "driverquery",
    "systeminfo",
    "ver",
    "ipconfig",
    "ping",
    "tracert",
    "nbtstat",
    "pathping",
    "hostname",
    "quser",
    "qwinsta",
    "wevtutil",
    "chcp",
    // Windows PowerShell — get-* (read-only by convention)
    "get-process",
    "get-service",
    "get-eventlog",
    "get-childitem",
    "get-content",
    "get-date",
    "get-location",
    "get-physicalmemory",
    "get-processor",
    "get-volume",
    "get-partition",
    "get-disk",
    "get-computerinfo",
    "get-windowsfeature",
    "get-module",
    "get-command",
    "get-wmiobject",
    "get-ciminstance",
    "get-counter",
    "get-netadapter",
    "get-netipaddress",
    "get-netroute",
    "get-nettcpconnection",
    "get-netfirewallrule",
    "get-itemproperty",
    "get-alias",
    "get-variable",
    "get-psdrive",
    "get-clipboard",
    "get-scheduledtask",
    "get-job",
    "get-runspace",
];

// ── Tier 2: General mutating / connecting commands ────────────────────────────

pub const TIER2_GENERAL_COMMANDS: &[&str] = &[
    // Linux — file / permission mutation
    "ssh",
    "scp",
    "rsync",
    "chmod",
    "chown",
    "mv",
    "cp",
    "awk",
    "sed",
    "sudo",
    "ln",
    "touch",
    "truncate",
    "mktemp",
    "mkdir",
    "rmdir",
    "mount",
    "umount",
    // Linux — network with side effects
    "curl",
    "wget",
    "ftp",
    "sftp",
    "tftp",
    // LDAP — mutating directory operations (Bug 3 fix: removed from Tier 1)
    "ldapmodify",
    "ldapdelete",
    "ldapadd",
    "ldapbind",
    "ldifde",
    "csvde",
    // Windows cmd — mutating file / system operations
    "move",
    "ren",
    "rename",
    "copy",
    "xcopy",
    "robocopy",
    "mklink",
    "attrib",
    "cacls",
    "icacls",
    "takeown",
    "setx",
    "reg",
    "schtasks",
    "pushd",
    "popd",
    "subst",
    // Windows PowerShell — set-* / new-* (mutating)
    "set-item",
    "set-itemproperty",
    "set-location",
    "set-variable",
    "set-alias",
    "set-executionpolicy",
    "set-service",
    "set-date",
    "new-item",
    "new-itemproperty",
    "register-scheduledtask",
    "enable-scheduledtask",
    "disable-scheduledtask",
    "new-scheduledtask",
    "new-module",
    "import-module",
    "import-pssession",
    "new-pssession",
    "enter-pssession",
    "new-job",
    "wait-job",
    "receive-job",
];

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone)]
pub enum CommandTier {
    Tier1, // Auto-execute
    Tier2, // Requires approval
    Tier3, // Always deny
}

impl CommandTier {
    pub fn to_tier_number(&self) -> i32 {
        match self {
            CommandTier::Tier1 => 1,
            CommandTier::Tier2 => 2,
            CommandTier::Tier3 => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandComponent {
    pub command: String,
    pub subcommand: Option<String>,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct ClassificationResult {
    pub tier: CommandTier,
    pub components: Vec<CommandComponent>,
    pub reasoning: String,
    pub risk_factors: Vec<String>,
}

/// Structured tier rules returned by `get_rules()` for UI consumption.
/// Each field is a Vec of the actual command/subcommand strings the classifier
/// uses, so additions to the const arrays automatically appear in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierRules {
    pub tier1_kubectl: Vec<String>,
    pub tier1_systemctl: Vec<String>,
    pub tier1_proxmox: Vec<String>,
    pub tier1_general: Vec<String>,
    pub tier2_kubectl: Vec<String>,
    pub tier2_systemctl: Vec<String>,
    pub tier2_proxmox: Vec<String>,
    pub tier2_general: Vec<String>,
    pub tier3: Vec<String>,
}

pub struct CommandClassifier;

impl Default for CommandClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandClassifier {
    pub fn new() -> Self {
        CommandClassifier
    }

    /// Return the live tier rule data for UI rendering.
    /// Derives directly from the module-level const arrays — stays in sync
    /// automatically whenever the arrays are updated.
    pub fn get_rules() -> ClassifierRules {
        ClassifierRules {
            tier1_kubectl: TIER1_KUBECTL_SUBCOMMANDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            tier1_systemctl: TIER1_SYSTEMCTL_SUBCOMMANDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            tier1_proxmox: TIER1_PROXMOX_SUBCOMMANDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            tier1_general: TIER1_GENERAL_COMMANDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            tier2_kubectl: TIER2_KUBECTL_SUBCOMMANDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            tier2_systemctl: TIER2_SYSTEMCTL_SUBCOMMANDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            tier2_proxmox: TIER2_PROXMOX_SUBCOMMANDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            tier2_general: TIER2_GENERAL_COMMANDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            tier3: TIER3_COMMANDS.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn classify(&self, command: &str) -> ClassificationResult {
        let mut risk_factors = Vec::new();

        // Check for command substitution
        if command.contains("$(") || command.contains('`') {
            risk_factors.push("command_substitution".to_string());
        }

        // Parse into components (pipe, &&, ||, ;)
        let components = Self::parse_command_structure(command);

        let mut highest_tier = CommandTier::Tier1;
        let mut reasoning_parts = Vec::new();

        for component in &components {
            let tier =
                self.classify_single_command(&component.command, component.subcommand.as_deref());

            match tier {
                CommandTier::Tier3 => {
                    highest_tier = CommandTier::Tier3;
                    reasoning_parts.push(format!(
                        "'{}' is a destructive operation",
                        component.command
                    ));
                }
                CommandTier::Tier2 => {
                    if highest_tier != CommandTier::Tier3 {
                        highest_tier = CommandTier::Tier2;
                        reasoning_parts
                            .push(format!("'{}' is a mutating operation", component.command));
                    }
                }
                CommandTier::Tier1 => {
                    if reasoning_parts.is_empty() && highest_tier == CommandTier::Tier1 {
                        reasoning_parts.push("read-only operations only".to_string());
                    }
                }
            }
        }

        // Command substitution escalates to Tier 2 minimum
        if !risk_factors.is_empty() && highest_tier == CommandTier::Tier1 {
            highest_tier = CommandTier::Tier2;
            reasoning_parts.push("contains command substitution".to_string());
        }

        let reasoning = if reasoning_parts.is_empty() {
            "safe read-only command".to_string()
        } else {
            reasoning_parts.join(", ")
        };

        ClassificationResult {
            tier: highest_tier,
            components,
            reasoning,
            risk_factors,
        }
    }

    fn classify_single_command(&self, command: &str, subcommand: Option<&str>) -> CommandTier {
        // ── Tier 3 check (runs first — most restrictive wins) ─────────────────
        if TIER3_COMMANDS.contains(&command) {
            // Special case: bootrec — only certain subcommands are truly destructive
            if command == "bootrec" {
                if let Some(sub) = subcommand {
                    if matches!(sub, "/fixmbr" | "/fixboot" | "/rebuildbcd") {
                        return CommandTier::Tier3;
                    }
                    // Unknown bootrec subcommand — conservative Tier 3
                    return CommandTier::Tier3;
                }
            }
            // Special case: cipher — only /w: (wipe free space) is destructive
            if command == "cipher" {
                if let Some(args) = subcommand {
                    if args.contains("/w:") {
                        return CommandTier::Tier3;
                    }
                }
                // cipher without /w: is read-only (certificate operations) — Tier 2
                return CommandTier::Tier2;
            }
            return CommandTier::Tier3;
        }

        // ── kubectl ───────────────────────────────────────────────────────────
        if command == "kubectl" {
            if let Some(sub) = subcommand {
                if TIER1_KUBECTL_SUBCOMMANDS.contains(&sub) {
                    return CommandTier::Tier1;
                }
                if TIER2_KUBECTL_SUBCOMMANDS.contains(&sub) {
                    return CommandTier::Tier2;
                }
                // Unknown kubectl subcommand — conservative Tier 2
                return CommandTier::Tier2;
            }
            return CommandTier::Tier2;
        }

        // ── systemctl (Bug 2 fix: moved here — was dead code inside tier1_general) ──
        if command == "systemctl" {
            if let Some(sub) = subcommand {
                if TIER1_SYSTEMCTL_SUBCOMMANDS.contains(&sub) {
                    return CommandTier::Tier1;
                }
            }
            // restart, stop, start, enable, disable, reload, etc. → Tier 2
            return CommandTier::Tier2;
        }

        // ── Proxmox ───────────────────────────────────────────────────────────
        if matches!(command, "pvecm" | "pvesh" | "qm") {
            if let Some(sub) = subcommand {
                if TIER1_PROXMOX_SUBCOMMANDS.contains(&sub) {
                    return CommandTier::Tier1;
                }
                if TIER2_PROXMOX_SUBCOMMANDS.contains(&sub) {
                    return CommandTier::Tier2;
                }
            }
            return CommandTier::Tier2;
        }

        // ── Tier 1 general ────────────────────────────────────────────────────
        // PowerShell get-* pattern catches any get-<noun> not explicitly listed
        if command.starts_with("get-") {
            return CommandTier::Tier1;
        }

        if TIER1_GENERAL_COMMANDS.contains(&command) {
            return CommandTier::Tier1;
        }

        // ── Tier 2 general ────────────────────────────────────────────────────
        if TIER2_GENERAL_COMMANDS.contains(&command) {
            return CommandTier::Tier2;
        }

        // Default: unknown commands require approval
        CommandTier::Tier2
    }

    fn parse_command_structure(command: &str) -> Vec<CommandComponent> {
        let mut components = Vec::new();
        let mut current_cmd = String::new();
        let mut chars = command.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '|' {
                if chars.peek() == Some(&'|') {
                    chars.next();
                    if !current_cmd.trim().is_empty() {
                        components.push(Self::parse_single_component(current_cmd.trim()));
                    }
                    current_cmd.clear();
                } else {
                    if !current_cmd.trim().is_empty() {
                        components.push(Self::parse_single_component(current_cmd.trim()));
                    }
                    current_cmd.clear();
                }
            } else if ch == '&' && chars.peek() == Some(&'&') {
                chars.next();
                if !current_cmd.trim().is_empty() {
                    components.push(Self::parse_single_component(current_cmd.trim()));
                }
                current_cmd.clear();
            } else if ch == ';' {
                if !current_cmd.trim().is_empty() {
                    components.push(Self::parse_single_component(current_cmd.trim()));
                }
                current_cmd.clear();
            } else {
                current_cmd.push(ch);
            }
        }

        if !current_cmd.trim().is_empty() {
            components.push(Self::parse_single_component(current_cmd.trim()));
        }

        components
    }

    fn parse_single_component(cmd_str: &str) -> CommandComponent {
        let parts: Vec<&str> = cmd_str.split_whitespace().collect();

        if parts.is_empty() {
            return CommandComponent {
                command: String::new(),
                subcommand: None,
                args: Vec::new(),
            };
        }

        let command = parts[0].to_string();
        let mut subcommand = None;
        let mut args = Vec::new();

        // Commands for which the second token is the subcommand
        if matches!(
            command.as_str(),
            "kubectl" | "pvecm" | "pvesh" | "qm" | "systemctl"
        ) {
            if parts.len() > 1 {
                subcommand = Some(parts[1].to_string());
                args = parts[2..].iter().map(|s| s.to_string()).collect();
            }
        } else {
            args = parts[1..].iter().map(|s| s.to_string()).collect();
        }

        CommandComponent {
            command,
            subcommand,
            args,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier1_kubectl_get() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl get pods");
        assert_eq!(result.tier, CommandTier::Tier1);
        assert_eq!(result.components.len(), 1);
        assert!(result.reasoning.contains("read-only") || result.reasoning.contains("safe"));
    }

    #[test]
    fn test_tier1_kubectl_describe() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl describe pod nginx");
        assert_eq!(result.tier, CommandTier::Tier1);
    }

    #[test]
    fn test_tier1_kubectl_logs() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl logs nginx-pod");
        assert_eq!(result.tier, CommandTier::Tier1);
    }

    #[test]
    fn test_tier2_kubectl_delete() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl delete pod nginx");
        assert_eq!(result.tier, CommandTier::Tier2);
        assert!(result.reasoning.contains("delete") || result.reasoning.contains("mutating"));
    }

    #[test]
    fn test_tier2_kubectl_apply() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl apply -f deployment.yaml");
        assert_eq!(result.tier, CommandTier::Tier2);
    }

    #[test]
    fn test_tier2_kubectl_scale() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl scale deployment nginx --replicas=5");
        assert_eq!(result.tier, CommandTier::Tier2);
    }

    #[test]
    fn test_tier3_rm_rf() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("rm -rf /");
        assert_eq!(result.tier, CommandTier::Tier3);
        assert!(result.reasoning.contains("destructive") || result.reasoning.contains("dangerous"));
    }

    #[test]
    fn test_tier3_shutdown() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("shutdown -h now");
        assert_eq!(result.tier, CommandTier::Tier3);
    }

    #[test]
    fn test_pipe_safe_to_safe() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl get pods | grep nginx");
        assert_eq!(result.tier, CommandTier::Tier1);
        assert_eq!(result.components.len(), 2);
    }

    #[test]
    fn test_pipe_safe_to_danger() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl get pods | kubectl delete -f -");
        assert_eq!(result.tier, CommandTier::Tier2);
    }

    #[test]
    fn test_command_substitution() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl get $(dangerous)");
        assert_eq!(result.tier, CommandTier::Tier2);
        assert!(result
            .risk_factors
            .contains(&"command_substitution".to_string()));
    }

    #[test]
    fn test_backtick_substitution() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("kubectl get `whoami`");
        assert_eq!(result.tier, CommandTier::Tier2);
        assert!(result
            .risk_factors
            .contains(&"command_substitution".to_string()));
    }

    #[test]
    fn test_logical_and_operator() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("ls /tmp && rm -rf /tmp/test");
        assert_eq!(result.tier, CommandTier::Tier3);
    }

    #[test]
    fn test_logical_or_operator() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("test -f file || rm -rf /tmp");
        assert_eq!(result.tier, CommandTier::Tier3);
    }

    #[test]
    fn test_semicolon_separator() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("cat file.txt; echo done");
        assert_eq!(result.tier, CommandTier::Tier1);
    }

    #[test]
    fn test_proxmox_tier1() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("pvecm status");
        assert_eq!(result.tier, CommandTier::Tier1);
    }

    #[test]
    fn test_proxmox_tier2() {
        let classifier = CommandClassifier::new();
        let result = classifier.classify("qm migrate 100 node2");
        assert_eq!(result.tier, CommandTier::Tier2);
    }

    #[test]
    fn test_general_safe_commands() {
        let classifier = CommandClassifier::new();
        let safe_commands = vec![
            "cat /var/log/syslog",
            "grep error log.txt",
            "ls -la",
            "df -h",
        ];
        for cmd in safe_commands {
            let result = classifier.classify(cmd);
            assert_eq!(
                result.tier,
                CommandTier::Tier1,
                "Command '{cmd}' should be Tier 1"
            );
        }
    }

    #[test]
    fn test_tier2_network_commands() {
        let classifier = CommandClassifier::new();
        let tier2_commands = vec!["ssh user@host", "scp file.txt user@host:"];
        for cmd in tier2_commands {
            let result = classifier.classify(cmd);
            assert_eq!(
                result.tier,
                CommandTier::Tier2,
                "Command '{cmd}' should be Tier 2"
            );
        }
    }

    #[test]
    fn test_windows_tier1_readonly_commands() {
        let classifier = CommandClassifier::new();
        let tier1_commands = vec![
            "dir",
            "findstr pattern file.txt",
            "ipconfig",
            "ping 127.0.0.1",
            "tracert 127.0.0.1",
            "netstat",
            "whoami",
            "systeminfo",
            "ver",
            "hostname",
            "get-process",
            "get-service",
            "get-eventlog -logname System",
            "get-childitem",
            "get-content file.txt",
            "get-date",
            "get-location",
            "get-volume",
            "get-partition",
            "get-disk",
            "get-computerinfo",
        ];
        for cmd in tier1_commands {
            let result = classifier.classify(cmd);
            assert_eq!(
                result.tier,
                CommandTier::Tier1,
                "Command '{cmd}' should be Tier 1"
            );
        }
    }

    #[test]
    fn test_windows_tier2_mutating_commands() {
        let classifier = CommandClassifier::new();
        let tier2_commands = vec![
            "move file.txt newfile.txt",
            "ren file.txt newfile.txt",
            "copy file.txt dest.txt",
            "xcopy file.txt dest.txt",
            "robocopy source dest",
            "attrib +r file.txt",
            "icacls file.txt /grant user:F",
            "schtasks /create /tn test /tr test.exe",
            "reg add HKLM\\Software\\Test",
            "setx VAR value",
            "new-item -path C:\\test",
            "set-itemproperty -path HKLM:\\Software\\Test -name Test -value 1",
            "sudo",
            "new-scheduledtask -action (new-scheduledtaskaction -execute notepad)",
            "register-scheduledtask -taskname test -action (new-scheduledtaskaction -execute notepad)",
            "curl -X POST http://example.com",
            "wget --post-data test http://example.com",
        ];
        for cmd in tier2_commands {
            let result = classifier.classify(cmd);
            assert_eq!(
                result.tier,
                CommandTier::Tier2,
                "Command '{cmd}' should be Tier 2"
            );
        }
    }

    #[test]
    fn test_windows_tier3_destructive_commands() {
        let classifier = CommandClassifier::new();
        let tier3_commands = vec![
            "format C: /q",
            "del file.txt",
            "erase file.txt",
            "sdelete C:\\test",
            "bootrec /fixmbr",
            "bootrec /fixboot",
            "diskpart",
            "remove-item -recurse C:\\test",
            "clear-recyclebin",
            "stop-computer",
            "restart-computer",
            "stop-process -name notepad",
            "stop-service nginx",
            "uninstall-module -name PowerShellGet",
            "uninstall-package -name Package",
            "unregister-scheduledtask -taskname test",
            "dd if=/dev/zero of=/dev/sda",
            "mkfs.ext4 /dev/sda1",
            "clear-host",
        ];
        for cmd in tier3_commands {
            let result = classifier.classify(cmd);
            assert_eq!(
                result.tier,
                CommandTier::Tier3,
                "Command '{cmd}' should be Tier 3"
            );
        }
    }

    // ── Bug fix tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_kill_is_tier3() {
        let c = CommandClassifier::new();
        assert_eq!(c.classify("kill -9 1234").tier, CommandTier::Tier3);
        assert_eq!(c.classify("kill 1234").tier, CommandTier::Tier3);
    }

    #[test]
    fn test_pkill_is_tier3() {
        let c = CommandClassifier::new();
        assert_eq!(c.classify("pkill -9 nginx").tier, CommandTier::Tier3);
        assert_eq!(c.classify("pkill nginx").tier, CommandTier::Tier3);
    }

    #[test]
    fn test_killall_is_tier3() {
        let c = CommandClassifier::new();
        assert_eq!(c.classify("killall nginx").tier, CommandTier::Tier3);
    }

    #[test]
    fn test_init_is_tier3() {
        let c = CommandClassifier::new();
        assert_eq!(c.classify("init 0").tier, CommandTier::Tier3);
        assert_eq!(c.classify("init 6").tier, CommandTier::Tier3);
    }

    #[test]
    fn test_systemctl_status_is_tier1() {
        let c = CommandClassifier::new();
        assert_eq!(
            c.classify("systemctl status nginx").tier,
            CommandTier::Tier1
        );
        assert_eq!(
            c.classify("systemctl is-active nginx").tier,
            CommandTier::Tier1
        );
        assert_eq!(
            c.classify("systemctl is-enabled nginx").tier,
            CommandTier::Tier1
        );
        assert_eq!(c.classify("systemctl list-units").tier, CommandTier::Tier1);
        assert_eq!(
            c.classify("systemctl list-unit-files").tier,
            CommandTier::Tier1
        );
    }

    #[test]
    fn test_systemctl_mutating_is_tier2() {
        let c = CommandClassifier::new();
        assert_eq!(
            c.classify("systemctl restart nginx").tier,
            CommandTier::Tier2
        );
        assert_eq!(c.classify("systemctl stop nginx").tier, CommandTier::Tier2);
        assert_eq!(c.classify("systemctl start nginx").tier, CommandTier::Tier2);
        assert_eq!(
            c.classify("systemctl enable nginx").tier,
            CommandTier::Tier2
        );
        assert_eq!(
            c.classify("systemctl disable nginx").tier,
            CommandTier::Tier2
        );
    }

    #[test]
    fn test_ldapmodify_is_tier2() {
        let c = CommandClassifier::new();
        assert_eq!(
            c.classify("ldapmodify -f changes.ldif").tier,
            CommandTier::Tier2
        );
    }

    #[test]
    fn test_ldapdelete_is_tier2() {
        let c = CommandClassifier::new();
        assert_eq!(
            c.classify("ldapdelete cn=user,dc=example,dc=com").tier,
            CommandTier::Tier2
        );
    }

    #[test]
    fn test_ldapadd_is_tier2() {
        let c = CommandClassifier::new();
        assert_eq!(c.classify("ldapadd -f new.ldif").tier, CommandTier::Tier2);
    }

    #[test]
    fn test_ldapsearch_is_tier1() {
        let c = CommandClassifier::new();
        assert_eq!(
            c.classify("ldapsearch -x -b dc=example,dc=com").tier,
            CommandTier::Tier1
        );
    }

    #[test]
    fn test_get_rules_returns_all_tiers() {
        let rules = CommandClassifier::get_rules();
        assert!(
            !rules.tier1_kubectl.is_empty(),
            "tier1 kubectl should not be empty"
        );
        assert!(
            !rules.tier1_general.is_empty(),
            "tier1 general should not be empty"
        );
        assert!(
            !rules.tier1_systemctl.is_empty(),
            "tier1 systemctl should not be empty"
        );
        assert!(
            !rules.tier2_kubectl.is_empty(),
            "tier2 kubectl should not be empty"
        );
        assert!(
            !rules.tier2_general.is_empty(),
            "tier2 general should not be empty"
        );
        assert!(!rules.tier3.is_empty(), "tier3 should not be empty");
    }

    #[test]
    fn test_get_rules_tier1_contains_expected_kubectl() {
        let rules = CommandClassifier::get_rules();
        assert!(rules.tier1_kubectl.iter().any(|s| s == "get"));
        assert!(rules.tier1_kubectl.iter().any(|s| s == "logs"));
    }

    #[test]
    fn test_get_rules_tier3_contains_expected() {
        let rules = CommandClassifier::get_rules();
        assert!(rules.tier3.iter().any(|s| s == "rm"));
        assert!(rules.tier3.iter().any(|s| s == "kill"));
        assert!(rules.tier3.iter().any(|s| s == "init"));
    }
}
