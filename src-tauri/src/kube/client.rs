pub struct ClusterClient {
    pub id: String,
    pub name: String,
    pub context: String,
    pub server_url: String,
}

impl ClusterClient {
    pub fn new(id: String, name: String, context: String, server_url: String) -> Self {
        Self {
            id,
            name,
            context,
            server_url,
        }
    }
}
