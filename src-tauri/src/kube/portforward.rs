pub struct PortForwardSession {
    pub id: String,
    pub cluster_id: String,
    pub namespace: String,
    pub pod: String,
    pub container: Option<String>,
    pub ports: Vec<u16>,
    pub local_ports: Vec<u16>,
    pub status: PortForwardStatus,
}

pub enum PortForwardStatus {
    Active,
    Stopped,
    Error(String),
}

impl PortForwardSession {
    pub fn new(
        id: String,
        cluster_id: String,
        namespace: String,
        pod: String,
        container: Option<String>,
        ports: Vec<u16>,
        local_ports: Vec<u16>,
    ) -> Self {
        Self {
            id,
            cluster_id,
            namespace,
            pod,
            container,
            ports,
            local_ports,
            status: PortForwardStatus::Active,
        }
    }

    pub fn stop(&mut self) {
        self.status = PortForwardStatus::Stopped;
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, PortForwardStatus::Active)
    }
}
