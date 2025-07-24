// External service implementations

pub mod email;
pub mod notification;
pub mod payment;
pub mod market_data;

pub use email::{SendGridEmailService, MockEmailService, SentEmail};
pub use notification::{
    Notification, NotificationType, NotificationPriority, NotificationService,
    InMemoryNotificationService, DatabaseNotificationService
};
pub use payment::{
    PaymentGatewayConfig, NetworkConfig, MultiGatewayPaymentService,
    CoinPaymentsGateway, MockPaymentGateway
};
pub use market_data::{
    MarketDataConfig, AlphaVantageService, MockMarketDataService
};

// TODO: Implement remaining external services like:
// - WebSocketService (real-time communication)