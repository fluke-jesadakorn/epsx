// Connection Repository Adapter
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
// Manages WebSocket and SSE connections for real-time events

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::realtime_events::{
    ConnectionId, RealtimeUserId,
    repository_ports::ConnectionRepositoryPort,
    value_objects::{ConnectionInfo, ConnectionType, ConnectionStatus}
};

/// In-memory connection tracking
/// In production, this might use Redis or a distributed cache
pub struct ConnectionRepositoryAdapter {
    // Connection ID -> Connection Info
    connections: Arc<RwLock<HashMap<String, StoredConnection>>>,
    // User ID -> List of Connection IDs  
    user_connections: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

unsafe impl Send for ConnectionRepositoryAdapter {}
unsafe impl Sync for ConnectionRepositoryAdapter {}

#[derive(Debug, Clone)]
struct StoredConnection {
    connection_id: String,
    user_id: String,
    connection_info: ConnectionInfo,
    status: ConnectionStatus,
    established_at: DateTime<Utc>,
    last_heartbeat: DateTime<Utc>,
}

impl ConnectionRepositoryAdapter {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            user_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ConnectionRepositoryPort for ConnectionRepositoryAdapter {
    async fn store_connection(
        &self,
        connection_id: &ConnectionId,
        user_id: &RealtimeUserId,
        connection_info: ConnectionInfo,
    ) -> Result<(), String> {
        let conn_id_str = connection_id.to_string();
        let user_id_str = user_id.to_string();
        
        let stored_connection = StoredConnection {
            connection_id: conn_id_str.clone(),
            user_id: user_id_str.clone(),
            connection_info,
            status: ConnectionStatus::Connected,
            established_at: Utc::now(),
            last_heartbeat: Utc::now(),
        };
        
        // Store connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(conn_id_str.clone(), stored_connection);
        }
        
        // Update user -> connections mapping
        {
            let mut user_connections = self.user_connections.write().await;
            user_connections
                .entry(user_id_str)
                .or_insert_with(Vec::new)
                .push(conn_id_str);
        }
        
        tracing::info!(
            connection_id = %connection_id.to_string(),
            user_id = %user_id.to_string(),
            "Stored new real-time connection"
        );
        
        Ok(())
    }
    
    async fn remove_connection(&self, connection_id: &ConnectionId) -> Result<(), String> {
        let conn_id_str = connection_id.to_string();
        
        // Get the connection to find user ID
        let user_id_str = {
            let connections = self.connections.read().await;
            connections.get(&conn_id_str)
                .map(|conn| conn.user_id.clone())
        };
        
        if let Some(user_id) = user_id_str {
            // Remove from connections
            {
                let mut connections = self.connections.write().await;
                connections.remove(&conn_id_str);
            }
            
            // Remove from user connections mapping
            {
                let mut user_connections = self.user_connections.write().await;
                if let Some(user_conn_list) = user_connections.get_mut(&user_id) {
                    user_conn_list.retain(|id| id != &conn_id_str);
                    if user_conn_list.is_empty() {
                        user_connections.remove(&user_id);
                    }
                }
            }
            
            tracing::info!(
                connection_id = %connection_id.to_string(),
                user_id = user_id,
                "Removed real-time connection"
            );
        }
        
        Ok(())
    }
    
    async fn find_connection(&self, connection_id: &ConnectionId) -> Result<Option<ConnectionInfo>, String> {
        let connections = self.connections.read().await;
        let connection_info = connections.get(&connection_id.to_string())
            .map(|conn| conn.connection_info.clone());
        
        Ok(connection_info)
    }
    
    async fn find_user_connections(&self, user_id: &RealtimeUserId) -> Result<Vec<ConnectionId>, String> {
        let user_connections = self.user_connections.read().await;
        let connection_ids = user_connections.get(&user_id.to_string())
            .map(|conn_ids| {
                conn_ids.iter()
                    .filter_map(|id| ConnectionId::from_string(id.to_string()).ok())
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(connection_ids)
    }
    
    async fn find_connections_by_type(&self, connection_type: ConnectionType) -> Result<Vec<ConnectionId>, String> {
        let connections = self.connections.read().await;
        let connection_ids: Vec<ConnectionId> = connections.values()
            .filter(|conn| conn.connection_info.connection_type == connection_type)
            .filter_map(|conn| ConnectionId::from_string(conn.connection_id.clone()).ok())
            .collect();
        
        Ok(connection_ids)
    }
    
    async fn update_heartbeat(&self, connection_id: &ConnectionId) -> Result<(), String> {
        let conn_id_str = connection_id.to_string();
        
        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(&conn_id_str) {
                connection.last_heartbeat = Utc::now();
                tracing::trace!(
                    connection_id = %connection_id.to_string(),
                    "Updated connection heartbeat"
                );
            }
        }
        
        Ok(())
    }
    
    async fn find_stale_connections(&self, stale_after: DateTime<Utc>) -> Result<Vec<ConnectionId>, String> {
        let connections = self.connections.read().await;
        let stale_connections: Vec<ConnectionId> = connections.values()
            .filter(|conn| conn.last_heartbeat < stale_after)
            .filter_map(|conn| ConnectionId::from_string(conn.connection_id.clone()).ok())
            .collect();
        
        tracing::debug!(
            stale_count = stale_connections.len(),
            stale_after = %stale_after,
            "Found stale connections"
        );
        
        Ok(stale_connections)
    }
    
    async fn count_active_connections(&self) -> Result<u64, String> {
        let connections = self.connections.read().await;
        let active_count = connections.values()
            .filter(|conn| conn.status == ConnectionStatus::Connected)
            .count() as u64;
        
        Ok(active_count)
    }
    
    async fn count_user_connections(&self, user_id: &RealtimeUserId) -> Result<u32, String> {
        let user_connections = self.user_connections.read().await;
        let count = user_connections.get(&user_id.to_string())
            .map(|connections| connections.len())
            .unwrap_or(0) as u32;
        
        Ok(count)
    }
}

impl Default for ConnectionRepositoryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::realtime_events::value_objects::UserId;
    
    #[tokio::test]
    async fn test_store_and_retrieve_connection() {
        let adapter = ConnectionRepositoryAdapter::new();
        let connection_id = ConnectionId::new();
        let user_id = UserId::from_numeric(123);
        
        let connection_info = ConnectionInfo {
            user_agent: Some("Test Agent".to_string()),
            ip_address: "127.0.0.1".to_string(),
            connected_at: Utc::now(),
            last_ping: Utc::now(),
            connection_type: ConnectionType::WebSocket,
        };
        
        // Store connection
        adapter.store_connection(&connection_id, &user_id, connection_info.clone()).await.unwrap();
        
        // Retrieve connection
        let retrieved = adapter.find_connection(&connection_id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_info = retrieved.unwrap();
        assert_eq!(retrieved_info.ip_address, connection_info.ip_address);
        assert_eq!(retrieved_info.connection_type, connection_info.connection_type);
    }
    
    #[tokio::test]
    async fn test_user_connections_mapping() {
        let adapter = ConnectionRepositoryAdapter::new();
        let user_id = UserId::from_numeric(123);
        
        // Add multiple connections for the same user
        let conn1 = ConnectionId::new();
        let conn2 = ConnectionId::new();
        
        let connection_info = ConnectionInfo {
            user_agent: None,
            ip_address: "127.0.0.1".to_string(),
            connected_at: Utc::now(),
            last_ping: Utc::now(),
            connection_type: ConnectionType::WebSocket,
        };
        
        adapter.store_connection(&conn1, &user_id, connection_info.clone()).await.unwrap();
        adapter.store_connection(&conn2, &user_id, connection_info).await.unwrap();
        
        // Check user connections
        let user_connections = adapter.find_user_connections(&user_id).await.unwrap();
        assert_eq!(user_connections.len(), 2);
        
        // Remove one connection
        adapter.remove_connection(&conn1).await.unwrap();
        
        let user_connections = adapter.find_user_connections(&user_id).await.unwrap();
        assert_eq!(user_connections.len(), 1);
    }
    
    #[tokio::test]
    async fn test_heartbeat_updates() {
        let adapter = ConnectionRepositoryAdapter::new();
        let connection_id = ConnectionId::new();
        let user_id = UserId::from_numeric(123);
        
        let connection_info = ConnectionInfo {
            user_agent: None,
            ip_address: "127.0.0.1".to_string(),
            connected_at: Utc::now(),
            last_ping: Utc::now(),
            connection_type: ConnectionType::ServerSentEvents,
        };
        
        adapter.store_connection(&connection_id, &user_id, connection_info).await.unwrap();
        
        // Update heartbeat
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        adapter.update_heartbeat(&connection_id).await.unwrap();
        
        // Check that connection is not stale
        let past = Utc::now() - chrono::Duration::milliseconds(5);
        let stale_connections = adapter.find_stale_connections(past).await.unwrap();
        assert!(stale_connections.is_empty());
    }
}