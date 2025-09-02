// External service implementations

pub mod email_service;
pub mod fcm_notification_service;
pub mod fcm_token_service;
pub mod fcm_push_service;
pub mod payment_service;
pub mod market_data_service;
pub mod encryption_service;
pub mod tradingview;
// pub mod websocket_service; // Removed during WebSocket cleanup
pub mod tradingview_websocket_service;
pub mod permission_infrastructure;

// Re-export with shorter alias for backward compatibility
pub use tradingview_websocket_service as tradingview_websocket;
pub mod api_key_service;

pub use email_service::{SendGridEmailService, MockEmailService, SentEmail};
pub use fcm_token_service::{
    FcmTokenService, SimpleFcmTokenService, FcmTokenInfo, FcmTokenError
};
pub use fcm_notification_service::{
    FcmNotificationService, NotificationService, Notification, NotificationType,
    NotificationPriority, NotificationDeliveryStatus, NotificationPreferences,
    NotificationQuery, ServiceNotificationStats
};
pub use fcm_push_service::{
    FcmPushService, ComprehensiveFcmPushService, FcmMessage, FcmPriority,
    FcmSendResult, FcmBatchResult, FcmPushError
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
// pub use websocket_service::{WebSocketClient, WebSocketConnection, WebSocketError}; // Removed during WebSocket cleanup

// Re-export websocket_service as websocket for backward compatibility  
// pub use websocket_service as websocket; // Removed during WebSocket cleanup
pub use tradingview_websocket_service::TradingViewWebSocketService;
pub use api_key_service::{ApiKeyService, ApiKeyError};
pub use permission_infrastructure::{
    PermissionInfrastructureService, PermissionInfrastructureServiceFactory, 
    InfrastructurePermissionError
};

// TODO: Implement remaining external services like:
// - WebSocketService (real-time communication)