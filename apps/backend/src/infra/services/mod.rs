// External service implementations

pub mod email;
pub mod notification;
pub mod payment;
pub mod market_data;
pub mod encryption;
pub mod tradingview;
pub mod websocket;
pub mod tradingview_websocket;

pub use email::{SendGridEmailService, MockEmailService, SentEmail};
pub use notification::{
    Notification, NotificationType, NotificationPriority, NotificationService,
    InMemoryNotificationService, DatabaseNotificationService, NotificationPortAdapter
};
pub use payment::{
    PaymentGatewayConfig, NetworkConfig, MultiGatewayPaymentService,
    CoinPaymentsGateway, MockPaymentGateway
};
pub use market_data::{
    MarketDataConfig, AlphaVantageService, MockMarketDataService
};
pub use encryption::{EncryptionService, EncryptionError};
pub use tradingview::{TradingViewService, TradingViewApiService, TradingViewConfig};
pub use websocket::{WebSocketClient, WebSocketConnection, WebSocketError};
pub use tradingview_websocket::TradingViewWebSocketService;

// TODO: Implement remaining external services like:
// - WebSocketService (real-time communication)