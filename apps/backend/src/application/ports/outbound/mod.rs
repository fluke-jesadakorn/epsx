// Outbound Ports (Driven Ports) - Hexagonal Architecture
// These are interfaces that the application layer uses to interact with infrastructure
// Following dependency inversion principle: high-level modules don't depend on low-level modules

// pub mod event_bus_port; // Removed - unused event bus abstraction (uses DomainEventBus from shared_kernel instead)
// pub mod repository_ports; // Removed - duplicate of domain layer repository ports
pub mod service_ports;

// Re-export commonly used ports
// Event bus port exports removed - unused abstraction
// Repository port exports removed - use domain layer ports directly
pub use service_ports::*;

// The repository ports are already defined in the domain layer
// Additional outbound ports for external services:

// Additional driven port interfaces would be defined here
// Currently implemented services that could be abstracted as ports:
// - Web3 service (infrastructure/adapters/services/permission_adapter.rs)
// - Payment service (infrastructure/adapters/services/payment_security_service.rs)  
// - Notification service (infrastructure/adapters/services/notification_service_adapter.rs)
// - Cache service (infrastructure/cache/mod.rs)
// Future ports to consider: File storage, Message queue