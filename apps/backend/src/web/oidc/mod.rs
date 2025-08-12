// Enhanced Multi-Tenant OpenID Connect (OIDC) System
// Comprehensive OIDC implementation with dynamic provider discovery,
// multi-tenant support, session federation, and advanced security features

// Core OIDC modules
pub mod handlers;
pub mod routes;
pub mod discovery;
pub mod types;
pub mod authorization;
pub mod token;

// Enhanced multi-tenant modules
pub mod provider_registry;
pub mod tenant_resolver;
pub mod discovery_client;
pub mod enhanced_token_broker;
pub mod enhanced_handlers;
pub mod session_federation;
pub mod token_management;

// Re-exports for backward compatibility
pub use handlers::*;
pub use routes::*;
pub use discovery::*;
pub use types::*;

// Enhanced system exports
pub use provider_registry::{
    ProviderRegistryTrait, InMemoryProviderRegistry, OIDCProviderConfig, 
    OIDCProviderType, TenantResolution, OIDCDiscoveryDocument,
};
pub use tenant_resolver::{
    TenantResolverTrait, InMemoryTenantResolver, TenantMapping, 
    DomainMatchStrategy, EnhancedTenantResolution,
};
pub use discovery_client::{
    DiscoveryClientTrait, HttpDiscoveryClient, DiscoveryClientConfig,
    EnhancedProviderConfigurator,
};
pub use enhanced_token_broker::{
    EnhancedTokenBroker, EnhancedTokenBrokerConfig, EnhancedUnifiedJWT,
    EnhancedAuthorizationRequest, EnhancedTokenRequest, PKCEChallenge,
    AuthorizationFlowResult,
};
pub use enhanced_handlers::{
    enhanced_authorize, enhanced_token, enhanced_userinfo,
    token_introspection, token_revocation, EnhancedTokenResponse,
    EnhancedUserInfoResponse, OIDCErrorResponse,
};
pub use session_federation::{
    SessionFederationTrait, InMemorySessionFederation, FederatedSession,
    AppSessionInfo, CrossAppAuthRequest, CrossAppAuthResponse,
    SessionFederationConfig,
};
pub use token_management::{
    TokenManagementTrait, TTLPolicyManagerTrait, InMemoryTokenManager,
    TokenMetadata, TokenType, TokenStatus, TTLPolicy, TTLConfiguration,
    TokenRevocationRequest, BulkRevocationRequest, TTLEvaluationContext,
};