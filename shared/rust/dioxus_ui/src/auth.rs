pub mod user;
pub mod auth_modal;
pub mod auth_gate;
pub mod wallet_button;
pub mod progressive_banner;
pub mod access_denied;
// === wave40-t2 domain subdirs port ===
// 11 NEW auth components ported from
// `apps-old/frontend/components/auth/`:
//   - analytics_auth_wrapper  (11 LoC)
//   - api_key_manager         (465 LoC, see below)
//   - connected_wallet_dropdown (147 LoC)
//   - frontend_auth_gate      (19 LoC)
//   - frontend_auth_modal     (17 LoC)
//   - global_auth_guard       (34 LoC)
//   - permissions_display     (111 LoC)
//   - progressive_auth_banner (16 LoC, re-export of progressive_banner)
//   - wallet_connect_auth     (259 LoC)
//   - wallet_connection_modal (54 LoC)
// `api_key_manager` is ported as a full CRUD shell — the key
// generator + the per-key row + the API docs section — but the
// API calls themselves are stubbed (the real BFF endpoints are
// added in a follow-up wave; this is the visual + structural
// port).
pub mod analytics_auth_wrapper;
pub mod api_key_manager;
pub mod connected_wallet_dropdown;
pub mod frontend_auth_gate;
pub mod frontend_auth_modal;
pub mod global_auth_guard;
pub mod permissions_display;
pub mod progressive_auth_banner;
pub mod wallet_connect_auth;
pub mod wallet_connection_modal;

pub use user::*;
pub use auth_modal::*;
pub use auth_gate::*;
pub use wallet_button::*;
pub use progressive_banner::*;
pub use access_denied::*;
pub use analytics_auth_wrapper::AnalyticsAuthWrapper;
pub use api_key_manager::{ApiKey, ApiKeyManager, ApiKeyScope};
pub use connected_wallet_dropdown::ConnectedWalletDropdown;
pub use frontend_auth_gate::FrontendAuthGate;
pub use frontend_auth_modal::FrontendAuthModal;
pub use global_auth_guard::GlobalAuthGuard;
pub use permissions_display::{PermissionsDisplay, UserPermissionsSnapshot};
pub use progressive_auth_banner::ProgressiveAuthBannerLegacy;
pub use wallet_connect_auth::WalletConnectAuth;
pub use wallet_connection_modal::WalletConnectionModal;
