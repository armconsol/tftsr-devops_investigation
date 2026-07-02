// Connection pooling manager for database drivers

use crate::db_drivers::{
    error::{DriverError, DriverResult},
    types::ConnectionConfig,
    DatabaseDriver,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type alias for a shared, thread-safe database driver
type SharedDriver = Arc<RwLock<Box<dyn DatabaseDriver>>>;

/// Type alias for the pool storage
type PoolStorage = Arc<RwLock<HashMap<String, SharedDriver>>>;

/// Manages connection pools for multiple database connections
pub struct DatabasePoolManager {
    pools: PoolStorage,
}

impl DatabasePoolManager {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get an existing driver from the pool
    pub async fn get_driver(&self, connection_id: &str) -> DriverResult<SharedDriver> {
        let pools = self.pools.read().await;

        if let Some(driver) = pools.get(connection_id) {
            Ok(Arc::clone(driver))
        } else {
            Err(DriverError::NotConnected)
        }
    }

    /// Get or create a driver for the given connection ID
    pub async fn get_or_create_driver(
        &self,
        connection_id: &str,
        config: &ConnectionConfig,
    ) -> DriverResult<SharedDriver> {
        let pools = self.pools.read().await;

        if let Some(driver) = pools.get(connection_id) {
            return Ok(Arc::clone(driver));
        }

        drop(pools);

        // Create new driver
        let mut driver = crate::db_drivers::create_driver(config)?;
        driver.connect(config).await?;

        let driver_arc = Arc::new(RwLock::new(driver));

        let mut pools = self.pools.write().await;
        pools.insert(connection_id.to_string(), Arc::clone(&driver_arc));

        Ok(driver_arc)
    }

    /// Remove a driver from the pool
    pub async fn remove_driver(&self, connection_id: &str) -> DriverResult<()> {
        let mut pools = self.pools.write().await;

        if let Some(driver_arc) = pools.remove(connection_id) {
            let mut driver = driver_arc.write().await;
            driver.disconnect().await?;
        }

        Ok(())
    }

    /// Disconnect all drivers and clear the pool
    pub async fn clear_all(&self) -> DriverResult<()> {
        let mut pools = self.pools.write().await;

        for (_id, driver_arc) in pools.drain() {
            let mut driver = driver_arc.write().await;
            let _ = driver.disconnect().await; // Ignore errors during cleanup
        }

        Ok(())
    }

    /// Get number of active connections
    pub async fn active_count(&self) -> usize {
        let pools = self.pools.read().await;
        pools.len()
    }

    /// Check if a connection exists in the pool
    pub async fn has_connection(&self, connection_id: &str) -> bool {
        let pools = self.pools.read().await;
        pools.contains_key(connection_id)
    }
}

impl Default for DatabasePoolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_manager_create() {
        let manager = DatabasePoolManager::new();
        assert_eq!(manager.active_count().await, 0);
    }

    #[tokio::test]
    async fn test_pool_manager_has_connection() {
        let manager = DatabasePoolManager::new();
        assert!(!manager.has_connection("test_conn").await);
    }

    #[tokio::test]
    async fn test_pool_manager_clear_all() {
        let manager = DatabasePoolManager::new();
        manager.clear_all().await.unwrap();
        assert_eq!(manager.active_count().await, 0);
    }
}
