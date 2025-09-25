// Domain Ports - Interfaces for external dependencies
// Ports define contracts that infrastructure adapters must implement

pub mod market_data_service_port;

// Re-export commonly used ports
pub use market_data_service_port::{MarketDataServicePort, MarketDataConfig};