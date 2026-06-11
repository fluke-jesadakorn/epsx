//! /profile — user profile with 4-tab layout.
//!
//! Wave 6C Track E — the 12 profile sub-components were extracted
//! to `crate::components::user::profile`. The `tab_class` helper
//! is also lifted and `pub`.

use crate::components::user::profile::RenderProfile;
use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Profile");
    (meta, rsx! { RenderProfile { ctx: ctx.clone() } })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use crate::auth::user::AuthMethod;

    fn authed_ctx(path: &str) -> PageContext {
        let user = User {
            id: "u1".to_string(),
            address: "0x1234…abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("test@epsx.io".to_string()),
            tier: Some("Pro".to_string()),
            permissions: vec![
                "profile:read".to_string(),
                "profile:write".to_string(),
            ],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: Some("Test".to_string()),
        };
        PageContext { user: Some(user), path: path.to_string(), ..Default::default() }
    }

    #[test]
    fn test_render_smoke() {
        let ctx = authed_ctx("/profile");
        let (_meta, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Profile &#38; Settings"), "/profile header must render. Got: {}", html);
    }

    #[test]
    fn test_section_markers() {
        let ctx = authed_ctx("/profile");
        let (_meta, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);
        for marker in [
            "profile-tab-nav",
            "wallet-profile-sidebar",
            "profile-tab-panels",
        ] {
            assert!(html.contains(marker), "missing section marker: {}", marker);
        }
    }
}
