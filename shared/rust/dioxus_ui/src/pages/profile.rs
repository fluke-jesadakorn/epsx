//! `/profile` — a read-only view of the locally verified session.
//!
//! The server-rendered page intentionally limits itself to claims supplied by
//! the verified access token. Profile mutations remain unavailable until they
//! have backend endpoints and an interactive, authenticated client flow.

use super::{PageContext, PageMeta};
use crate::auth::{user::AuthMethod, AuthGate, User};
use crate::layout::{main_layout::MainLayout, PageHeader};
use crate::primitives::Icon;
use dioxus::prelude::*;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Profile");
    (meta, rsx! { RenderProfile { ctx: ctx.clone() } })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProfileTab {
    Web3,
    Account,
    Email,
    Data,
}

impl ProfileTab {
    const ALL: [Self; 4] = [Self::Web3, Self::Account, Self::Email, Self::Data];

    fn from_exact(value: &str) -> Option<Self> {
        match value {
            "web3" => Some(Self::Web3),
            "account" => Some(Self::Account),
            "email" => Some(Self::Email),
            "data" => Some(Self::Data),
            _ => None,
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::Web3 => "web3",
            Self::Account => "account",
            Self::Email => "email",
            Self::Data => "data",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Web3 => "Web3",
            Self::Account => "Account",
            Self::Email => "Email",
            Self::Data => "Data",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Web3 => "shield",
            Self::Account => "settings",
            Self::Email => "mail",
            Self::Data => "database",
        }
    }
}

/// Select only one literal `tab` value. Unknown, encoded, malformed, or
/// duplicate values fall back to the least surprising default.
fn selected_profile_tab(query: &str) -> ProfileTab {
    let mut selected = None;
    let mut saw_tab = false;

    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next() != Some("tab") {
            continue;
        }

        if saw_tab {
            return ProfileTab::Web3;
        }
        saw_tab = true;
        selected = parts.next().and_then(ProfileTab::from_exact);
    }

    selected.unwrap_or(ProfileTab::Web3)
}

#[component]
fn RenderProfile(ctx: PageContext) -> Element {
    let selected_tab = selected_profile_tab(&ctx.query);

    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("your profile".to_string()),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-6xl",
                    PageHeader {
                        title: "Profile & Settings".to_string(),
                        description: Some("Review your authenticated wallet and backend-issued permissions.".to_string()),
                        icon: Some("user".to_string())
                    }
                    if let Some(user) = ctx.user.clone() {
                        ProfileBody { user, selected_tab }
                    }
                }
            }
        }
    }
}

#[component]
fn ProfileBody(user: User, selected_tab: ProfileTab) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 lg:grid-cols-4 gap-6",
            div { class: "lg:col-span-1",
                WalletProfile { user: user.clone() }
            }
            div { class: "lg:col-span-3",
                ProfileTabNav { selected_tab }
                div { class: "profile-tab-panels",
                    ProfilePanel { user, selected_tab }
                }
            }
        }
    }
}

#[component]
fn ProfileTabNav(selected_tab: ProfileTab) -> Element {
    rsx! {
        nav {
            class: "tabs profile-tab-nav mb-4",
            "aria-label": "Profile sections",
            for tab in ProfileTab::ALL {
                {
                    let active = tab == selected_tab;
                    let key = tab.key();
                    let class = if active { "btn btn-primary" } else { "btn btn-outline" };
                    rsx! {
                        a {
                            class,
                            href: "/profile?tab={key}",
                            id: "profile-tab-{key}",
                            "aria-current": if active { "page" } else { "false" },
                            Icon { name: tab.icon().to_string(), size: Some(16) }
                            " {tab.label()}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn WalletProfile(user: User) -> Element {
    let permission_count = user.permissions.len();
    let auth_method = auth_method_label(&user.auth_method);

    rsx! {
        aside { class: "card card-glass wallet-profile-sidebar",
            div { class: "card-body text-center",
                div { class: "mx-auto mb-4 h-20 w-20 rounded-full bg-gradient-to-br from-orange-400 to-pink-500 flex items-center justify-center",
                    Icon { name: "wallet".to_string(), size: Some(40) }
                }
                h3 { class: "text-lg font-bold", "Authenticated wallet" }
                div { class: "mt-2 flex justify-center",
                    span { class: "badge badge-success", "Session active" }
                }
                dl { class: "mt-4 pt-4 border-t border-border text-left space-y-3 text-sm",
                    div {
                        dt { class: "text-muted-foreground", "Address" }
                        dd { class: "font-mono text-xs break-all", "{user.address}" }
                    }
                    div { class: "flex justify-between gap-4",
                        dt { class: "text-muted-foreground", "Authentication method" }
                        dd { class: "font-medium", "{auth_method}" }
                    }
                    div { class: "flex justify-between gap-4",
                        dt { class: "text-muted-foreground", "Backend permissions" }
                        dd { class: "font-medium", "{permission_count}" }
                    }
                }
            }
        }
    }
}

#[component]
fn ProfilePanel(user: User, selected_tab: ProfileTab) -> Element {
    let key = selected_tab.key();

    rsx! {
        section {
            id: "profile-panel-{key}",
            class: "profile-panel profile-{key}-panel",
            "aria-labelledby": "profile-tab-{key}",
            match selected_tab {
                ProfileTab::Web3 => rsx! { Web3SessionPanel { user } },
                ProfileTab::Account => rsx! { AccountPanel { user } },
                ProfileTab::Email => rsx! { EmailPanel {} },
                ProfileTab::Data => rsx! { DataPanel { user } },
            }
        }
    }
}

#[component]
fn Web3SessionPanel(user: User) -> Element {
    let auth_method = auth_method_label(&user.auth_method);

    rsx! {
        div { class: "space-y-6 web3-integration-panel",
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title flex items-center gap-2",
                        Icon { name: "wallet".to_string(), size: Some(20) }
                        "Authenticated Wallet Session"
                    }
                    p { class: "text-sm text-muted-foreground",
                        "These values come from the locally verified access token."
                    }
                }
                div { class: "card-body space-y-5",
                    ClaimRow { label: "Wallet address".to_string(), value: user.address.clone(), monospace: true }
                    ClaimRow { label: "Authentication method".to_string(), value: auth_method.to_string(), monospace: false }
                    PermissionList { permissions: user.permissions.clone() }
                }
            }
            UnavailableNotice {
                icon: "info".to_string(),
                title: "Wallet connection details unavailable".to_string(),
                body: "Connector, provider, network, and chain details are not included in the verified session claims shown here.".to_string()
            }
            UnavailableNotice {
                icon: "key".to_string(),
                title: "API key management unavailable".to_string(),
                body: "API key creation and management are not available in this read-only migration.".to_string()
            }
        }
    }
}

#[component]
fn AccountPanel(user: User) -> Element {
    let auth_method = auth_method_label(&user.auth_method);

    rsx! {
        div { class: "space-y-6",
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title flex items-center gap-2",
                        Icon { name: "settings".to_string(), size: Some(20) }
                        "Verified Session Claims"
                    }
                }
                div { class: "card-body space-y-5",
                    ClaimRow { label: "Session subject".to_string(), value: user.id.clone(), monospace: true }
                    ClaimRow { label: "Wallet address".to_string(), value: user.address.clone(), monospace: true }
                    ClaimRow { label: "Authentication method".to_string(), value: auth_method.to_string(), monospace: false }
                    PermissionList { permissions: user.permissions.clone() }
                }
            }
            UnavailableNotice {
                icon: "info".to_string(),
                title: "Additional account details unavailable".to_string(),
                body: "Email, role, plan tier, profile name, and account verification status are not asserted by the verified session used for this page.".to_string()
            }
        }
    }
}

#[component]
fn EmailPanel() -> Element {
    rsx! {
        div { class: "space-y-6",
            UnavailableNotice {
                icon: "mail".to_string(),
                title: "Email management unavailable".to_string(),
                body: "The verified session does not provide an email-management workflow for this read-only page.".to_string()
            }
            UnavailableNotice {
                icon: "settings".to_string(),
                title: "Email preferences unavailable".to_string(),
                body: "Notification and marketing preferences cannot be viewed or changed in this migration yet.".to_string()
            }
        }
    }
}

#[component]
fn DataPanel(user: User) -> Element {
    let permission_count = user.permissions.len();

    rsx! {
        div { class: "space-y-6",
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title flex items-center gap-2",
                        Icon { name: "database".to_string(), size: Some(20) }
                        "Session Data Summary"
                    }
                }
                div { class: "card-body grid grid-cols-1 md:grid-cols-2 gap-4",
                    ClaimRow { label: "Wallet address".to_string(), value: user.address.clone(), monospace: true }
                    ClaimRow { label: "Backend permission count".to_string(), value: permission_count.to_string(), monospace: false }
                }
            }
            UnavailableNotice {
                icon: "download".to_string(),
                title: "Data export unavailable".to_string(),
                body: "No authenticated export endpoint is connected to this read-only page.".to_string()
            }
            UnavailableNotice {
                icon: "alert-triangle".to_string(),
                title: "Account deletion unavailable".to_string(),
                body: "Account deletion is not exposed without a backend-owned confirmation and deletion workflow.".to_string()
            }
        }
    }
}

#[component]
fn ClaimRow(label: String, value: String, monospace: bool) -> Element {
    let value_class = if monospace {
        "mt-1 font-mono text-sm break-all"
    } else {
        "mt-1 text-sm"
    };

    rsx! {
        div { class: "profile-claim rounded-lg bg-muted p-3",
            div { class: "text-sm font-medium text-muted-foreground", "{label}" }
            div { class: value_class, "{value}" }
        }
    }
}

#[component]
fn PermissionList(permissions: Vec<String>) -> Element {
    let permission_count = permissions.len();

    rsx! {
        div { class: "profile-permissions",
            h4 { class: "text-sm font-medium", "Backend-issued permissions ({permission_count})" }
            if permissions.is_empty() {
                p { class: "mt-2 text-sm text-muted-foreground",
                    "No backend permissions were issued for this session."
                }
            } else {
                ul { class: "mt-2 flex flex-wrap gap-2",
                    for permission in permissions {
                        li { class: "badge badge-outline font-mono", "{permission}" }
                    }
                }
            }
        }
    }
}

#[component]
fn UnavailableNotice(icon: String, title: String, body: String) -> Element {
    rsx! {
        div {
            class: "card card-glass profile-unavailable-notice",
            role: "note",
            div { class: "card-body",
                h3 { class: "text-base font-bold flex items-center gap-2",
                    Icon { name: icon, size: Some(18) }
                    "{title}"
                }
                p { class: "mt-2 text-sm text-muted-foreground", "{body}" }
            }
        }
    }
}

fn auth_method_label(method: &AuthMethod) -> &'static str {
    match method {
        AuthMethod::Wallet => "Wallet",
        AuthMethod::Email => "Email",
        AuthMethod::Demo => "Demo",
        AuthMethod::OAuth => "OAuth",
        AuthMethod::Siwe => "SIWE",
        AuthMethod::Unknown => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_user() -> User {
        User {
            id: "session-subject-42".to_string(),
            address: "0x9abc00000000000000000000000000000000def0".to_string(),
            chain_id: "ignored-chain-claim".to_string(),
            roles: vec!["ignored-role-claim".to_string()],
            email: Some("ignored-email@example.invalid".to_string()),
            tier: Some("ignored-tier-claim".to_string()),
            permissions: vec!["reports:read".to_string(), "billing:view".to_string()],
            last_login_at: Some("ignored-last-login".to_string()),
            auth_method: AuthMethod::Siwe,
            display_name: Some("ignored-display-name".to_string()),
        }
    }

    fn page_ctx(query: &str, user: Option<User>) -> PageContext {
        PageContext {
            user,
            path: "/profile".to_string(),
            query: query.to_string(),
            ..Default::default()
        }
    }

    fn render_page(query: &str, user: Option<User>) -> String {
        let ctx = page_ctx(query, user);
        let (_, element) = render(&ctx);
        dioxus_ssr::render_element(element)
    }

    fn render_body(tab: ProfileTab, user: User) -> String {
        dioxus_ssr::render_element(rsx! { ProfileBody { user, selected_tab: tab } })
    }

    #[test]
    fn profile_tab_selection_is_exact_and_fail_closed() {
        assert_eq!(selected_profile_tab(""), ProfileTab::Web3);
        assert_eq!(selected_profile_tab("tab=web3"), ProfileTab::Web3);
        assert_eq!(selected_profile_tab("tab=account"), ProfileTab::Account);
        assert_eq!(selected_profile_tab("foo=1&tab=email"), ProfileTab::Email);
        assert_eq!(selected_profile_tab("tab=data&foo=1"), ProfileTab::Data);

        for query in [
            "tab=Account",
            "tab=%64ata",
            "tab=unknown",
            "tab=",
            "tab",
            "tab=data&tab=email",
            "tab&tab=data",
            "xtab=data",
        ] {
            assert_eq!(
                selected_profile_tab(query),
                ProfileTab::Web3,
                "query must fail closed: {query}"
            );
        }
    }

    #[test]
    fn native_tab_links_render_only_the_selected_panel() {
        for selected_tab in ProfileTab::ALL {
            let html = render_body(selected_tab, session_user());

            for tab in ProfileTab::ALL {
                assert!(html.contains(&format!("href=\"/profile?tab={}\"", tab.key())));
            }
            assert_eq!(html.matches("aria-current=\"page\"").count(), 1);
            assert!(html.contains(&format!("id=\"profile-panel-{}\"", selected_tab.key())));
            for other_tab in ProfileTab::ALL {
                if other_tab != selected_tab {
                    assert!(!html.contains(&format!("id=\"profile-panel-{}\"", other_tab.key())));
                }
            }
        }
    }

    #[test]
    fn authenticated_user_without_profile_permissions_renders() {
        let mut user = session_user();
        user.permissions = vec!["rankings:read".to_string()];
        let html = render_page("tab=account", Some(user));

        assert!(html.contains("Profile &#38; Settings"));
        assert!(html.contains("session-subject-42"));
        assert!(html.contains("rankings:read"));
        assert!(!html.contains("Permission required"));
        assert!(!html.contains("profile:read"));
        assert!(!html.contains("profile:write"));
    }

    #[test]
    fn renders_only_exact_verified_claims_and_escapes_hostile_text() {
        let mut user = session_user();
        user.id = "subject-<script>alert('subject')</script>".to_string();
        user.address = "wallet-<img src=x onerror=alert('address')>".to_string();
        user.permissions = vec!["reports:<script>alert('permission')</script>".to_string()];
        let html = render_page("tab=account", Some(user));

        assert!(html.contains("subject-&#60;script&#62;alert(&#39;subject&#39;)&#60;/script&#62;"));
        assert!(html.contains("wallet-&#60;img src=x onerror=alert(&#39;address&#39;)&#62;"));
        assert!(
            html.contains("reports:&#60;script&#62;alert(&#39;permission&#39;)&#60;/script&#62;")
        );
        assert!(html.contains("SIWE"));
        assert!(!html.contains("<script>alert('subject')</script>"));
        assert!(!html.contains("<img src=x onerror=alert('address')>"));

        for ignored_claim in [
            "ignored-chain-claim",
            "ignored-role-claim",
            "ignored-email@example.invalid",
            "ignored-tier-claim",
            "ignored-last-login",
            "ignored-display-name",
        ] {
            assert!(
                !html.contains(ignored_claim),
                "rendered unverified claim: {ignored_claim}"
            );
        }
    }

    #[test]
    fn fixture_values_and_unsupported_controls_are_absent() {
        let html = ProfileTab::ALL
            .into_iter()
            .map(|tab| render_body(tab, session_user()))
            .collect::<String>();

        for unavailable in [
            "API key management unavailable",
            "Email management unavailable",
            "Email preferences unavailable",
            "Data export unavailable",
            "Account deletion unavailable",
        ] {
            assert!(html.contains(unavailable), "missing notice: {unavailable}");
        }

        for fixture_or_control in [
            "0x1234…abcd",
            "Pro tier",
            "Connected via MetaMask",
            "trade:read",
            "payments:read",
            "profile:write",
            "DELETE MY ACCOUNT",
            "Create API Key",
            "Export Basic",
            "Send Verification Code",
            "<button",
            "<form",
            "<input",
            "onclick=",
        ] {
            assert!(
                !html.contains(fixture_or_control),
                "rendered fixture or unsupported control: {fixture_or_control}"
            );
        }
    }
}
