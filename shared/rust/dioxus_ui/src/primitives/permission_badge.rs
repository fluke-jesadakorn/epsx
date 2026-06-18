//! `PermissionBadge` — displays a permission with a human-readable title,
//! optional tooltip, and platform-tinted background.
//!
//! Mirrors `apps-old/frontend/components/ui/permission-badge.tsx`. The
//! Dioxus port is data-only (no API fetch): the parent pre-resolves the
//! `title` / `note` / `platform` / `category` and passes them in. This
//! keeps the component pure (testable) and avoids spinning up an async
//! client under SSR.
//!
//! Platform color tokens map to the design-system's `badge-*-platform`
//! classes — adjust the `epsx_templates::design_system_head` palette to
//! override.

use super::badge::Badge;
use super::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PermissionBadgeSize { Sm, Md, Lg }

impl PermissionBadgeSize {
    pub fn classes(&self) -> &'static str {
        match self {
            PermissionBadgeSize::Sm => "text-xs px-2 py-0.5",
            PermissionBadgeSize::Md => "text-sm px-2.5 py-1",
            PermissionBadgeSize::Lg => "text-base px-3 py-1.5",
        }
    }

    pub fn icon_size_px(&self) -> u32 {
        match self {
            PermissionBadgeSize::Sm => 12,
            PermissionBadgeSize::Md => 14,
            PermissionBadgeSize::Lg => 16,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PermissionBadgeProps {
    pub permission: String,
    pub title: Option<String>,
    pub note: Option<String>,
    pub platform: Option<String>,
    pub category: Option<String>,
    pub size: PermissionBadgeSize,
    pub show_note: bool,
    pub show_platform: bool,
    pub show_code: bool,
}

/// Renders a single permission badge.
///
/// `permission` is the technical code (e.g. `"epsx:analytics:view"`).
/// `title` is the human-readable display label; when `None` the
/// permission code is shown verbatim (a useful fallback during loading).
pub fn PermissionBadge(props: PermissionBadgeProps) -> Element {
    let PermissionBadgeProps {
        permission, title, note, platform, category,
        size, show_note, show_platform, show_code,
    } = props;

    let platform_cls = match platform.as_deref() {
        Some("epsx") => "permission-badge-platform-epsx",
        Some("epsx-pay") => "permission-badge-platform-epsx-pay",
        Some("epsx-token") => "permission-badge-platform-epsx-token",
        Some("admin") => "permission-badge-platform-admin",
        _ => "permission-badge-platform-default",
    };
    let display_title = title.clone().unwrap_or_else(|| permission.clone());
    let icon_name = infer_icon_for_permission(&permission);

    let size_cls = size.classes();
    let icon_px = size.icon_size_px();

    let badge_inner = rsx! {
        Badge {
            kind: super::badge::BadgeKind::Outline,
            class_name: format!(
                "inline-flex items-center gap-1.5 font-medium transition-colors {size_cls} {platform_cls}"
            ),
            Icon { name: icon_name.to_string(), size: Some(icon_px) }
            span { class: "truncate", "{display_title}" }
            if show_platform {
                if let Some(p) = &platform {
                    span { class: "text-[9px] uppercase opacity-60 ml-1", "{p}" }
                }
            }
        }
    };

    let should_show_tooltip = show_note
        && note.as_deref().map(|s| !s.is_empty()).unwrap_or(false);

    if should_show_tooltip {
        let tooltip_note = note.clone().unwrap_or_default();
        let tooltip_title = title.clone().unwrap_or_else(|| permission.clone());
        let tooltip_category = category.clone().unwrap_or_default();
        return rsx! {
            div { class: "permission-badge-tooltip-wrap",
                {badge_inner}
                div { class: "permission-badge-tooltip",
                    p { class: "font-medium", "{tooltip_title}" }
                    p { class: "text-sm text-muted-foreground", "{tooltip_note}" }
                    if show_code {
                        code { class: "text-xs text-muted-foreground/70 font-mono block mt-1",
                            "{permission}"
                        }
                    }
                    if !tooltip_category.is_empty() {
                        span { class: "text-[10px] uppercase text-muted-foreground/60",
                            "{tooltip_category}"
                        }
                    }
                }
            }
        };
    }

    badge_inner
}

/// Best-effort icon-name mapping for a permission string. Returns one of
/// the canonical lucide names already exposed by the Dioxus Icon
/// component (matches the 33-name registry).
fn infer_icon_for_permission(permission: &str) -> &'static str {
    let p = permission.to_lowercase();
    for part in p.split(':') {
        match part.trim() {
            "view" | "read" => return "eye",
            "manage" => return "settings",
            "admin" => return "crown",
            "users" => return "users",
            "analytics" | "rankings" => return "bar-chart-3",
            "realtime" => return "zap",
            _ => {}
        }
    }
    "shield"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_classes_have_distinct_values() {
        assert_ne!(
            PermissionBadgeSize::Sm.classes(),
            PermissionBadgeSize::Md.classes()
        );
        assert_ne!(
            PermissionBadgeSize::Md.classes(),
            PermissionBadgeSize::Lg.classes()
        );
    }

    #[test]
    fn icon_inference_handles_known_actions() {
        // First-match in iteration order: "view" alone = eye, but
        // "epsx:analytics:view" hits "analytics" first → bar-chart-3.
        // The test documents the precedence the impl actually has.
        assert_eq!(infer_icon_for_permission("epsx:analytics:view"), "bar-chart-3");
        assert_eq!(infer_icon_for_permission(":view"), "eye");
        assert_eq!(infer_icon_for_permission(":users:manage"), "users");
        assert_eq!(infer_icon_for_permission(":admin:admin"), "crown");
        assert_eq!(infer_icon_for_permission(":realtime"), "zap");
    }

    #[test]
    fn icon_inference_falls_back_to_shield() {
        assert_eq!(infer_icon_for_permission("foo:bar:unknown"), "shield");
        assert_eq!(infer_icon_for_permission(""), "shield");
    }

    #[test]
    fn props_can_be_constructed_with_minimal_fields() {
        let p = PermissionBadgeProps {
            permission: "epsx:analytics:view".to_string(),
            title: Some("View Analytics".to_string()),
            note: None,
            platform: Some("epsx".to_string()),
            category: None,
            size: PermissionBadgeSize::Sm,
            show_note: false,
            show_platform: false,
            show_code: false,
        };
        assert_eq!(p.permission, "epsx:analytics:view");
    }
}