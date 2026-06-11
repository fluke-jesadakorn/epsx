//! Admin pages — 1:1 mirror of `apps/admin-frontend/app/**/page.tsx`.

use dioxus::prelude::*;
use crate::primitives::*;
use super::{PageContext, PageMeta};
use super::not_found;

pub mod dashboard;
pub mod analytics;
pub mod audit_log;
pub mod chat;
pub mod developer_portal;
pub mod media;
pub mod news;
pub mod notifications;
pub mod notifications_redirect;
pub mod payments;
pub mod settings;
pub mod unauthorized;
pub mod auth_redirect;
pub mod access_denied;
pub mod wallet_redirect;
pub mod wallet_wallets;
pub mod wallet_credits;
pub mod wallet_access;
pub mod wallet_plans;
pub mod policies;

pub fn dispatch(ctx: &PageContext) -> (PageMeta, Element) {
    let p = ctx.path.as_str();
    match p {
        "/" | "/index" => dashboard::render(ctx),
        "/analytics" => analytics::render(ctx),
        "/audit-log" => audit_log::render(ctx),
        "/chat" => chat::render(ctx),
        "/developer-portal" => developer_portal::render(ctx),
        "/developer-portal/api-keys/create" => developer_portal::render_create_key(ctx),
        "/media" => media::render(ctx),
        "/news" => news::render(ctx),
        "/news/create" => news::render_create(ctx),
        "/notifications" => notifications_redirect::render(ctx),
        "/notifications/create" => notifications::render_create(ctx),
        "/notifications/manage" => notifications::render_manage(ctx),
        "/payments" => payments::render(ctx),
        "/policies" => policies::render(ctx),
        "/settings" => settings::render(ctx),
        "/unauthorized" => unauthorized::render(ctx),
        "/auth" => auth_redirect::render(ctx),
        "/access-denied" => access_denied::render(ctx),
        "/wallet-management" => wallet_redirect::render(ctx),
        "/wallet-management/wallets" => wallet_wallets::render(ctx),
        "/wallet-management/credits" => wallet_credits::render(ctx),
        "/wallet-management/access" => wallet_access::render(ctx),
        "/wallet-management/access/plans" => wallet_plans::render(ctx),
        _ => {
            if p.starts_with("/chat/") {
                let id = p.trim_start_matches("/chat/").trim_end_matches('/').to_string();
                let mut c = ctx.clone();
                c.params.insert("id".into(), id);
                chat::render_conversation(&c)
            } else if p.starts_with("/news/") && p.ends_with("/edit") {
                let rest = p.trim_start_matches("/news/").trim_end_matches("/edit").trim_end_matches('/');
                let mut c = ctx.clone();
                c.params.insert("id".into(), rest.to_string());
                news::render_edit(&c)
            } else if p.starts_with("/wallet-management/wallets/") && p.ends_with("/disable") {
                let rest = p.trim_start_matches("/wallet-management/wallets/").trim_end_matches("/disable").trim_end_matches('/');
                let mut c = ctx.clone();
                c.params.insert("address".into(), rest.to_string());
                wallet_wallets::render_disable(&c)
            } else if p.starts_with("/wallet-management/access/plans/") {
                let rest = p.trim_start_matches("/wallet-management/access/plans/").trim_end_matches('/');
                let mut c = ctx.clone();
                c.params.insert("planId".into(), rest.to_string());
                wallet_plans::render_editor(&c)
            } else if p.starts_with("/wallet-management/") {
                let addr = p.trim_start_matches("/wallet-management/").trim_end_matches('/');
                if !addr.is_empty() && !addr.contains('/') {
                    let mut c = ctx.clone();
                    c.params.insert("address".into(), addr.to_string());
                    wallet_wallets::render_detail(&c)
                } else {
                    not_found::render(ctx)
                }
            } else {
                not_found::render(ctx)
            }
        }
    }
}
