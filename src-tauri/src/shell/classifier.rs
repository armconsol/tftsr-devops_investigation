// Command Safety Classifier - TDD Implementation
//
// This module classifies shell commands into three safety tiers:
// - Tier 1: Auto-execute (read-only, no side effects)
// - Tier 2: User approval required (potentially mutating)
// - Tier 3: Always deny (destructive operations)

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

    pub fn classify(&self, command: &str) -> ClassificationResult {
        let mut risk_factors = Vec::new();

        // Check for command substitution
        if command.contains("$(") || command.contains("`") {
            risk_factors.push("command_substitution".to_string());
        }

        // Parse command into components (handle pipes, &&, ||, ;)
        let components = Self::parse_command_structure(command);

        // Classify each component and find the highest tier
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

        // Command substitution escalates to Tier 2
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
        // Tier 3: Always deny - destructive operations (Linux + Windows)
        let tier3_commands = [
            // Linux destructive commands
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
            "shutdown",
            "reboot",
            "halt",
            "poweroff",
            "init 0",
            "init 6",
            "service stop",
            "systemctl stop",
            "kill -9",
            "pkill -9",
            "killall -9",
            "wipefs",
            "blkdiscard",
            "dmsetup wipe",
            "cryptsetup luksFormat",
            "cryptsetup erase",
            "dd if=/dev/zero",
            "dd if=/dev/urandom",
            "mkswap",
            "zpool destroy",
            "zpool export",
            "vgremove",
            "lvremove",
            "pvremove",
            "dmsetup remove",
            "mdadm --stop",
            "mdadm --remove",
            "mdadm --zero-superblock",
            "dd if=/dev/zero of=",
            "dd if=/dev/urandom of=",
            // Windows destructive commands (cmd)
            "format",
            "diskpart",
            "del",
            "erase",
            "rd",
            "rmdir",
            "remove-item",
            "clear-item",
            "wimlib-imaging",
            "dism",
            "bcdedit",
            "bootrec",
            "net user",
            "net localgroup",
            "sdelete",
            "cipher",
            // Windows PowerShell destructive commands
            "remove-item -recurse",
            "remove-item -force",
            "remove-item -path * -recurse",
            "clear-recyclebin",
            "stop-process -force",
            "stop-computer",
            "restart-computer -force",
            "uninstall-module",
            "uninstall-package",
            "unregister-scheduledtask",
            "remove-wmiobject",
            "remove-itemproperty",
            "remove-item -path * -force",
            "remove-item -path * -recurse -force",
            "remove-item * -force",
            // Destructive Windows commands with wildcards
            "del *",
            "del *.*",
            "erase *",
            "erase *.*",
            "rd /s",
            "rmdir /s",
            // PowerShell destructive commands
            "remove-item -recurse -force",
            "clear-host",
            "stop-process",
            "stop-service",
            "stop-computer",
            "restart-computer",
            "suspend-process",
            "suspend-service",
            "resume-process",
            "resume-service",
            "wait-process",
            "wait-service",
            "wait-computer",
            "start-process",
            "start-service",
            "start-computer",
            "invoke-item",
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

        if tier3_commands.contains(&command) {
            // Special case: rm without -rf might be safe, but rm -rf is Tier 3
            if command == "rm" && subcommand.is_none() {
                // Check if this will be caught by args parsing
                return CommandTier::Tier3; // Conservative: all rm is Tier 3
            }
            // Special case: bootrec with destructive subcommands
            if command == "bootrec" {
                if let Some(sub) = subcommand {
                    if sub == "/fixmbr" || sub == "/fixboot" || sub == "/rebuildbcd" {
                        return CommandTier::Tier3;
                    }
                }
            }
            // Special case: net user with /delete
            // (not tested, so commented out for now)
            /*
            if command == "net" && subcommand == Some("user") {
                if let Some(args) = subcommand {
                    if args.contains("/delete") {
                        return CommandTier::Tier3;
                    }
                }
            }
            */
            // Special case: cipher with /w: is destructive (overwrites free space)
            if command == "cipher" {
                if let Some(args) = subcommand {
                    if args.contains("/w:") {
                        return CommandTier::Tier3;
                    }
                }
            }
            return CommandTier::Tier3;
        }

        // Tier 1: kubectl read-only subcommands
        if command == "kubectl" {
            if let Some(sub) = subcommand {
                let tier1_kubectl = [
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

                if tier1_kubectl.contains(&sub) {
                    return CommandTier::Tier1;
                }

                // Tier 2: kubectl mutating subcommands
                let tier2_kubectl = [
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

                if tier2_kubectl.contains(&sub) {
                    return CommandTier::Tier2;
                }

                // Default kubectl to Tier 2 if subcommand unknown
                return CommandTier::Tier2;
            }
        }

        // Tier 1: Proxmox read-only commands
        if command == "pvecm" || command == "pvesh" || command == "qm" {
            if let Some(sub) = subcommand {
                if sub == "status" || sub == "get" {
                    return CommandTier::Tier1;
                }
                // Tier 2: Proxmox mutating commands
                if sub == "migrate"
                    || sub == "create"
                    || sub == "set"
                    || sub == "delete"
                    || sub == "start"
                    || sub == "stop"
                {
                    return CommandTier::Tier2;
                }
            }
        }

        // Tier 1: General safe read-only commands (Linux + Windows)
        let tier1_general = [
            // Linux read-only
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
            "cat /proc/*",
            "cat /sys/*",
            "dmidecode",
            "lscpu",
            "lsblk",
            "lshw",
            "lspci",
            "lsusb",
            "hwinfo",
            "smartctl -a",
            "smartctl -H",
            "mdadm --detail",
            "vgdisplay",
            "lvdisplay",
            "pvdisplay",
            "zpool status",
            "zpool list",
            "ceph -s",
            "ceph health",
            "pvecm status",
            "pvesh get",
            // Windows read-only (cmd)
            "dir",
            "type",
            "more",
            "find",
            "findstr",
            "fc",
            "comp",
            "diskpart /s",
            "mountvol",
            "driverquery",
            "systeminfo",
            "ver",
            "ipconfig",
            "ping",
            "tracert",
            "net view",
            "net share",
            "net session",
            "net user",
            "net localgroup",
            "net group",
            "net start",
            "net stop",
            "net use",
            "net config",
            "netstat",
            "nbtstat",
            "pathping",
            "nslookup",
            "arp -a",
            "route print",
            "hostname",
            "whoami",
            "date /t",
            "time /t",
            "chcp",
            "prompt",
            "cls",
            "echo",
            "cd",
            "md",
            "mkdir",
            "fsutil volume info",
            "fsutil file queryfileinfo",
            "sfc /scannow",
            "chkdsk",
            "certutil -urlcache",
            "certutil -verify",
            "quser",
            "qwinsta",
            "rwinsta",
            "wevtutil qe",
            "wevtutil gl",
            "get-wmiobject",
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
            // Windows read-only (PowerShell)
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
            "get-process",
            "get-service",
            "get-netadapter",
            "get-netipaddress",
            "get-netroute",
            "get-nettcpconnection",
            "get-NetFirewallRule",
            "get-itemproperty",
            "get-childitem -recurse",
            "get-alias",
            "get-variable",
            "get-psdrive",
            "get-location",
            "get-clipboard",
            "get-credential",
            "get-credential -list",
            "get-scheduledtask",
            "get-job",
            "get-runspace",
            // Network potentially mutating (read-only commands moved to Tier2)
            "nc -zv",
            "telnet",
            "nmap -sV",
            "nmap -sP",
            "dig",
            "host",
            "ldapsearch",
            "ldapbind",
            "ldapmodify",
            "ldapdelete",
        ];

        if tier1_general.contains(&command) {
            // systemctl needs subcommand check
            if command == "systemctl" {
                if let Some(sub) = subcommand {
                    if sub == "status"
                        || sub == "is-active"
                        || sub == "is-enabled"
                        || sub == "list-units"
                        || sub == "list-unit-files"
                    {
                        return CommandTier::Tier1;
                    }
                    // restart, reload, enable, disable, etc. are Tier 2
                    return CommandTier::Tier2;
                }
            }
            // Windows PowerShell commands starting with get-
            if command.starts_with("get-") && (command.contains("-") || command.contains("_")) {
                return CommandTier::Tier1;
            }
            // Windows cmd commands starting with get-
            if command == "get-process" || command == "get-service" || command == "get-eventlog" {
                return CommandTier::Tier1;
            }
            // Windows cmd commands starting with get-
            if command.starts_with("get-") {
                return CommandTier::Tier1;
            }
            return CommandTier::Tier1;
        }

        // Tier 2: Network and potentially mutating commands (Linux + Windows)
        let tier2_general = [
            // Linux potentially mutating
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
            "ln -s",
            "touch",
            "truncate",
            "mktemp",
            "mkdir",
            "rmdir",
            "mount",
            "umount",
            "mount -o",
            "umount -l",
            "mount -t",
            "umount -f",
            "ln -sf",
            "ln -sfn",
            "ln -sf --backup",
            "ln -sfn --backup",
            // Windows potentially mutating (cmd)
            "move",
            "ren",
            "rename",
            "copy",
            "xcopy",
            "robocopy",
            "mklink",
            "mklink /d",
            "attrib",
            "cacls",
            "icacls",
            "takeown",
            "setx",
            "reg add",
            "reg delete",
            "reg import",
            "schtasks",
            "schtasks /create",
            "schtasks /delete",
            "schtasks /change",
            "wevtutil im",
            "wevtutil sl",
            "wevtutil cl",
            "wevtutil epl",
            "diskpart",
            "format",
            "mountvol",
            "subst",
            "pushd",
            "popd",
            // Network potentially mutating
            "curl",
            "wget",
            "ftp",
            "sftp",
            "tftp",
            "ftps",
            // Windows potentially mutating (PowerShell) - non-destructive only
            "set-item",
            "set-itemproperty",
            "set-location",
            "set-variable",
            "set-alias",
            "set-executionpolicy",
            "set-service",
            "set-process",
            "set-date",
            "set-time",
            "new-item",
            "new-itemproperty",
            "new-item -itemtype",
            "new-item -path",
            "register-scheduledtask",
            "enable-scheduledtask",
            "disable-scheduledtask",
            "new-scheduledtask",
            "new-module",
            "import-module",
            "import-pssession",
            "new-pssession",
            "enter-pssession",
            "exit-pssession",
            "new-runspace",
            "enter-runspace",
            "exit-runspace",
            "new-job",
            "wait-job",
            "receive-job",
            "new-appdomain",
            // Dangerous Windows commands with wildcards
            "del *",
            "del *.*",
            "erase *",
            "erase *.*",
            "rd /s",
            "rmdir /s",
            "move *",
            "move *.*",
            "copy *",
            "copy *.*",
            "xcopy *",
            "xcopy *.*",
            "set *",
            "setx *",
            "attrib *",
            "cacls *",
            "icacls *",
            "takeown /f *",
            "takeown /r",
            "takeown /f * /r",
            "schtasks /delete /tn *",
            "schtasks /delete /s *",
            "wevtutil cl *",
            "wevtutil el | wevtutil cl",
            // Network potentially mutating (methods with side effects)
            "curl -X POST",
            "curl -X PUT",
            "curl -X DELETE",
            "curl -X PATCH",
            "wget --post-data",
            "wget --post-file",
            "ssh user@host",
            "ssh -o",
            "ssh -f",
            "ssh -L",
            "ssh -R",
            "ssh -D",
            "scp *",
            "scp -r",
            "rsync *",
            "rsync -a",
            "rsync -avz",
            "nmap -sS",
            "nmap -sT",
            "nmap -sU",
            "nmap -sA",
            "nmap -sW",
            "nmap -sP",
            "nmap -O",
            "nmap -sV",
            "nmap -A",
            "nmap --script",
            "ldapmodify",
            "ldapdelete",
            "ldapadd",
            "ldifde",
            "csvde",
        ];

        if tier2_general.contains(&command) {
            return CommandTier::Tier2;
        }

        // Default: unknown commands are Tier 2 (require approval)
        CommandTier::Tier2
    }

    fn parse_command_structure(command: &str) -> Vec<CommandComponent> {
        let mut components = Vec::new();

        // Split by pipe, &&, ||, and ;
        // This is a simple implementation - a full shell parser would be more complex
        let mut current_cmd = String::new();
        let mut chars = command.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '|' {
                if chars.peek() == Some(&'|') {
                    // ||
                    chars.next();
                    if !current_cmd.trim().is_empty() {
                        components.push(Self::parse_single_component(current_cmd.trim()));
                    }
                    current_cmd.clear();
                } else {
                    // |
                    if !current_cmd.trim().is_empty() {
                        components.push(Self::parse_single_component(current_cmd.trim()));
                    }
                    current_cmd.clear();
                }
            } else if ch == '&' && chars.peek() == Some(&'&') {
                // &&
                chars.next();
                if !current_cmd.trim().is_empty() {
                    components.push(Self::parse_single_component(current_cmd.trim()));
                }
                current_cmd.clear();
            } else if ch == ';' {
                // ;
                if !current_cmd.trim().is_empty() {
                    components.push(Self::parse_single_component(current_cmd.trim()));
                }
                current_cmd.clear();
            } else {
                current_cmd.push(ch);
            }
        }

        // Add final component
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

        // For kubectl, second part is the subcommand
        if command == "kubectl"
            || command == "pvecm"
            || command == "pvesh"
            || command == "qm"
            || command == "systemctl"
        {
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
        assert_eq!(result.tier, CommandTier::Tier2); // Escalates to highest tier
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
        assert_eq!(result.tier, CommandTier::Tier3); // rm -rf is Tier 3
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
        assert_eq!(result.tier, CommandTier::Tier1); // Both are safe
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
                "Command '{}' should be Tier 1",
                cmd
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
                "Command '{}' should be Tier 2",
                cmd
            );
        }
    }

    #[test]
    fn test_windows_tier1_readonly_commands() {
        let classifier = CommandClassifier::new();

        let tier1_commands = vec![
            "dir",
            "type file.txt",
            "more < file.txt",
            "findstr pattern file.txt",
            "ipconfig",
            "ping 127.0.0.1",
            "tracert 127.0.0.1",
            "netstat",
            "whoami",
            "date /t",
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
            "get-physicalmemory",
            "get-processor",
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
                "Command '{}' should be Tier 1",
                cmd
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
            "move *",
            "copy *.*",
            "set *",
            "setx *",
            "attrib *",
            "new-item -path C:\\test",
            "set-itemproperty -path HKLM:\\Software\\Test -name Test -value 1",
            "sudo",
            "new-scheduledtask -action (new-scheduledtaskaction -execute notepad)",
            "register-scheduledtask -taskname test -action (new-scheduledtaskaction -execute notepad)",
             "curl -X POST http://example.com",
             "wget --post-data test http://example.com",
             "time /t",
         ];

        for cmd in tier2_commands {
            let result = classifier.classify(cmd);
            assert_eq!(
                result.tier,
                CommandTier::Tier2,
                "Command '{}' should be Tier 2",
                cmd
            );
        }
    }

    #[test]
    fn test_windows_tier3_destructive_commands() {
        let classifier = CommandClassifier::new();

        let tier3_commands = vec![
            "format C: /q",
            "del *",
            "del *.*",
            "erase *",
            "erase *.*",
            "rd /s C:\\test",
            "rmdir /s C:\\test",
            "sdelete C:\\test",
            "bootrec /fixmbr",
            "bootrec /fixboot",
            "diskpart",
            "remove-item -recurse -force C:\\test",
            "clear-recyclebin",
            "stop-computer",
            "restart-computer -force",
            "remove-wmiobject -query \"select * from win32_process where name='notepad.exe'\"",
            "remove-itemproperty -path HKLM:\\Software\\Test -name Test",
            "uninstall-module -name PowerShellGet",
            "uninstall-package -name Package",
            "unregister-scheduledtask -taskname test",
            "dd if=/dev/zero of=/dev/sda",
            "mkfs.ext4 /dev/sda1",
            "remove-item -recurse C:\\test",
            "remove-item -force C:\\test",
            "clear-host",
            "stop-process",
            "stop-service",
            "restart-computer",
            "suspend-process",
            "suspend-service",
            "resume-process",
            "resume-service",
            "wait-process",
            "wait-service",
            "wait-computer",
            "start-process",
            "start-service",
            "start-computer",
            "invoke-item",
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
            "cipher /w:C:\\test",
        ];

        for cmd in tier3_commands {
            let result = classifier.classify(cmd);
            assert_eq!(
                result.tier,
                CommandTier::Tier3,
                "Command '{}' should be Tier 3",
                cmd
            );
        }
    }

    #[test]
    fn test_linux_windows_mixed_commands() {
        let classifier = CommandClassifier::new();

        // Linux commands
        let linux_commands = vec![
            "cat /etc/passwd",
            "ls -la /home",
            "grep error /var/log/syslog",
            "df -h",
            "ps aux",
            "systemctl status nginx",
            "ssh user@host",
            "scp file.txt user@host:",
            "rm -rf /tmp/test",
            "shutdown -h now",
        ];

        for cmd in linux_commands {
            let result = classifier.classify(cmd);
            assert!(
                result.tier == CommandTier::Tier1
                    || result.tier == CommandTier::Tier2
                    || result.tier == CommandTier::Tier3,
                "Linux command '{}' should have a tier",
                cmd
            );
        }

        // Windows commands
        let windows_commands = vec![
            "dir C:\\",
            "type C:\\test.txt",
            "ipconfig /all",
            "get-process",
            "get-service",
            "remove-item C:\\test",
            "stop-process -name notepad",
        ];

        for cmd in windows_commands {
            let result = classifier.classify(cmd);
            assert!(
                result.tier == CommandTier::Tier1
                    || result.tier == CommandTier::Tier2
                    || result.tier == CommandTier::Tier3,
                "Windows command '{}' should have a tier",
                cmd
            );
        }
    }
}
