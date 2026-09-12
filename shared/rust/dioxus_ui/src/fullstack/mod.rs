//! Hydrated UI contracts. Server-only providers are supplied by each native BFF.
pub mod admin;
pub mod admin_auth;
pub mod admin_catalog;
pub mod admin_orders;
pub mod admin_settings;
pub mod analytics;
pub mod frontend_auth;
pub mod frontend_payment;
#[cfg(feature = "pay-ui")]
pub mod pay;
pub mod shell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Frontend,
    Admin,
    Pay,
}

impl Surface {
    pub const fn server_prefix(self) -> &'static str {
        match self {
            Self::Frontend => "/_server/frontend/",
            Self::Admin => "/_server/admin/",
            Self::Pay => "/_server/pay/",
        }
    }

    pub fn owns_server_path(self, path: &str) -> bool {
        path.strip_prefix(self.server_prefix())
            .is_some_and(|name| !name.is_empty() && !name.contains('/') && !name.contains('.'))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LoadError {
    InvalidQuery,
    NotFound,
    Unauthenticated,
    Forbidden,
    Unavailable,
    Malformed,
}

impl LoadError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::NotFound => "The requested page was not found.",
            Self::InvalidQuery => "Check the selected filters and try again.",
            Self::Unauthenticated => "Please sign in again to continue.",
            Self::Forbidden => "Your account cannot access these results.",
            Self::Unavailable | Self::Malformed => "Could not load results. Please try again.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn server_functions_are_separated_by_application() {
        for surface in [Surface::Frontend, Surface::Admin, Surface::Pay] {
            assert!(surface.owns_server_path(&format!("{}read", surface.server_prefix())));
            for other in [Surface::Frontend, Surface::Admin, Surface::Pay] {
                if surface != other {
                    assert!(!surface.owns_server_path(&format!("{}read", other.server_prefix())));
                }
            }
            for path in [
                "/api/legacy",
                "/_server/frontend/../admin/read",
                "/_server/frontend/",
            ] {
                assert!(!surface.owns_server_path(path));
            }
        }
    }
}

pub mod admin_escrow;

pub mod admin_chat;
pub mod admin_media;
pub mod admin_textarea;

pub mod admin_news;

pub mod admin_developer;

pub mod admin_wallets;

pub mod admin_system;

pub mod admin_notifications;

pub mod admin_payments;

pub mod core_credits;
pub mod core_wallets;
