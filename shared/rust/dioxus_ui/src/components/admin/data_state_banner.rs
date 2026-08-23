//! `AdminDataStateBanner` — one truthful explanation per admin load failure.
//!
//! The five states mirror the BFF adapter taxonomy exactly:
//! unauthenticated (no verified session), unauthorized (upstream rejected the
//! token), forbidden (permission denied), unavailable (backend unreachable),
//! and malformed (contract violation). Sign-in actions never retry blindly.

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminDataState {
    Unauthenticated,
    Unauthorized,
    Forbidden,
    Unavailable,
    Malformed,
}

impl AdminDataState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unauthenticated => "unauthenticated",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::Unavailable => "unavailable",
            Self::Malformed => "malformed",
        }
    }

    fn copy(self, subject: &str) -> (String, String, &'static str) {
        match self {
            Self::Unauthenticated => (
                "Sign in required".to_string(),
                format!(
                    "No verified admin session reached {subject}. Data stays hidden until you complete sign-in."
                ),
                "status",
            ),
            Self::Unauthorized => (
                "Session expired".to_string(),
                format!(
                    "Your admin token was rejected while reading {subject}. Sign in again to continue."
                ),
                "status",
            ),
            Self::Forbidden => (
                "Permission required".to_string(),
                format!("The backend did not authorize this session to read {subject}."),
                "alert",
            ),
            Self::Unavailable => (
                format!("{subject} is unavailable"),
                format!(
                    "The owning backend could not provide an authoritative response for {subject}. Nothing is assumed."
                ),
                "status",
            ),
            Self::Malformed => (
                format!("{subject} could not be verified"),
                format!(
                    "The backend response did not match the strict contract for {subject}. Nothing is shown rather than guessed."
                ),
                "alert",
            ),
        }
    }
}

/// URL-component encoder matching `crate::auth::auth_gate` so a signed-in
/// bounce returns to the original admin path.
fn url_encode_query_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[component]
pub fn AdminDataStateBanner(
    state: AdminDataState,
    /// Human-readable subject, e.g. "Wallet access".
    subject: String,
    /// Path used as the `?return_url=` target after sign-in.
    return_path: String,
    /// Retry link target for transient failures.
    retry_href: String,
) -> Element {
    let (title, detail, role) = state.copy(&subject);
    let needs_sign_in = matches!(
        state,
        AdminDataState::Unauthenticated | AdminDataState::Unauthorized
    );
    let primary_label = if needs_sign_in {
        "Sign in"
    } else {
        "Retry read"
    };
    let primary_href = if needs_sign_in {
        format!("/auth?return_url={}", url_encode_query_value(&return_path))
    } else {
        retry_href.clone()
    };

    rsx! {
        section {
            class: "rounded-xl border border-amber-500/25 bg-amber-500/10 px-5 py-4",
            role,
            "data-admin-data-state": state.as_str(),
            div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                div { class: "min-w-0",
                    h2 { class: "font-semibold text-foreground", "{title}" }
                    p { class: "mt-1 max-w-3xl text-sm text-muted-foreground", "{detail}" }
                }
                nav { class: "flex shrink-0 flex-wrap gap-2",
                    a { class: "btn btn-sm btn-outline", href: primary_href, "{primary_label}" }
                    a { class: "btn btn-sm btn-ghost", href: "/", "Admin home" }
                }
            }
        }
    }
}
