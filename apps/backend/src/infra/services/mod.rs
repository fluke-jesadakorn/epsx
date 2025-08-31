// External service implementations

pub mod email_service;
pub mod notification_service;
pub mod payment_service;
pub mod market_data_service;
pub mod encryption_service;
pub mod tradingview;
pub mod websocket_service;
pub mod tradingview_websocket_service;
pub mod permission_infrastructure;

// Re-export with shorter alias for backward compatibility
pub use tradingview_websocket_service as tradingview_websocket;
pub mod api_key_service;

pub use email_service::{SendGridEmailService, MockEmailService, SentEmail};
pub use notification_service::{
    Notification, NotificationType, NotificationPriority, NotificationService,
    InMemoryNotificationService, NotificationPortAdapter
};
pub use payment_service::{
    PaymentGatewayConfig, NetworkConfig, MultiGatewayPaymentService,
    CoinPaymentsGateway, MockPaymentGateway
};
pub use market_data_service::{
    MarketDataConfig, AlphaVantageService, MockMarketDataService
};
pub use encryption_service::{EncryptionService, EncryptionError};
pub use tradingview::{TradingViewService, TradingViewApiService, TradingViewConfig};
pub use websocket_service::{WebSocketClient, WebSocketConnection, WebSocketError};

// Re-export websocket_service as websocket for backward compatibility  
pub use websocket_service as websocket;
pub use tradingview_websocket_service::TradingViewWebSocketService;
pub use api_key_service::{ApiKeyService, ApiKeyError};
pub use permission_infrastructure::{
    PermissionInfrastructureService, PermissionInfrastructureServiceFactory, 
    InfrastructurePermissionError
};

// TODO: Implement remaining external services like:
// - WebSocketService (real-time communication)