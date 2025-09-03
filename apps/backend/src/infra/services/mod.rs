// External service implementations

pub mod email_service;
pub mod payment_service;
pub mod market_data_service;
pub mod encryption_service;
pub mod tradingview;
// pub mod websocket_service; // Removed during WebSocket cleanup
pub mod tradingview_websocket_service;
pub mod permission_infrastructure;
pub mod fcm_service;
pub mod fcm_topic_service;

// Re-export with shorter alias for backward compatibility
pub use tradingview_websocket_service as tradingview_websocket;
pub mod api_key_service;

pub use email_service::{SendGridEmailService, MockEmailService, SentEmail};
pub use payment_service::{
    PaymentGatewayConfig, NetworkConfig, MultiGatewayPaymentService,
    CoinPaymentsGateway, MockPaymentGateway
};
pub use market_data_service::{
    MarketDataConfig, AlphaVantageService, MockMarketDataService
};
pub use encryption_service::{EncryptionService, EncryptionError};
pub use tradingview::{TradingViewService, TradingViewApiService, TradingViewConfig};
// pub use websocket_service::{WebSocketClient, WebSocketConnection, WebSocketError}; // Removed during WebSocket cleanup

// Re-export websocket_service as websocket for backward compatibility  
// pub use websocket_service as websocket; // Removed during WebSocket cleanup
pub use tradingview_websocket_service::TradingViewWebSocketService;
pub use api_key_service::{ApiKeyService, ApiKeyError};
pub use permission_infrastructure::{
    PermissionInfrastructureService, PermissionInfrastructureServiceFactory, 
    InfrastructurePermissionError
};
pub use fcm_service::{FcmService, FcmMessage, FcmNotification, FcmTarget, DeliveryStats};
pub use fcm_topic_service::{FcmTopicService, PlatformTopics, TopicSubscriptionRequest, TopicSubscriptionResponse};

// TODO: Implement remaining external services like:
// - WebSocketService (real-time communication)