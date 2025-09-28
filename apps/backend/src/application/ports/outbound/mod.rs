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

// Additional driven port interfaces would be defined here
// Currently implemented services that could be abstracted as ports:
// - Email service (infrastructure/adapters/services/email_service.rs)
// - Payment service (infrastructure/adapters/services/payment_security_service.rs)  
// - Notification service (infrastructure/adapters/services/notification_service_adapter.rs)
// - Cache service (infrastructure/cache/mod.rs)
// Future ports to consider: File storage, Message queue