// Route architecture separation for dynamic plan system
// Enables different access patterns: internal web app, external API, admin interface

pub mod internal;
pub mod external;
pub mod admin;

use axum::Router;
use std::sync::Arc;
use crate::infrastructure::AppContainer;

pub use internal::InternalRoutes;
pub use external::ExternalRoutes;
pub use admin::AdminRoutes;

/// Access context for route-specific middleware and authentication
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessContext {
    /// Internal web application access - OIDC authenticated users
    Internal,
    /// External API access - API key authenticated developers
    External,
    /// Admin interface access - OIDC + admin permissions
    Admin,
}

impl AccessContext {
    pub fn prefix(&self) -> &'static str {
        match self {
            AccessContext::Internal => "/web",
            AccessContext::External => "/api/external",
            AccessContext::Admin => "/admin",
        }
    }

    pub fn requires_api_key(&self) -> bool {
        matches!(self, AccessContext::External)
    }

    pub fn requires_oidc(&self) -> bool {
        matches!(self, AccessContext::Internal | AccessContext::Admin)
    }

    pub fn is_billable_context(&self) -> bool {
        matches!(self, AccessContext::External)
    }
}

/// Route builder with context-aware middleware
pub struct ContextualRouterBuilder {
    context: AccessContext,
    container: Arc<AppContainer>,
}

impl ContextualRouterBuilder {
    pub fn new(context: AccessContext, container: Arc<AppContainer>) -> Self {
        Self { context, container }
    }

    /// Build router with context-specific middleware stack
    pub async fn build(self) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
        match self.context {
            AccessContext::Internal => {
                InternalRoutes::create_routes(self.container).await
            }
            AccessContext::External => {
                ExternalRoutes::create_routes(self.container).await
            }
            AccessContext::Admin => {
                AdminRoutes::create_routes(self.container).await
            }
        }
    }
}