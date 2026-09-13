//! Frontend-only document chrome. Admin and Pay keep their existing shells.
use epsx_templates::{design_system_head_with_keywords, global_js};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::LazyLock,
};

fn lucide(name: &str) -> String {
    epsx_templates::lucide(name, "20", "")
}

pub const CSS: &str = include_str!("../public/enterprise.css");
// Keep cached styles in step with the markup shipped by this binary.
static CSS_VERSION: LazyLock<String> = LazyLock::new(|| {
    let mut hasher = DefaultHasher::new();
    CSS.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shell {
    Marketing,
    Workspace,
    Auth,
}

pub fn shell(path: &str) -> Shell {
    match path {
        "/auth" => Shell::Auth,
        "/" | "/index" | "/about" | "/contact" | "/manual" | "/privacy" | "/terms" | "/offline" => {
            Shell::Marketing
        }
        _ => Shell::Workspace,
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn active(path: &str, href: &str) -> bool {
    if matches!(href, "/account" | "/developer") {
        return path == href;
    }
    path == href || (href != "/" && path.starts_with(&format!("{href}/")))
}

fn link(path: &str, href: &str, label: &str, icon: &str) -> String {
    let current = if active(path, href) {
        " aria-current=\"page\""
    } else {
        ""
    };
    format!("<a class=\"fe-nav-link\" href=\"{href}\"{current} title=\"{label}\">{}<span class=\"fe-nav-label\">{label}</span></a>", lucide(icon))
}

type NavItem = (&'static str, &'static str, &'static str);
const MARKET: &[NavItem] = &[
    ("/analytics", "Explore", "search"),
    ("/portfolio", "Saved companies", "heart"),
    ("/plans", "Plans", "layers"),
    ("/news", "News", "newspaper"),
];
const ACCOUNT: &[NavItem] = &[
    ("/account", "Settings", "settings"),
    ("/dashboard", "Overview", "layout-dashboard"),
    ("/profile", "Profile", "user"),
    ("/permissions", "Access", "shield"),
    ("/account/credits", "Credits", "coins"),
    ("/payment", "Billing", "wallet"),
    ("/account/payments", "Purchases", "file-text"),
];
const DEVELOPER: &[NavItem] = &[
    ("/developer", "API keys", "key"),
    ("/developer/usage", "API usage", "chart-line"),
    ("/developer/docs", "API documentation", "book-open"),
];
const COMPANY: &[NavItem] = &[
    ("/about", "About", "info"),
    ("/contact", "Contact", "mail"),
    ("/chat", "Support", "message-circle"),
];

fn links(path: &str, items: &[NavItem]) -> String {
    items
        .iter()
        .map(|(href, label, icon)| link(path, href, label, icon))
        .collect()
}

fn marketing_group(path: &str, name: &str, items: &[NavItem]) -> String {
    format!("<details class=\"fe-nav-group\" name=\"fe-marketing-menu\"><summary>{name}{}</summary><nav aria-label=\"{name}\">{}</nav></details>", lucide("chevron-down"), links(path, items))
}

fn brand() -> &'static str {
    "<a class=\"fe-brand\" href=\"/\" aria-label=\"EPSX home\"><img src=\"/public/logos/epsx-icon.svg\" alt=\"\" width=\"30\" height=\"30\"><span class=\"fe-nav-label\">EPSX</span></a>"
}

fn theme() -> String {
    format!("<button class=\"fe-icon-button\" type=\"button\" data-epsx-action=\"theme-toggle\" aria-label=\"Toggle theme\" title=\"Toggle theme\">{}</button>", lucide("sun"))
}

fn wallet_menu(wallet: Option<&str>) -> String {
    let address = wallet.filter(|value| {
        value.len() == 42
            && value.starts_with("0x")
            && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
    });
    let short = address
        .map(|value| format!("{}…{}", &value[..6], &value[38..]))
        .unwrap_or_else(|| "Your wallet".into());
    let identity = wallet.map(escape).unwrap_or_else(|| "Your account".into());
    let copy = address.map(|value| format!(r#"<button class="fe-wallet-copy" type="button" data-epsx-action="copy" data-copy="{value}" data-copy-status="fe-wallet-copy-status">{}<span>Copy address</span></button><span id="fe-wallet-copy-status" class="fe-wallet-copy-status" role="status" aria-live="polite"></span>"#, lucide("copy"))).unwrap_or_default();
    format!(
        r#"<details class="fe-account-menu"><summary class="fe-wallet-trigger" aria-label="Account menu"><span class="fe-wallet-avatar" aria-hidden="true">{wallet_icon}</span><span class="fe-wallet-short">{short}</span>{chevron}</summary>
      <div class="fe-account-popover fe-wallet-popover">
        <div class="fe-wallet-card"><div class="fe-wallet-card-top"><span class="fe-wallet-avatar" aria-hidden="true">{wallet_icon}</span><div><strong>Your wallet</strong><span class="fe-wallet-session"><span aria-hidden="true"></span>Signed in with wallet</span></div></div><code class="fe-account-identity" title="{identity}">{short}</code>{copy}</div>
        <nav class="fe-wallet-links" aria-label="Wallet account"><a href="/dashboard">{overview}<span>Overview</span></a><a href="/account">{user}<span>Account settings</span></a><a href="/developer">{code}<span>Developer</span></a></nav>
        <button class="fe-wallet-disconnect" type="button" data-epsx-action="logout" data-epsx-logout-target="/">{disconnect}<span>Disconnect</span></button>
      </div></details>"#,
        wallet_icon = lucide("wallet"),
        chevron = lucide("chevron-down"),
        overview = lucide("layout-dashboard"),
        user = lucide("user"),
        code = lucide("code"),
        disconnect = lucide("log-out")
    )
}

pub fn navigation(path: &str, target: &str, signed_in: bool, wallet: Option<&str>) -> String {
    if shell(path) == Shell::Auth {
        return String::new();
    }
    let account = if signed_in {
        wallet_menu(wallet)
    } else {
        // The header must preserve the same local-return safety as sign-in.
        let target = if target.starts_with('/')
            && !target.starts_with("//")
            && !target.contains('\\')
            && !target.chars().any(char::is_control)
        {
            target
        } else {
            "/"
        };
        let encoded: String = url::form_urlencoded::byte_serialize(target.as_bytes()).collect();
        format!("<a class=\"fe-button fe-primary fe-connect-wallet\" aria-label=\"Connect wallet\" title=\"Connect wallet\" data-epsx-auth-link href=\"/auth?return_url={encoded}\">{}<span>Connect wallet</span></a>", lucide("wallet"))
    };
    let preview = if std::env::var("EPSX_ENV").as_deref() == Ok("local")
        && std::env::var("EPSX_UI_PREVIEW").as_deref() == Ok("fixture")
    {
        "<span class=\"fe-preview-indicator\">Preview data</span>"
    } else {
        ""
    };
    let tools = format!(
        "<div class=\"fe-header-tools\">{preview}{}{}{account}</div>",
        if signed_in {
            format!("<a class=\"fe-icon-button\" href=\"/notifications\"{} aria-label=\"Notifications\" title=\"Notifications\">{}</a>", if path == "/notifications" { " aria-current=\"page\"" } else { "" }, lucide("bell"))
        } else {
            String::new()
        },
        theme()
    );
    let main_links = links(path, MARKET);
    let account_links = links(path, ACCOUNT);
    let developer_links = links(path, DEVELOPER);
    let company_links = links(path, COMPANY);
    if shell(path) == Shell::Marketing {
        return format!("<header class=\"fe-marketing-header\">{}<nav class=\"fe-marketing-nav\" aria-label=\"Primary\">{}{}{}</nav>{tools}</header><details class=\"fe-public-mobile\"><summary>Menu</summary><nav aria-label=\"Mobile navigation\"><p class=\"fe-nav-caption\">MARKET</p>{main_links}<p class=\"fe-nav-caption\">DEVELOPER</p>{developer_links}<p class=\"fe-nav-caption\">COMPANY</p>{company_links}<a class=\"fe-nav-link\" href=\"/account\">Account</a></nav></details>",brand(),marketing_group(path,"Market",MARKET),marketing_group(path,"Developer",DEVELOPER),marketing_group(path,"Company",COMPANY));
    }
    format!(
        r#"<aside id="fe-sidebar" class="fe-sidebar" aria-label="Workspace navigation">
      <div class="fe-sidebar-brand">{}<button type="button" id="fe-nav-desktop-trigger" class="fe-icon-button fe-nav-desktop-trigger" data-epsx-action="fe-nav-toggle" aria-controls="fe-sidebar" aria-expanded="true" aria-label="Toggle navigation" title="Toggle navigation">{}</button><button type="button" class="fe-icon-button fe-drawer-close" data-epsx-action="fe-nav-close" aria-label="Close navigation">{}</button></div>
      <div class="fe-sidebar-scroll"><p class="fe-nav-caption">YOUR WORKSPACE</p><nav aria-label="Primary">{main_links}</nav>
      <p class="fe-nav-caption fe-nav-section">ACCOUNT</p><nav aria-label="Account">{account_links}</nav>
      <p class="fe-nav-caption fe-nav-section">DEVELOPER</p><nav aria-label="Developer">{developer_links}</nav>
      <p class="fe-nav-caption fe-nav-section">COMPANY</p><nav aria-label="Company">{company_links}</nav></div>
      <div class="fe-sidebar-bottom">{tools}<div class="fe-sidebar-legal"><a href="/privacy">Privacy</a><a href="/terms">Terms</a></div></div>
    </aside><button class="fe-drawer-overlay" data-epsx-action="fe-nav-close" type="button" aria-label="Close navigation" tabindex="-1"></button>"#,
        brand(),
        lucide("menu"),
        lucide("x"),
    )
}

fn mobile_navigation() -> String {
    format!(
        r#"<button type="button" id="fe-nav-trigger" class="fe-icon-button fe-nav-mobile-trigger" data-epsx-action="fe-nav-toggle" aria-controls="fe-sidebar" aria-expanded="false" aria-label="Open navigation">{}</button>"#,
        lucide("menu")
    )
}

fn footer() -> &'static str {
    "<footer class=\"fe-footer\"><div><strong>EPSX</strong><p>Financial Technology Platform</p></div><nav aria-label=\"Footer\"><a href=\"/about\">About</a><a href=\"/news\">News</a><a href=\"/contact\">Contact</a><a href=\"/chat\">Support</a><a href=\"/developer\">Developer</a><a href=\"/terms\">Terms</a><a href=\"/privacy\">Privacy</a></nav></footer>"
}

pub fn document(
    path: &str,
    title: &str,
    description: &str,
    keywords: Option<&str>,
    nav: &str,
    body: &str,
) -> String {
    let description = if description == "EPSX"
        || description
            == "EPSX — Explore reported company data and organize the companies you follow."
    {
        "EPSX — Financial Technology Platform. Explore EPSX rankings, see upcoming company reports, and save companies to revisit."
    } else {
        description
    };
    let kind = match shell(path) {
        Shell::Marketing => "marketing",
        Shell::Workspace => "workspace",
        Shell::Auth => "auth",
    };
    let footer = if shell(path) == Shell::Marketing {
        footer()
    } else {
        ""
    };
    let css_version = CSS_VERSION.as_str();
    format!(
        r##"<!DOCTYPE html><html lang="en" data-epsx-frontend="true"><head>{}<link rel="stylesheet" href="/public/enterprise.css?v={css_version}">{}</head><body class="epsx-frontend fe-{kind}"><a class="epsx-skip-link" href="#epsx-main-content">Skip to main content</a>{nav}<main id="epsx-main-content" tabindex="-1" class="fe-main">{}<div class="fe-content" data-fe-page="{}">{body}</div></main>{footer}</body></html>"##,
        design_system_head_with_keywords(title, description, keywords),
        global_js(),
        if shell(path) == Shell::Workspace {
            mobile_navigation()
        } else {
            String::new()
        },
        escape(path)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workspace_has_one_navigation_home_and_exact_current_destination() {
        for (path, current) in [
            ("/analytics", "/analytics"),
            ("/plans", "/plans"),
            ("/account", "/account"),
            ("/profile", "/profile"),
            ("/dashboard", "/dashboard"),
            ("/permissions", "/permissions"),
            ("/account/credits", "/account/credits"),
            ("/payment", "/payment"),
            ("/payment/order-123", "/payment"),
            ("/account/payments/order-123", "/account/payments"),
            ("/developer", "/developer"),
            ("/developer/usage", "/developer/usage"),
            ("/developer/docs", "/developer/docs"),
        ] {
            for signed_in in [false, true] {
                let nav = navigation(
                    path,
                    path,
                    signed_in,
                    Some("0x0305000000000000000000000000000000007494"),
                );
                let html = document(path, "Test", "Test", None, &nav, "Content");
                assert!(!html.contains("fe-workspace-header"));
                assert!(!html.contains("fe-subnav"));
                assert_eq!(html.matches("id=\"fe-sidebar\"").count(), 1);
                assert_eq!(html.matches("id=\"fe-nav-trigger\"").count(), 1);
                assert_eq!(html.matches("id=\"fe-nav-desktop-trigger\"").count(), 1);
                let sidebar = nav.split("</aside>").next().unwrap();
                assert_eq!(
                    sidebar.matches("aria-current=\"page\"").count(),
                    1,
                    "{path}"
                );
                assert!(sidebar.contains(&format!("href=\"{current}\" aria-current=\"page\"")));
                for (href, _, _) in ACCOUNT.iter().chain(DEVELOPER.iter()) {
                    assert!(sidebar.contains(&format!("href=\"{href}\"")));
                }
                let bottom = sidebar.split("fe-sidebar-bottom").nth(1).unwrap();
                assert!(bottom.contains("theme-toggle"));
                assert_eq!(bottom.contains("fe-account-menu"), signed_in);
                assert_eq!(bottom.contains("Connect wallet"), !signed_in);
                if signed_in {
                    assert!(bottom.contains("/notifications"));
                    assert!(
                        bottom.contains("data-copy=\"0x0305000000000000000000000000000000007494\"")
                    );
                    assert!(bottom.contains("data-epsx-logout-target=\"/\""));
                }
            }
        }
    }

    #[test]
    fn public_and_auth_shells_do_not_gain_workspace_controls() {
        for path in ["/", "/about", "/auth"] {
            let nav = navigation(path, path, false, None);
            let html = document(path, "Test", "Test", None, &nav, "Content");
            assert!(!html.contains("id=\"fe-sidebar\""));
            assert!(!html.contains("fe-nav-trigger"));
            assert_eq!(html.contains("fe-marketing-header"), path != "/auth");
        }
    }

    #[test]
    fn shell_is_frontend_only_and_keeps_one_landmark() {
        let html = document("/analytics", "Explore", "Company data", None, "", "body");
        assert_eq!(html.matches("<main ").count(), 1);
        assert!(html.contains("fe-workspace"));
        assert_eq!(shell("/index"), Shell::Marketing);
        assert!(!html.contains("fe-footer"));
        assert!(document("/", "Home", "Data", None, "", "body").contains("fe-footer"));
    }
    #[test]
    fn navigation_escapes_identity_and_preserves_return_query() {
        let html = navigation("/analytics", "/analytics?country=US&page=2", false, None);
        assert!(html.contains("return_url=%2Fanalytics%3Fcountry%3DUS%26page%3D2"));
        assert!(html.contains("href=\"/analytics\" aria-current=\"page\""));
        assert!(!navigation(
            "/analytics",
            "/analytics",
            true,
            Some("<script>bad</script>")
        )
        .contains("<script>bad"));
        assert!(navigation("/auth", "/auth", false, None).is_empty());
    }
}
