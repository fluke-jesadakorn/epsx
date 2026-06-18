//! Admin `WalletLabelBadge` component — Wave 38a T1 admin wallet
//! domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/wallet-label-badge.tsx`.
//! Renders a colored pill for a wallet label, with auto-assigned
//! colors based on a deterministic hash of the label text.
//!
//! ## Color palette
//!
//! 8 colors (blue / emerald / amber / rose / purple / cyan /
//! orange / indigo) — same as the prod `LABEL_COLORS` array.
//! The label text's lowercase hash mod 8 picks the color.
//!
//! ## Tests
//!
//! `test_wallet_label_badge_renders_label_text` — the label text
//! appears in the pill.
//! `test_wallet_label_badge_size_sm` / `_md` — both size variants
//! render the right Tailwind classes.
//! `test_wallet_label_badge_deterministic_color` — the same label
//! always picks the same color (no random drift).
//! `test_wallet_label_badge_propagates_class_name` — caller
//! `class_name` is appended to the pill.

use dioxus::prelude::*;

/// Color slot for the label pill. Mirrors the prod `LABEL_COLORS`
/// array (8 colors, indexed 0–7).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelColorSlot {
    /// Background classes (light + dark).
    pub bg: &'static str,
    /// Text classes (light + dark).
    pub text: &'static str,
    /// Border classes (light + dark).
    pub border: &'static str,
}

const LABEL_COLORS: &[LabelColorSlot] = &[
    LabelColorSlot {
        bg: "bg-blue-100 dark:bg-blue-900/40",
        text: "text-blue-700 dark:text-blue-300",
        border: "border-blue-200 dark:border-blue-800",
    },
    LabelColorSlot {
        bg: "bg-emerald-100 dark:bg-emerald-900/40",
        text: "text-emerald-700 dark:text-emerald-300",
        border: "border-emerald-200 dark:border-emerald-800",
    },
    LabelColorSlot {
        bg: "bg-amber-100 dark:bg-amber-900/40",
        text: "text-amber-700 dark:text-amber-300",
        border: "border-amber-200 dark:border-amber-800",
    },
    LabelColorSlot {
        bg: "bg-rose-100 dark:bg-rose-900/40",
        text: "text-rose-700 dark:text-rose-300",
        border: "border-rose-200 dark:border-rose-800",
    },
    LabelColorSlot {
        bg: "bg-purple-100 dark:bg-purple-900/40",
        text: "text-purple-700 dark:text-purple-300",
        border: "border-purple-200 dark:border-purple-800",
    },
    LabelColorSlot {
        bg: "bg-cyan-100 dark:bg-cyan-900/40",
        text: "text-cyan-700 dark:text-cyan-300",
        border: "border-cyan-200 dark:border-cyan-800",
    },
    LabelColorSlot {
        bg: "bg-orange-100 dark:bg-orange-900/40",
        text: "text-orange-700 dark:text-orange-300",
        border: "border-orange-200 dark:border-orange-800",
    },
    LabelColorSlot {
        bg: "bg-indigo-100 dark:bg-indigo-900/40",
        text: "text-indigo-700 dark:text-indigo-300",
        border: "border-indigo-200 dark:border-indigo-800",
    },
];

/// Deterministic 32-bit FNV-style hash (matches the prod
/// `hashString` function: `hash = ((hash << 5) - hash) + char`,
/// converted to a 32-bit integer). We use wrapping arithmetic so
/// overflow doesn't panic — same as JS's lossy `& hash`.
fn hash_string(s: &str) -> u32 {
    let mut hash: u32 = 0;
    for b in s.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u32);
    }
    hash
}

/// Resolve the color slot for a label. Pure function — same input
/// always returns the same color (no random drift).
pub fn get_label_color(label: &str) -> LabelColorSlot {
    let index = (hash_string(&label.to_lowercase()) as usize) % LABEL_COLORS.len();
    LABEL_COLORS[index]
}

/// Size variant for the label pill.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WalletLabelSize {
    Sm,
    Md,
}

impl WalletLabelSize {
    pub fn from_str(s: &str) -> Self {
        match s {
            "md" => WalletLabelSize::Md,
            _ => WalletLabelSize::Sm,
        }
    }
    pub fn classes(&self) -> &'static str {
        match self {
            WalletLabelSize::Sm => "px-2 py-0.5 text-xs",
            WalletLabelSize::Md => "px-3 py-1 text-sm",
        }
    }
}

/// Colored label pill with deterministic color assignment.
///
/// Mirrors the prod `WalletLabelBadge` from
/// `apps-old/admin-frontend/components/wallet/wallet-label-badge.tsx`
/// lines 78–115. The label text is hashed to pick one of 8 colors,
/// and a small remove (×) button is rendered when `on_remove` is
/// supplied.
#[component]
pub fn WalletLabelBadge(
    label: String,
    /// Size variant. Defaults to `"sm"`.
    size: Option<String>,
    /// Optional extra classes appended to the pill.
    class_name: Option<String>,
    /// Optional remove callback. When set, renders an `×` button
    /// that calls the callback.
    on_remove: Option<EventHandler<MouseEvent>>,
) -> Element {
    let size_kind = WalletLabelSize::from_str(size.as_deref().unwrap_or("sm"));
    let colors = get_label_color(&label);
    let mut cls = format!(
        "inline-flex items-center gap-1 font-medium border rounded-full {} {} {} {}",
        colors.bg,
        colors.text,
        colors.border,
        size_kind.classes(),
    );
    if let Some(extra) = class_name {
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
    }
    rsx! {
        span { class: "{cls}",
            span { class: "truncate max-w-[120px]", "{label}" }
            if let Some(cb) = on_remove {
                button {
                    r#type: "button",
                    class: "ml-0.5 hover:bg-black/10 dark:hover:bg-muted/50 rounded-full p-0.5 transition-colors",
                    aria_label: "Remove label {label}",
                    onclick: move |e| {
                        e.stop_propagation();
                        cb.call(e);
                    },
                    svg {
                        class: "w-3 h-3",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M6 18L18 6M6 6l12 12",
                        }
                    }
                }
            }
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// The pill renders the label text and a color class.
    #[test]
    fn test_wallet_label_badge_renders_label_text() {
        let el = rsx! {
            WalletLabelBadge {
                label: "VIP Customer".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("VIP Customer"),
            "LabelBadge should render the label text. Got: {html}"
        );
        // Should have a colored bg (any of the 8 palette colors).
        assert!(
            html.contains("bg-")
                && (html.contains("100") || html.contains("900")),
            "LabelBadge should render a palette background. Got: {html}"
        );
    }

    /// `size = "sm"` renders the small pill classes.
    #[test]
    fn test_wallet_label_badge_size_sm() {
        let el = rsx! {
            WalletLabelBadge {
                label: "sm".to_string(),
                size: Some("sm".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("text-xs"),
            "sm size should use text-xs. Got: {html}"
        );
    }

    /// `size = "md"` renders the medium pill classes.
    #[test]
    fn test_wallet_label_badge_size_md() {
        let el = rsx! {
            WalletLabelBadge {
                label: "md".to_string(),
                size: Some("md".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("text-sm"),
            "md size should use text-sm. Got: {html}"
        );
    }

    /// Same label always picks the same color (no random drift
    /// across renders).
    #[test]
    fn test_wallet_label_badge_deterministic_color() {
        let c1 = get_label_color("VIP Customer");
        let c2 = get_label_color("VIP Customer");
        let c3 = get_label_color("VIP CUSTOMER"); // case-insensitive
        assert_eq!(c1.bg, c2.bg, "Same label must yield the same bg color");
        assert_eq!(c1.bg, c3.bg, "Color must be case-insensitive");
    }

    /// Caller-supplied `class_name` is appended to the pill.
    #[test]
    fn test_wallet_label_badge_propagates_class_name() {
        let el = rsx! {
            WalletLabelBadge {
                label: "X".to_string(),
                class_name: Some("ml-2 hidden sm:inline-flex".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("ml-2"),
            "class_name should propagate. Got: {html}"
        );
        assert!(
            html.contains("hidden"),
            "class_name should propagate. Got: {html}"
        );
    }
}
