use crate::primitives::icon::Icon;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// How the user authenticated. Mirrors the discriminator in the
/// `useSharedAuth()` TS provider — the Rust side just stores the tag
/// so the UI can render the right pill / icon.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    Wallet,
    Email,
    Demo,
    OAuth,
    Siwe,
    Unknown,
}

impl Default for AuthMethod {
    fn default() -> Self { Self::Unknown }
}

/// Existing user shape — extended with the optional auth metadata
/// (display name, last login timestamp, method tag). All new fields
/// are `Option`/`Default` so existing serialized users keep working.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub address: String,
    pub chain_id: String,
    pub roles: Vec<String>,
    pub email: Option<String>,
    pub tier: Option<String>,
    pub permissions: Vec<String>,
    /// ISO-8601 timestamp of the most recent successful sign-in.
    /// `None` when the user has never logged in (e.g. freshly
    /// provisioned by an admin).
    #[serde(default)]
    pub last_login_at: Option<String>,
    /// How the user most recently authenticated. Defaults to
    /// `AuthMethod::Unknown` for legacy records.
    #[serde(default)]
    pub auth_method: AuthMethod,
    /// Human-friendly display name (e.g. ENS, OAuth name). Used by
    /// the connected-wallet dropdown header and account overview.
    #[serde(default)]
    pub display_name: Option<String>,
}

impl User {
    pub fn is_authed(&self) -> bool { !self.id.is_empty() }

    pub fn short_address(&self) -> String {
        if self.address.len() < 10 { return self.address.clone(); }
        format!("{}…{}", &self.address[..6], &self.address[self.address.len()-4..])
    }

    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin" || r == "super_admin" || r == "Admin")
    }

    pub fn has_permission(&self, p: &str) -> bool { self.permissions.iter().any(|x| x == p) }

    /// Returns `true` when the user has the given role (case-insensitive,
    /// exact match). Empty / unknown role tags never match.
    pub fn has_role(&self, role: &str) -> bool {
        if role.is_empty() { return false; }
        self.roles.iter().any(|r| r.eq_ignore_ascii_case(role))
    }

    /// Returns `true` when the user has at least one of the given roles.
    /// Empty input list returns `false` (consistent with the
    /// "any-permission" pattern used elsewhere in the app).
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        if roles.is_empty() { return false; }
        roles.iter().any(|r| self.has_role(r))
    }

    /// Returns the display name when set, falling back to the short
    /// wallet address, then to the bare email local-part, then to
    /// a generic "Guest" placeholder. Never returns an empty string.
    pub fn display_name_or_fallback(&self) -> String {
        if let Some(n) = &self.display_name {
            if !n.is_empty() { return n.clone(); }
        }
        if !self.address.is_empty() { return self.short_address(); }
        if let Some(email) = &self.email {
            if let Some(at) = email.find('@') {
                return email[..at].to_string();
            }
            return email.clone();
        }
        "Guest".to_string()
    }

    /// Tiny helper to render a leading icon appropriate for the user's
    /// auth method. Useful for the connected-wallet dropdown header
    /// and the access-denied audit row. Returns a wallet icon for
    /// all on-chain methods, mail for email/OAuth, and the generic
    /// user icon for everything else.
    pub fn method_icon(&self) -> String {
        match self.auth_method {
            AuthMethod::Wallet | AuthMethod::Siwe => "wallet".to_string(),
            AuthMethod::Email => "mail".to_string(),
            AuthMethod::OAuth => "shield".to_string(),
            AuthMethod::Demo => "user".to_string(),
            AuthMethod::Unknown => {
                if self.is_authed() { "user".to_string() } else { "user".to_string() }
            }
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: String::new(),
            address: String::new(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: AuthMethod::Unknown,
            display_name: None,
        }
    }
}

/// Convenience render helper: a small "Logged in via <method>" pill
/// that the connected-wallet dropdown and account pages can drop in
/// without re-implementing the icon/label mapping.
#[component]
pub fn AuthMethodPill(user: User) -> Element {
    let label = match user.auth_method {
        AuthMethod::Wallet => "Wallet",
        AuthMethod::Siwe => "SIWE",
        AuthMethod::Email => "Email",
        AuthMethod::OAuth => "OAuth",
        AuthMethod::Demo => "Demo",
        AuthMethod::Unknown => "Signed in",
    };
    let icon = user.method_icon();
    rsx! {
        span { class: "auth-method-pill",
            span { class: "auth-method-pill-icon", Icon { name: icon, size: Some(12) } }
            span { class: "auth-method-pill-label", "{label}" }
        }
    }
}
