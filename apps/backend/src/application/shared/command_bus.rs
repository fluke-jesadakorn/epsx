use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;

use super::ApplicationResult;

/// Base trait for all commands (write operations)
/// Commands represent intent to change state
pub trait Command: Send + Sync + Debug + Clone {
    /// The type of response this command produces
    type Response: Send + Sync;
    
    /// Validate the command before execution
    fn validate(&self) -> ApplicationResult<()> {
        Ok(())
    }
}

/// Handler for a specific command type
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    /// Handle the command and return the response
    async fn handle(&self, command: C) -> ApplicationResult<C::Response>;
}

// CommandBus trait and InMemoryCommandBus removed - unused abstraction
// Handlers are called directly in application services, no bus dispatch needed

/// Command metadata for audit and tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    pub command_id: uuid::Uuid,
    pub command_type: String,
    pub initiated_by: Option<String>, // User ID who initiated the command
    pub initiated_at: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<String>, // For tracing across services
}

impl CommandMetadata {
    pub fn new(command_type: impl Into<String>) -> Self {
        Self {
            command_id: uuid::Uuid::new_v4(),
            command_type: command_type.into(),
            initiated_by: None,
            initiated_at: chrono::Utc::now(),
            correlation_id: None,
        }
    }
    
    pub fn with_user(mut self, wallet_address: impl Into<String>) -> Self {
        self.initiated_by = Some(wallet_address.into());
        self
    }
    
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
}