use std::collections::HashMap;

use crate::ai::Message;

#[derive(Clone)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub tools: Vec<String>,
    pub model: Option<String>,
    pub priority: u32,
}

pub struct AgentRegistry {
    agents: HashMap<String, Agent>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    pub fn new() -> Self {
        AgentRegistry {
            agents: HashMap::new(),
        }
    }

    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.insert(agent.name.clone(), agent);
    }

    pub fn get(&self, name: &str) -> Option<&Agent> {
        self.agents.get(name)
    }

    pub fn get_all(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    pub fn has_agent(&self, name: &str) -> bool {
        self.agents.contains_key(name)
    }
}

pub fn create_agent_registry() -> AgentRegistry {
    let mut registry = AgentRegistry::new();

    let devops_agent = include_str!("agents/devops_incident_responder.md");
    registry.add_agent(Agent {
        name: "devops-incident-responder".to_string(),
        description: "Production incident response, diagnosis, and postmortems".to_string(),
        system_prompt: devops_agent.to_string(),
        tools: vec![],
        model: None,
        priority: 10,
    });

    registry
}

pub fn load_agent(name: &str) -> Option<Agent> {
    let registry = create_agent_registry();
    registry.get(name).cloned()
}

pub fn detect_domain(messages: &[Message]) -> String {
    let combined_text = messages
        .iter()
        .map(|m| m.content.as_str())
        .collect::<Vec<&str>>()
        .join(" ");

    let combined_lower = combined_text.to_lowercase();

    let domain_keywords: &[(&str, &[&str])] = &[
        (
            "linux",
            &[
                "linux", "ubuntu", "debian", "rhel", "centos", "systemd", "kernel", "selinux",
            ],
        ),
        (
            "windows",
            &[
                "windows",
                "windows server",
                "ad",
                "active directory",
                "iis",
                "gpo",
            ],
        ),
        (
            "network",
            &[
                "network",
                "firewall",
                "router",
                "switch",
                "fortigate",
                "cisco",
                "aruba",
            ],
        ),
        (
            "kubernetes",
            &[
                "kubernetes",
                "k8s",
                "k3s",
                "helm",
                "pod",
                "deployment",
                "namespace",
            ],
        ),
        (
            "databases",
            &[
                "database",
                "postgresql",
                "mysql",
                "redis",
                "rabbitmq",
                "sql",
            ],
        ),
        (
            "virtualization",
            &[
                "vm",
                "virtual machine",
                "vmware",
                "proxmox",
                "hyper-v",
                "kvm",
            ],
        ),
        (
            "hardware",
            &["hardware", "disk", "raid", "memory", "cpu", "motherboard"],
        ),
        (
            "observability",
            &[
                "monitoring",
                "grafana",
                "prometheus",
                "kibana",
                "logging",
                "metrics",
            ],
        ),
        (
            "telephony",
            &["voip", "sip", "asterisk", "pbx", "telephony", "sbc"],
        ),
        (
            "security",
            &[
                "security",
                "vault",
                "encryption",
                "certificate",
                "tls",
                "ssl",
                "firewall",
            ],
        ),
        (
            "public_safety",
            &["911", "ng911", "nena", "psap", "cad", "dispatch"],
        ),
        (
            "application",
            &["java", "spring", "tomcat", "jvm", "application", "app"],
        ),
        (
            "automation",
            &[
                "ansible",
                "jenkins",
                "ci/cd",
                "automation",
                "pipeline",
                "terraform",
            ],
        ),
        (
            "hpe_infra",
            &["hpe", "oneview", "ilo", "synergy", "dl360", "dl320"],
        ),
        (
            "dell_hardware",
            &["dell", "idrac", "poweredge", "perc", "lifecycle controller"],
        ),
        (
            "identity",
            &[
                "identity", "keycloak", "boundary", "sso", "ldap", "ad", "auth",
            ],
        ),
    ];

    let mut scores: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for (domain, keywords) in domain_keywords {
        let mut score = 0;
        for keyword in *keywords {
            if combined_lower.contains(keyword) {
                score += 1;
            }
        }
        if score > 0 {
            scores.insert(domain.to_string(), score);
        }
    }

    if scores.is_empty() {
        return "general".to_string();
    }

    scores
        .iter()
        .max_by_key(|(_, score)| *score)
        .map(|(domain, _)| domain.clone())
        .unwrap_or_else(|| "general".to_string())
}
