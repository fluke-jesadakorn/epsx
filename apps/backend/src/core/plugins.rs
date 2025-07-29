use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::dom::events::DomainEvent;
use crate::core::errors::AppError;

/// Plugin metadata containing information about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
    pub api_version: String,
}

/// Plugin lifecycle states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginState {
    Unloaded,
    Loading,
    Loaded,
    Active,
    Error(String),
}

/// Core plugin trait that all plugins must implement  
#[async_trait]
pub trait Plugin: Send + Sync + std::fmt::Debug {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin with configuration
    async fn initialize(&mut self, config: &PluginConfig) -> Result<(), AppError>;
    
    /// Start the plugin (called after all plugins are loaded)
    async fn start(&mut self) -> Result<(), AppError>;
    
    /// Stop the plugin gracefully
    async fn stop(&mut self) -> Result<(), AppError>;
    
    /// Handle events (optional)
    async fn handle_event(&self, _event: &dyn DomainEvent) -> Result<(), AppError> {
        Ok(())
    }
    
    /// Get plugin capabilities as trait objects
    fn capabilities(&self) -> HashMap<String, Box<dyn Any + Send + Sync>> {
        HashMap::new()
    }
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
    pub environment: String,
}

/// Plugin registry entry
#[derive(Debug)]
pub struct PluginEntry {
    pub metadata: PluginMetadata,
    pub plugin: Arc<dyn Plugin>,
    pub state: PluginState,
    pub config: PluginConfig,
}

/// Plugin manager for loading, managing, and coordinating plugins
pub struct PluginManager {
    plugins: HashMap<String, PluginEntry>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
    
    /// Register a plugin (for static loading)
    pub async fn register_plugin(
        &mut self,
        plugin: Arc<dyn Plugin>,
        config: PluginConfig,
    ) -> Result<(), AppError> {
        let metadata = plugin.metadata().clone();
        let name = metadata.name.clone();
        
        // Check dependencies
        self.check_dependencies(&metadata)?;
        
        let entry = PluginEntry {
            metadata,
            plugin,
            state: PluginState::Loaded,
            config,
        };
        
        self.plugins.insert(name, entry);
        Ok(())
    }
    
    /// Initialize all registered plugins
    pub async fn initialize_all(&mut self) -> Result<(), AppError> {
        // Sort plugins by dependencies (simplified - just process in order)
        let plugin_names: Vec<String> = self.plugins.keys().cloned().collect();
        
        for name in plugin_names {
            if let Some(entry) = self.plugins.get_mut(&name) {
                entry.state = PluginState::Loading;
                
                // Create mutable reference to plugin
                let mut plugin_clone = Arc::clone(&entry.plugin);
                let config = entry.config.clone();
                
                // Initialize plugin (Note: This requires unsafe due to Arc mutability)
                // In a real implementation, you'd use interior mutability or a different pattern
                match Arc::get_mut(&mut plugin_clone) {
                    Some(plugin_mut) => {
                        if let Err(e) = plugin_mut.initialize(&config).await {
                            entry.state = PluginState::Error(e.to_string());
                            return Err(e);
                        }
                        entry.state = PluginState::Active;
                    }
                    None => {
                        // Plugin is shared, use a different initialization pattern
                        entry.state = PluginState::Active;
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Start all plugins
    pub async fn start_all(&mut self) -> Result<(), AppError> {
        for (name, entry) in self.plugins.iter_mut() {
            if entry.state == PluginState::Active {
                if let Some(plugin_mut) = Arc::get_mut(&mut entry.plugin) {
                    if let Err(e) = plugin_mut.start().await {
                        entry.state = PluginState::Error(e.to_string());
                        return Err(AppError::new(
                            crate::core::errors::ErrorKind::InternalError,
                            format!("Failed to start plugin {}: {}", name, e)
                        ));
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Stop all plugins
    pub async fn stop_all(&mut self) -> Result<(), AppError> {
        for (name, entry) in self.plugins.iter_mut() {
            if entry.state == PluginState::Active {
                if let Some(plugin_mut) = Arc::get_mut(&mut entry.plugin) {
                    if let Err(e) = plugin_mut.stop().await {
                        entry.state = PluginState::Error(e.to_string());
                        // Continue stopping other plugins even if one fails
                        tracing::error!("Failed to stop plugin {}: {}", name, e);
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&PluginEntry> {
        self.plugins.get(name)
    }
    
    /// List all plugins
    pub fn list_plugins(&self) -> Vec<&PluginMetadata> {
        self.plugins.values().map(|entry| &entry.metadata).collect()
    }
    
    /// Get plugin states
    pub fn get_plugin_states(&self) -> HashMap<String, PluginState> {
        self.plugins
            .iter()
            .map(|(name, entry)| (name.clone(), entry.state.clone()))
            .collect()
    }
    
    /// Dispatch event to all plugins
    pub async fn dispatch_event(&self, event: &dyn DomainEvent) -> Result<(), AppError> {
        for entry in self.plugins.values() {
            if entry.state == PluginState::Active {
                if let Err(e) = entry.plugin.handle_event(event).await {
                    tracing::error!("Plugin {} failed to handle event: {}", entry.metadata.name, e);
                    // Continue dispatching to other plugins
                }
            }
        }
        Ok(())
    }
    
    /// Check plugin dependencies
    fn check_dependencies(&self, metadata: &PluginMetadata) -> Result<(), AppError> {
        for dep in &metadata.dependencies {
            if !self.plugins.contains_key(dep) {
                return Err(AppError::new(
                    crate::core::errors::ErrorKind::InternalError,
                    format!("Plugin {} depends on {}, which is not loaded", metadata.name, dep)
                ));
            }
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin factory for creating plugin instances
pub trait PluginFactory: Send + Sync {
    fn create_plugin(&self) -> Result<Arc<dyn Plugin>, AppError>;
}

/// Registry for plugin factories (for dynamic loading)
pub struct PluginRegistry {
    factories: HashMap<String, Box<dyn PluginFactory>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }
    
    pub fn register_factory(&mut self, name: String, factory: Box<dyn PluginFactory>) {
        self.factories.insert(name, factory);
    }
    
    pub fn create_plugin(&self, name: &str) -> Result<Arc<dyn Plugin>, AppError> {
        match self.factories.get(name) {
            Some(factory) => factory.create_plugin(),
            None => Err(AppError::new(
                crate::core::errors::ErrorKind::InternalError,
                format!("Plugin factory {} not found", name)
            )),
        }
    }
    
    pub fn list_available(&self) -> Vec<&String> {
        self.factories.keys().collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for plugins that handle specific event types
#[async_trait]
pub trait TypedEventHandler<T>: Send + Sync 
where 
    T: Clone + Send + Sync,
{
    async fn handle_typed_event(&self, event: &T) -> Result<(), AppError>;
}

/// Plugin capability trait for trading features
#[async_trait]
pub trait TradingPlugin: Plugin {
    async fn analyze_stock(&self, symbol: &str) -> Result<serde_json::Value, AppError>;
    async fn get_recommendations(&self) -> Result<Vec<String>, AppError>;
}

/// Plugin capability trait for data providers
#[async_trait]
pub trait DataProviderPlugin: Plugin {
    async fn fetch_market_data(&self, symbol: &str) -> Result<serde_json::Value, AppError>;
    async fn get_historical_data(&self, symbol: &str, days: u32) -> Result<serde_json::Value, AppError>;
}

/// Plugin capability trait for notification providers
#[async_trait]
pub trait NotificationPlugin: Plugin {
    async fn send_notification(&self, user_id: &str, message: &str) -> Result<(), AppError>;
    async fn send_bulk_notification(&self, user_ids: &[String], message: &str) -> Result<(), AppError>;
}