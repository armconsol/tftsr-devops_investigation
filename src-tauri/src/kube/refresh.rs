use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct RefreshRegistry {
    domains: HashMap<String, Domain>,
}

impl Default for RefreshRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Domain {
    pub name: String,
    pub refresh_interval: std::time::Duration,
    pub data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl RefreshRegistry {
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
        }
    }

    pub async fn register_domain(&mut self, domain: Domain) {
        self.domains.insert(domain.name.clone(), domain);
    }

    pub async fn get_domain(&self, name: &str) -> Option<&Domain> {
        self.domains.get(name)
    }
}
