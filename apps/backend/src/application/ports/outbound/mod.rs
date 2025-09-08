// Outbound Ports (Driven Ports) - Hexagonal Architecture
// These are interfaces that the application layer uses to interact with infrastructure
// Following dependency inversion principle: high-level modules don't depend on low-level modules

pub mod event_bus_port;
pub mod repository_ports;
pub mod service_ports;

// Re-export commonly used ports
pub use event_bus_port::{
    EventBusPort, DomainEventBusPort, EventBusError, EventReceipt, 
    EventBusHealthStatus, EventBusMetrics, PublishOptions, EventPriority,
    AdvancedEventBusPort, EventSubscriberPort, EventHandler, SubscriptionId
};
pub use repository_ports::*;
pub use service_ports::*;

// The repository ports are already defined in the domain layer
// Additional outbound ports for external services:

// TODO: Define additional driven port interfaces
// Examples:
// - Email service port
// - Payment service port  
// - Notification service port
// - File storage port
// - Cache port
// - Message queue port