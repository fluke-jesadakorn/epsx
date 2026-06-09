//! Shared UI helpers for the frontend BFF.
//!
//! Wraps the `epsx-templates` design system into higher-level components
//! (navbar, mobile sheet, footer, page sections) so individual page body
//! functions can stay short and focus on content.

use crate::auth::AuthUser;
use epsx_templates::{logo, navbar_close, navbar_open, theme_toggle_button};

// =====================================================================
// Navigation data model — mirrors the original `nav-config.ts` so the
// structure stays in lockstep with the design spec.
// =====================================================================

#[derive(Debug, Clone, Copy)]
pub struct NavItem {
    pub key: &'static str,
    pub label: &'static str,
    pub href: &'static str,
    pub icon: &'static str,
    pub desc: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub struct NavGroup {
    pub key: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub items: &'static [NavItem],
}

pub const NAV_GROUPS: &[NavGroup] = &[
    NavGroup {
        key: "market",
        label: "Market",
        icon: "chart-line",
        items: &[
            NavItem {
                key: "rankings",
                label: "Rankings",
                href: "/analytics",
                icon: "chart-line",
                desc: Some("EPS stock rankings"),
            },
            NavItem {
                key: "portfolio",
                label: "Portfolio",
                href: "/portfolio",
                icon: "arrow-trend-up",
                desc: Some("Watchlist & tracking"),
            },
        ],
    },
    NavGroup {
        key: "developer",
        label: "Developer",
        icon: "code",
        items: &[
            NavItem {
                key: "api-keys",
                label: "API Keys",
                href: "/developer",
                icon: "key",
                desc: Some("Manage API access"),
            },
            NavItem {
                key: "docs",
                label: "Documentation",
                href: "/developer/docs",
                icon: "book",
                desc: Some("API reference"),
            },
        ],
    },
    NavGroup {
        key: "company",
        label: "Company",
        icon: "building",
        items: &[
            NavItem {
                key: "about",
                label: "About",
                href: "/about",
                icon: "circle-info",
                desc: None,
            },
            NavItem {
                key: "news",
                label: "News",
                href: "/news",
                icon: "newspaper",
                desc: Some("Latest updates"),
            },
            NavItem {
                key: "contact",
                label: "Contact",
                href: "/contact",
                icon: "envelope",
                desc: None,
            },
            NavItem {
                key: "support",
                label: "Support",
                href: "/chat",
                icon: "circle-question",
                desc: None,
            },
        ],
    },
];

pub const FOOTER_LINKS: &[NavItem] = &[
    NavItem { key: "terms",   label: "Terms of Service", href: "/terms",   icon: "", desc: None },
    NavItem { key: "privacy", label: "Privacy Policy",   href: "/privacy", icon: "", desc: None },
    NavItem { key: "contact", label: "Contact",          href: "/contact", icon: "", desc: None },
];

pub fn is_group_active(group: &NavGroup, pathname: &str) -> bool {
    group.items.iter().any(|i| i.href == pathname)
}

pub fn is_item_active(item: &NavItem, pathname: &str) -> bool {
    item.href == pathname
}

// =====================================================================
// Navbar
// =====================================================================

/// Render the desktop + mobile navbar. Mobile collapses to a hamburger sheet.
pub fn render_navbar(active_path: &str, is_authed: bool, user: Option<&AuthUser>) -> String {
    let desktop_groups = NAV_GROUPS
        .iter()
        .map(|g| render_nav_dropdown(g, active_path))
        .collect::<String>();

    let nav_actions = render_nav_actions(is_authed, user);

    format!(
        r##"{open}
  {logo}
  <nav class="hidden lg:flex items-center gap-1" style="margin-left:2rem;">{groups}</nav>
  <div class="flex items-center gap-2" style="margin-left:auto;">
    {actions}
    <button class="nav-link hamburger lg:hidden" onclick="epsx.openSheet('mobile-sheet')" aria-label="Open menu" style="width:2.25rem;height:2.25rem;padding:0;justify-content:center;">
      <i class="fa-solid fa-bars"></i>
    </button>
  </div>
{close}
{mobile_sheet}"##,
        open = navbar_open(),
        logo = logo("/", "md"),
        groups = desktop_groups,
        actions = nav_actions,
        close = navbar_close(),
        mobile_sheet = render_mobile_sheet(active_path, is_authed, user),
    )
}

/// Right-side actions: theme toggle + (notification bell, wallet dropdown)
/// when authed, or "Sign In" button when not.
fn render_nav_actions(is_authed: bool, user: Option<&AuthUser>) -> String {
    let theme = theme_toggle_button();
    if !is_authed {
        return format!(
            r##"{theme}
            <a href="/auth" class="btn btn-gradient btn-sm">Sign In <i class="fa-solid fa-arrow-right" style="margin-left:0.25rem;"></i></a>"##,
            theme = theme
        );
    }
    let user = user.unwrap();
    let address = user.display();
    let role = user.role_label();
    let bell = r##"<a href="/notifications" class="nav-link" style="position:relative;width:2.25rem;height:2.25rem;padding:0;justify-content:center;" title="Notifications">
        <i class="fa-solid fa-bell"></i>
        <span id="nav-unread-badge" style="display:none;position:absolute;top:-0.125rem;right:-0.125rem;background:var(--epsx-red);color:white;font-size:0.625rem;font-weight:700;min-width:1.125rem;height:1.125rem;border-radius:9999px;padding:0 0.25rem;align-items:center;justify-content:center;">0</span>
    </a>"##;
    let wallet = format!(
        r##"<div class="nav-dropdown-wrap" id="nav-wallet">
          <button type="button" class="nav-dropdown-trigger" onclick="epsx.toggleNavDropdown('nav-wallet')" style="display:flex;align-items:center;gap:0.5rem;padding:0.375rem 0.75rem;border-radius:0.5rem;background:var(--bg-secondary);border:1px solid var(--border);font-size:0.875rem;">
            <div style="width:1.5rem;height:1.5rem;border-radius:9999px;background:var(--gradient-warm);display:flex;align-items:center;justify-content:center;color:white;font-size:0.625rem;font-weight:700;">{initials}</div>
            <span style="font-weight:500;">{addr}</span>
            <i class="fa-solid fa-chevron-down chev"></i>
          </button>
          <div class="nav-dropdown" role="menu" style="min-width:14rem;">
            <div style="padding:0.5rem 0.75rem;border-bottom:1px solid var(--border);margin-bottom:0.25rem;">
              <div style="font-size:0.6875rem;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">{role}</div>
              <div style="font-size:0.8125rem;font-family:monospace;color:var(--text);word-break:break-all;">{full_addr}</div>
            </div>
            <a href="/dashboard" class="nav-dropdown-item" role="menuitem"><i class="fa-solid fa-gauge-high"></i><div><div class="nav-dropdown-item-label">Dashboard</div></div></a>
            <a href="/profile" class="nav-dropdown-item" role="menuitem"><i class="fa-solid fa-user"></i><div><div class="nav-dropdown-item-label">Profile</div></div></a>
            <a href="/account" class="nav-dropdown-item" role="menuitem"><i class="fa-solid fa-gear"></i><div><div class="nav-dropdown-item-label">Account Settings</div></div></a>
            {admin_link}
            <div style="border-top:1px solid var(--border);margin:0.25rem 0;"></div>
            <button onclick="epsx.signOut()" class="nav-dropdown-item" role="menuitem" style="color:var(--epsx-red);width:100%;background:transparent;border:none;text-align:left;cursor:pointer;"><i class="fa-solid fa-arrow-right-from-bracket"></i><div><div class="nav-dropdown-item-label">Sign Out</div></div></button>
          </div>
        </div>"##,
        initials = address.chars().take(2).collect::<String>().to_uppercase(),
        addr = address,
        role = role,
        full_addr = user.address,
        admin_link = if user.is_admin() { r##"<a href="/admin" class="nav-dropdown-item" role="menuitem"><i class="fa-solid fa-shield-halved"></i><div><div class="nav-dropdown-item-label">Admin Panel</div></div></a>"## } else { "" }
    );
    let js = r##"<script>
(function() {
  function refreshNavUnread() {
    const badge = document.getElementById('nav-unread-badge');
    if (!badge) return;
    fetch('/api/v1/notifications?limit=1').then(r => r.ok ? r.json() : null).then(data => {
      if (!data) return;
      const items = data.items || [];
      const unread = items.filter(i => !i.read_at).length;
      if (unread > 0) { badge.textContent = unread; badge.style.display = 'flex'; }
      else { badge.style.display = 'none'; }
    }).catch(() => {});
  }
  refreshNavUnread();
  setInterval(refreshNavUnread, 30000);
  // Sign-out helper
  window.epsx = window.epsx || {};
  window.epsx.signOut = async function() {
    try {
      await fetch('/api/v1/auth/logout', { method: 'POST', credentials: 'include' });
      epsx.toast('Signed out', 'success');
      setTimeout(() => location.href = '/', 400);
    } catch (e) {
      epsx.toast('Sign-out failed', 'error');
    }
  };
})();
</script>"##;
    format!(
        r##"{theme}
        {bell}
        {wallet}
        {js}"##,
        theme = theme,
        bell = bell,
        wallet = wallet,
        js = js
    )
}

/// Render a desktop dropdown group (button with chevron + popover panel).
fn render_nav_dropdown(group: &NavGroup, active_path: &str) -> String {
    let active = is_group_active(group, active_path);
    let trigger_cls = if active { "nav-dropdown-trigger active" } else { "nav-dropdown-trigger" };

    let items_html = group
        .items
        .iter()
        .map(|item| render_nav_dropdown_item(item, active_path))
        .collect::<String>();

    format!(
        r##"<div class="nav-dropdown-wrap" id="nav-{k}">
  <button type="button" class="{tc}" onclick="epsx.toggleNavDropdown('nav-{k}')" aria-haspopup="true" aria-expanded="false">
    <i class="fa-solid fa-{icon}"></i>
    <span>{label}</span>
    <i class="fa-solid fa-chevron-down chev"></i>
  </button>
  <div class="nav-dropdown" role="menu">{items}</div>
</div>"##,
        k = group.key,
        tc = trigger_cls,
        icon = group.icon,
        label = group.label,
        items = items_html,
    )
}

fn render_nav_dropdown_item(item: &NavItem, active_path: &str) -> String {
    let active = is_item_active(item, active_path);
    let cls = if active { "nav-dropdown-item active" } else { "nav-dropdown-item" };
    let desc = item
        .desc
        .map(|d| format!(r#"<div class="item-desc">{}</div>"#, html_escape_text(d)))
        .unwrap_or_default();
    format!(
        r##"<a href="{href}" class="{cls}" role="menuitem">
  <i class="fa-solid fa-{icon} item-icon"></i>
  <div style="min-width:0;">
    <div class="item-label">{label}</div>
    {desc}
  </div>
</a>"##,
        href = html_escape_attr(item.href),
        cls = cls,
        icon = item.icon,
        label = html_escape_text(item.label),
        desc = desc,
    )
}

// =====================================================================
// Mobile sheet
// =====================================================================

fn render_mobile_sheet(active_path: &str, is_authed: bool, user: Option<&AuthUser>) -> String {
    let groups = NAV_GROUPS
        .iter()
        .map(|g| render_nav_accordion(g, active_path))
        .collect::<String>();

    let user_block = if let Some(u) = user {
        format!(
            r##"<div style="margin-bottom:1rem;padding:0.75rem;border-radius:0.75rem;background:var(--bg-secondary);border:1px solid var(--border);display:flex;align-items:center;gap:0.75rem;">
              <div style="width:2.5rem;height:2.5rem;border-radius:9999px;background:var(--gradient-warm);display:flex;align-items:center;justify-content:center;color:white;font-weight:700;">{initials}</div>
              <div style="flex:1;min-width:0;">
                <div style="font-weight:600;font-size:0.9375rem;">{addr}</div>
                <div style="font-size:0.75rem;color:var(--text-muted);">{role}</div>
              </div>
            </div>"##,
            initials = u.display().chars().take(2).collect::<String>().to_uppercase(),
            addr = u.display(),
            role = u.role_label(),
        )
    } else { String::new() };

    let sign = if is_authed {
        r##"<a href="/dashboard" class="btn btn-gradient btn-block" onclick="epsx.closeSheet('mobile-sheet')">Dashboard <i class="fa-solid fa-arrow-right"></i></a>
            <button class="btn btn-outline btn-block" style="margin-top:0.5rem;" onclick="epsx.closeSheet('mobile-sheet'); epsx.signOut();">Sign Out <i class="fa-solid fa-arrow-right-from-bracket"></i></button>"##
    } else {
        r##"<a href="/auth" class="btn btn-gradient btn-block" onclick="epsx.closeSheet('mobile-sheet')">Sign In <i class="fa-solid fa-arrow-right"></i></a>"##
    };

    format!(
        r##"<div id="mobile-sheet" class="mobile-sheet">
  <div class="backdrop" onclick="epsx.closeSheet('mobile-sheet')"></div>
  <aside class="panel">
    <div class="flex justify-between items-center mb-4">
      {logo}
      <button class="nav-link" onclick="epsx.closeSheet('mobile-sheet')" aria-label="Close menu" style="width:2.25rem;height:2.25rem;padding:0;justify-content:center;">
        <i class="fa-solid fa-xmark"></i>
      </button>
    </div>
    {user_block}
    <div class="flex flex-col gap-1">{groups}</div>
    <div style="margin-top:1.5rem;border-top:1px solid var(--border);padding-top:1.5rem;">{sign}</div>
  </aside>
</div>"##,
        logo = logo("/", "md"),
        user_block = user_block,
        groups = groups,
        sign = sign,
    )
}

fn render_nav_accordion(group: &NavGroup, active_path: &str) -> String {
    let group_active = is_group_active(group, active_path);
    let trigger_cls = if group_active { "nav-accordion-trigger active" } else { "nav-accordion-trigger" };
    let accordion_cls = if group_active { "nav-accordion open" } else { "nav-accordion" };

    let items_html = group
        .items
        .iter()
        .map(|item| {
            let item_active = is_item_active(item, active_path);
            let cls = if item_active { "active" } else { "" };
            format!(
                r##"<a href="{href}" class="{cls}" onclick="epsx.closeSheet('mobile-sheet')">
  <i class="fa-solid fa-{icon}"></i>
  <span>{label}</span>
</a>"##,
                href = html_escape_attr(item.href),
                cls = cls,
                icon = item.icon,
                label = html_escape_text(item.label),
            )
        })
        .collect::<String>();

    format!(
        r##"<div class="{ac}" id="mob-nav-{k}">
  <button type="button" class="{tc}" onclick="epsx.toggleNavAccordion('mob-nav-{k}')" aria-expanded="{ae}">
    <span class="trigger-label">
      <i class="fa-solid fa-{icon}"></i>
      <span>{label}</span>
    </span>
    <i class="fa-solid fa-chevron-right chev"></i>
  </button>
  <div class="nav-accordion-content">{items}</div>
</div>"##,
        ac = accordion_cls,
        tc = trigger_cls,
        ae = if group_active { "true" } else { "false" },
        k = group.key,
        icon = group.icon,
        label = group.label,
        items = items_html,
    )
}

// =====================================================================
// Local HTML escape helpers (kept private to this module — components
// crate has the same logic but with restricted visibility).
// =====================================================================

fn html_escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn html_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_groups_have_keys() {
        assert!(NAV_GROUPS.len() == 3);
        assert_eq!(NAV_GROUPS[0].key, "market");
        assert_eq!(NAV_GROUPS[1].key, "developer");
        assert_eq!(NAV_GROUPS[2].key, "company");
    }

    #[test]
    fn nav_groups_have_items() {
        for g in NAV_GROUPS {
            assert!(!g.items.is_empty(), "group {} has no items", g.key);
            for i in g.items {
                assert!(!i.href.is_empty(), "item {} has no href", i.key);
                assert!(!i.label.is_empty(), "item {} has no label", i.key);
            }
        }
    }

    #[test]
    fn is_group_active_matches() {
        let m = &NAV_GROUPS[0];
        assert!(is_group_active(m, "/analytics"));
        assert!(is_group_active(m, "/portfolio"));
        assert!(!is_group_active(m, "/about"));
    }

    #[test]
    fn is_item_active_matches() {
        let item = &NAV_GROUPS[0].items[0];
        assert!(is_item_active(item, "/analytics"));
        assert!(!is_item_active(item, "/about"));
    }

    #[test]
    fn dropdown_renders_for_active_group() {
        let g = &NAV_GROUPS[0];
        let html = render_nav_dropdown(g, "/analytics");
        assert!(html.contains(r#"id="nav-market""#));
        assert!(html.contains("Market"));
        assert!(html.contains("active"));
        assert!(html.contains("/analytics"));
        assert!(html.contains("Rankings"));
        assert!(html.contains("EPS stock rankings"));
    }

    #[test]
    fn accordion_renders_open_for_active_group() {
        let g = &NAV_GROUPS[1];
        let html = render_nav_accordion(g, "/developer");
        assert!(html.contains(r#"id="mob-nav-developer""#));
        assert!(html.contains("Developer"));
        assert!(html.contains("nav-accordion open"));
        assert!(html.contains("aria-expanded=\"true\""));
        assert!(html.contains("/developer"));
        assert!(html.contains("API Keys"));
    }

    #[test]
    fn accordion_renders_closed_for_inactive_group() {
        let g = &NAV_GROUPS[1];
        let html = render_nav_accordion(g, "/");
        assert!(html.contains(r#"id="mob-nav-developer""#));
        assert!(html.contains(r#"class="nav-accordion""#));
        assert!(!html.contains("open"));
        assert!(html.contains("aria-expanded=\"false\""));
    }

    #[test]
    fn footer_links_contains_required() {
        assert!(FOOTER_LINKS.iter().any(|l| l.key == "terms"));
        assert!(FOOTER_LINKS.iter().any(|l| l.key == "privacy"));
        assert!(FOOTER_LINKS.iter().any(|l| l.key == "contact"));
    }

    #[test]
    fn navbar_includes_all_three_groups() {
        let html = render_navbar("/", false, None);
        for g in NAV_GROUPS {
            assert!(html.contains(&format!(r#"id="nav-{k}""#, k = g.key)),
                "missing desktop nav group: {}", g.key);
            assert!(html.contains(&format!(r#"id="mob-nav-{k}""#, k = g.key)),
                "missing mobile nav group: {}", g.key);
        }
    }

    #[test]
    fn escape_does_not_inject() {
        assert_eq!(html_escape_text("a&b<c>d"), "a&amp;b&lt;c&gt;d");
        assert_eq!(html_escape_attr(r#"a"b&c<d>"#), "a&quot;b&amp;c&lt;d&gt;");
    }
}
