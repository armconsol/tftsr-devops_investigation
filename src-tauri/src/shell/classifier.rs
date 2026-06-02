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
        // Tier 3: Always deny - destructive operations
        let tier3_commands = [
            "rm", "mkfs", "dd", "fdisk", "parted", "shutdown", "reboot", "halt", "poweroff",
        ];

        if tier3_commands.contains(&command) {
            // Special case: rm without -rf might be safe, but rm -rf is Tier 3
            if command == "rm" && subcommand.is_none() {
                // Check if this will be caught by args parsing
                return CommandTier::Tier3; // Conservative: all rm is Tier 3
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

        // Tier 1: General safe read-only commands
        let tier1_general = [
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
            "systemctl",
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
        ];

        if tier1_general.contains(&command) {
            // systemctl needs subcommand check
            if command == "systemctl" {
                if let Some(sub) = subcommand {
                    if sub == "status" || sub == "is-active" || sub == "is-enabled" {
                        return CommandTier::Tier1;
                    }
                    // restart, reload, etc. are Tier 2
                    return CommandTier::Tier2;
                }
            }
            return CommandTier::Tier1;
        }

        // Tier 2: Network and potentially mutating commands
        let tier2_general = [
            "ssh", "scp", "rsync", "curl", "wget", "chmod", "chown", "mv", "cp", "awk",
            "sed", // Can be safe, but can also modify
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
}
