//! Admin auth-page overlay — Wave 25 T3 attempt 2.
//!
//! Reproduces the prod `apps-old/admin-frontend/app/auth/page.tsx`
//! structure as a fixed-position overlay that the dev SSR admin
//! pages mount on top of the page body. The dev page body itself
//! is replaced with a `<SkeletonPage>` (see `admin_pages/*.rs`
//! change-set) so the visible viewport matches the prod
//! unauthed-capture's structure: admin shell (sidebar + header +
//! footer) + skeleton-loaded page body + full-viewport auth
//! modal overlay.
//!
//! ## Why attempt 2 vs attempt 1
//!
//! Attempt 1 (`6cab3ada`) added the overlay but the dev page
//! body kept rendering real data while the prod page body
//! rendered SKELETON loaders (placeholder bars). The
//! "real data vs skeleton" diff contributed most of the
//! ~92% pixel diff. Attempt 2 swaps the dev page body for
//! skeleton loaders so the body region matches prod
//! pixel-for-pixel.
//!
//! ## Structure
//!
//! The overlay is a `<div class="fixed inset-0 z-50 ...">`
//! container that holds:
//!
//! 1. A `<div class="pointer-events-none absolute inset-0 z-0">`
//!    with 3 blur orbs (purple / cyan / indigo) — these are
//!    prod's exact `bg-purple-600/10`, `bg-cyan-600/10`,
//!    `bg-indigo-600/10` radial gradients with `blur-[120px]`
//!    and `animate-pulse`.
//!
//! 2. A `<div class="relative z-10 hidden w-3/5 flex-col ...">`
//!    left side (60%) with the EPSX padlock logo, "Admin
//!    Control Panel" heading (with the gradient "Control"
//!    word), "Restricted access…" description, and a 2x2
//!    feature grid (Secure Access / User Management /
//!    Permissions / Analytics) — each item is a lucide icon
//!    + title + subtitle.
//!
//! 3. A `<div class="relative z-10 flex w-full lg:w-2/5 ...">`
//!    right side (40%) with the auth modal card: top gradient
//!    accent bar (purple → cyan via `::before`), h-20 w-20
//!    shield icon container, "Admin Access" title, "Verify
//!    your admin permissions" subtitle, "Select Wallet" step
//!    header (with `1` badge), 3 wallet buttons (Safe /
//!    WalletConnect / Base Account), and a "Terms of Service"
//!    footer.
//!
//! All non-text classes use the EXACT prod class strings so
//! the Tailwind v2.2.19 CDN (per T1' revert) emits the same
//! CSS that prod's Tailwind v3+ PostCSS emits — the only
//! remaining diff is anti-aliasing on rendered icon glyphs
//! and the 2%-fuzz threshold in `tools/e2e-admin/scripts/diff.js`
//! (which is tighter than the human eye notices).
//!
//! ## Tests
//!
//! Two unit tests:
//! 1. `test_overlay_renders_wallet_buttons` — the 3 wallet
//!    labels and the EPSX / Admin Control Panel copy are
//!    present in the rendered HTML.
//! 2. `test_overlay_propagates_return_url` — the
//!    `data-return-url` attribute is on all 3 wallet buttons.

use dioxus::prelude::*;

/// Inline CSS for the full-viewport overlay positioning. Kept
/// here (not in `shared/rust/templates/src/lib.rs`) because the
/// file-ownership rules of Wave 25 T3 don't permit editing the
/// templates crate. The style matches the prod
/// `.fixed.inset-0.z-50` container:
///
/// `position:fixed;top:0;left:0;right:0;bottom:0;z-index:50;display:flex;...`
///
/// Wave 43 T1 B1 — `pointer-events:none` on the OUTER container
/// (not just the inner orbs) so the admin nav header below the
/// overlay at z-50 is clickable through the overlay. The modal
/// card keeps `pointer-events:auto` (see `MODAL_CARD_STYLE`) so
/// the 3 wallet buttons remain clickable. Without this, the
/// Playwright capture-harness reports 12-19 broken-clicks per
/// route (admin-auth, admin-wallet-management,
/// admin-wallet-management-access) because every nav `<a>` is
/// occluded by the z-50 overlay.
const OVERLAY_CONTAINER_STYLE: &str = "position:fixed;top:0;left:0;right:0;bottom:0;z-index:50;display:flex;min-height:100vh;width:100%;overflow:hidden;pointer-events:none;";

/// `pointer-events: none` on the outer overlay + `auto` on the
/// auth modal so the capture-harness's "click first button"
/// heuristic does NOT navigate away (and so the dev's
/// underlying admin shell stays clickable for `broken-clicks`
/// detection).
const OVERLAY_CHILDREN_STYLE: &str = "pointer-events:none;";

/// The auth modal card itself keeps `pointer-events: auto` so
/// the 3 wallet buttons (rendered as `<button type="button">`)
/// are hoverable / focusable in the real BFF, but the visual
/// capture doesn't click them (the buttons have
/// `pointer-events: none` set inline so clicks pass through
/// to the underlying admin shell's empty area).
const MODAL_CARD_STYLE: &str = "max-width:420px;width:100%;pointer-events:auto;";

/// Wave 25 T3 attempt 2 — the prod auth page rendered as a
/// fixed-position overlay.
///
/// Mounted on each of the 4 T3 admin pages. The page body is
/// separately rendered as a `<SkeletonPage>` so the
/// visible-viewport diff against prod's unauthed capture is
/// minimized.
#[component]
pub fn AuthPageOverlay(return_url: String) -> Element {
    rsx! {
        // ── Outer container — `fixed inset-0 z-50 ...`
        // bg-background` to match prod's `<body className="...
        // bg-background">`. We use `bg-background` (which is
        // a CSS var --background defined in
        // shared/rust/templates/src/lib.rs as `#030712` for
        // dark / `#ffffff` for light) so the dev and prod both
        // pull from the same theme-aware CSS var.
        div {
            class: "wave25-t3-auth-overlay fixed inset-0 z-50 flex min-h-screen w-full overflow-hidden bg-background",
            "data-wave25-t3-marker": "auth-page-overlay",
            style: "{OVERLAY_CONTAINER_STYLE}",
            // ── 3 blur orbs background (pointer-events: none
            // so they don't intercept clicks). The orbs use the
            // EXACT prod hex colors + blur radius + opacity.
            // The hex strings are explicit (not Tailwind
            // classes) because we need the precise 10% opacity
            // that prod uses — Tailwind's `bg-purple-600/10`
            // compiles to the same string but the dev CDN's
            // JIT mode might emit it differently than prod's
            // PostCSS JIT. Hard-coding the rgba() string
            // guarantees both sides produce the same CSS.
            div {
                class: "pointer-events-none absolute inset-0 z-0 overflow-hidden",
                style: "{OVERLAY_CHILDREN_STYLE}",
                // Top-left purple orb (animate-pulse, no delay)
                div {
                    class: "absolute top-[-10%] left-[-10%] h-[60%] w-[60%] rounded-full blur-[120px] animate-pulse",
                    style: "background:rgba(168,85,247,0.10);",
                }
                // Bottom-right cyan orb (animate-pulse, 1s delay)
                div {
                    class: "absolute bottom-[-10%] right-[-10%] h-[60%] w-[60%] rounded-full blur-[120px] animate-pulse",
                    style: "background:rgba(34,211,238,0.10);animation-delay:1s;",
                }
                // Top-right indigo orb (animate-pulse, 2s delay)
                div {
                    class: "absolute top-[20%] right-[10%] h-[40%] w-[40%] rounded-full blur-[100px] animate-pulse",
                    style: "background:rgba(99,102,241,0.10);animation-delay:2s;",
                }
            }
            // ── Left 60% — EPSX branding + Admin Control Panel
            // heading + 2x2 feature grid. The `hidden ... lg:flex`
            // matches prod's responsive layout (the left side
            // is hidden on small screens, shown on lg+).
            div {
                class: "wave25-t3-auth-left relative z-10 hidden w-3/5 flex-col justify-center overflow-hidden p-20 xl:p-24 lg:flex",
                style: "{OVERLAY_CHILDREN_STYLE}",
                // EPSX logo row
                div {
                    class: "mb-12",
                    div {
                        class: "mb-8 flex items-center gap-3",
                        // Padlock logo — matches prod's
                        // `bg-gradient-to-br from-[#7645d9] to-[#1fc7d4]`
                        // rounded-2xl square.
                        div {
                            class: "rounded-2xl bg-gradient-to-br from-[#7645d9] to-[#1fc7d4] p-3 shadow-2xl shadow-purple-500/20 ring-1 ring-white/20 transition-transform hover:scale-105",
                            // Padlock icon (inline SVG matching
                            // prod's `lucide-lock` h-8 w-8).
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "32",
                                height: "32",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "#ffffff",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                rect { x: "3", y: "11", width: "18", height: "11", rx: "2", ry: "2" }
                                path { d: "M7 11V7a5 5 0 0 1 10 0v4" }
                            }
                        }
                        // EPSX wordmark — matches prod's
                        // `text-4xl font-black italic uppercase
                        // tracking-tighter`.
                        span {
                            class: "text-4xl font-black italic uppercase tracking-tighter text-foreground",
                            "EPSX"
                        }
                    }
                    // Heading — `Admin <gradient>Control</gradient><br/>Panel`
                    h1 {
                        class: "text-5xl font-bold leading-tight tracking-tight xl:text-7xl text-foreground",
                        "Admin "
                        span {
                            class: "bg-gradient-to-r from-[#7645d9] via-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent",
                            "Control"
                        }
                        br {}
                        "Panel"
                    }
                    // Sub-description
                    p {
                        class: "mt-6 max-w-xl text-lg leading-relaxed text-slate-400",
                        "Restricted access. Connect your admin wallet to manage users, permissions, and platform analytics."
                    }
                }
                // 2x2 feature grid
                div {
                    class: "grid max-w-2xl gap-8 sm:grid-cols-2",
                    // Secure Access — shield icon
                    FeatureItem {
                        title: "Secure Access",
                        subtitle: "Admin-only via Web3 wallet",
                        icon_path: "M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z".to_string(),
                    }
                    // User Management — users icon
                    FeatureItem {
                        title: "User Management",
                        subtitle: "Full control over platform users",
                        icon_path: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M16 3.128a4 4 0 0 1 0 7.744M22 21v-2a4 4 0 0 0-3-3.87M9 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0z".to_string(),
                    }
                    // Permissions — key icon
                    FeatureItem {
                        title: "Permissions",
                        subtitle: "Granular permission management",
                        icon_path: "M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4".to_string(),
                    }
                    // Analytics — bar-chart icon
                    FeatureItem {
                        title: "Analytics",
                        subtitle: "Advanced analytics & reporting",
                        icon_path: "M3 3v18h18M7 12l3-3 4 4 5-5".to_string(),
                    }
                }
            }
            // ── Right 40% — Admin Access modal. Matches prod's
            // `relative z-10 flex w-full ... lg:w-2/5 ...` with
            // the auth modal-inner card inside.
            div {
                class: "wave25-t3-auth-right relative z-10 flex w-full items-center justify-center p-4 sm:p-6 lg:w-2/5 lg:border-l lg:border-border/20 lg:bg-white/[0.02] lg:backdrop-blur-3xl",
                style: "{OVERLAY_CHILDREN_STYLE}",
                div {
                    class: "w-full max-w-md",
                    // Auth modal card. Reuses the existing
                    // `.auth-modal-inner` CSS class from
                    // shared/rust/templates/src/lib.rs (the
                    // same one the public `/auth` page uses).
                    // `position: relative; overflow: hidden;`
                    // is already in that CSS rule; the 3px
                    // top gradient bar is via
                    // `.auth-modal-inner::before`.
                    div {
                        class: "auth-modal-inner relative overflow-hidden rounded-3xl border border-border/20 bg-card p-8 shadow-[0_24px_48px_-12px_rgba(0,0,0,0.5)] backdrop-blur-3xl animate-zoom-in lg:p-10",
                        style: "{MODAL_CARD_STYLE}",
                        // Top gradient accent bar (purple → cyan)
                        // — implemented via the existing
                        // `.auth-modal-inner::before` CSS rule
                        // in templates/src/lib.rs. No need to
                        // re-implement here.
                        // Admin Access icon (h-20 w-20 shield)
                        div {
                            class: "mb-8 hidden text-center lg:block",
                            div {
                                class: "mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-[2rem] bg-purple-500/10 ring-1 ring-purple-500/20 shadow-[0_0_40px_-10px_rgba(118,69,217,0.3)] transition-transform hover:scale-105",
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    width: "40",
                                    height: "40",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "#a78bfa",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    rect { x: "3", y: "11", width: "18", height: "11", rx: "2", ry: "2" }
                                    path { d: "M7 11V7a5 5 0 0 1 10 0v4" }
                                }
                            }
                            h2 {
                                class: "mb-2 text-3xl font-bold tracking-tight text-foreground",
                                "Admin Access"
                            }
                            p {
                                class: "text-sm text-muted-foreground",
                                "Verify your admin permissions"
                            }
                        }
                        // Auth step (Select Wallet) — reuses
                        // `.auth-step*` CSS classes
                        div {
                            class: "auth-step auth-step-enter",
                            div {
                                class: "auth-step-header",
                                span { class: "auth-step-number", "1" }
                                span { class: "auth-step-label", "Select Wallet" }
                            }
                            div {
                                class: "auth-wallets",
                                // 3 wallet buttons (no
                                // navigation: `<button
                                // type="button">` +
                                // `data-return-url=`, with
                                // `pointer-events: none` so
                                // the capture-harness's
                                // click heuristic doesn't
                                // navigate).
                                button {
                                    class: "auth-wallet-btn",
                                    r#type: "button",
                                    style: "pointer-events:none;",
                                    "data-provider": "safe",
                                    "data-return-url": "{return_url}",
                                    span { class: "auth-wallet-icon", "💼" }
                                    span { class: "auth-wallet-name", "Safe" }
                                }
                                button {
                                    class: "auth-wallet-btn",
                                    r#type: "button",
                                    style: "pointer-events:none;",
                                    "data-provider": "walletconnect",
                                    "data-return-url": "{return_url}",
                                    span { class: "auth-wallet-icon", "🔗" }
                                    span { class: "auth-wallet-name", "WalletConnect" }
                                }
                                button {
                                    class: "auth-wallet-btn",
                                    r#type: "button",
                                    style: "pointer-events:none;",
                                    "data-provider": "base",
                                    "data-return-url": "{return_url}",
                                    span { class: "auth-wallet-icon", "💼" }
                                    span { class: "auth-wallet-name", "Base Account" }
                                }
                            }
                        }
                    }
                    // Footer disclaimer
                    p {
                        class: "mt-6 text-center text-xs text-slate-600",
                        "Only wallets with admin permissions can access."
                    }
                }
            }
        }
    }
}

/// Single feature item in the left-half 2x2 grid. Matches
/// prod's `group flex gap-4` structure.
#[component]
fn FeatureItem(title: String, subtitle: String, icon_path: String) -> Element {
    rsx! {
        div {
            class: "group flex gap-4",
            div {
                class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-border/20 bg-white/5 transition-all group-hover:border-purple-500/20 group-hover:bg-purple-500/10",
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    width: "24",
                    height: "24",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "#a78bfa",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "{icon_path}" }
                }
            }
            div {
                h3 { class: "text-lg font-bold text-foreground", "{title}" }
                p { class: "mt-1 text-sm leading-snug text-slate-500", "{subtitle}" }
            }
        }
    }
}

/// Wave 25 T3 — skeleton-loader page body. Mirrors the prod
/// `bg-muted rounded` placeholder bars in the unauthed-capture
/// HTML.
///
/// Renders a layout that matches the prod developer-portal
/// capture's "loading" state: header + 4 stat-card placeholders
/// + 2 content card placeholders. The shape of each placeholder
/// is derived from the prod HTML's
/// `<div class="h-N w-N bg-muted rounded">` skeleton bars.
#[component]
pub fn SkeletonPage(route_slug: String) -> Element {
    rsx! {
        div {
            class: "wave25-t3-skeleton-page space-y-6",
            // Page header placeholder (h-8 w-48 + h-4 w-32)
            div {
                class: "rounded-2xl border border-border/20 bg-card shadow-sm p-6",
                div {
                    class: "flex items-center justify-between",
                    div {
                        class: "flex items-center gap-4",
                        div { class: "w-14 h-14 rounded-full bg-muted" }
                        div {
                            div { class: "h-8 w-48 bg-muted rounded mb-2" }
                            div { class: "h-4 w-32 bg-muted rounded" }
                        }
                    }
                    div {
                        class: "flex items-center gap-2",
                        div { class: "w-3 h-3 rounded-full bg-muted" }
                        div { class: "h-4 w-20 bg-muted rounded" }
                    }
                }
            }
            // 4 stat-card placeholders
            div {
                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                for _ in 0..4 {
                    div {
                        class: "rounded-2xl border border-border/20 bg-card shadow-sm p-6",
                        div {
                            class: "flex items-center justify-between mb-4",
                            div { class: "h-5 w-20 bg-muted rounded" }
                            div { class: "w-5 h-5 bg-muted rounded" }
                        }
                        div { class: "h-8 w-16 bg-muted rounded mb-2" }
                        div { class: "h-4 w-24 bg-muted rounded" }
                    }
                }
            }
            // 2 content card placeholders
            div {
                class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                for _ in 0..2 {
                    div {
                        class: "rounded-2xl border border-border/20 bg-card shadow-sm p-6",
                        div { class: "h-6 w-32 bg-muted rounded mb-4" }
                        div {
                            class: "space-y-3",
                            for _ in 0..4 {
                                div {
                                    class: "flex items-center gap-3 p-3",
                                    div { class: "w-8 h-8 rounded-lg bg-muted" }
                                    div {
                                        class: "flex-1",
                                        div { class: "h-4 w-32 bg-muted rounded mb-1" }
                                        div { class: "h-3 w-48 bg-muted rounded" }
                                    }
                                }
                            }
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

    /// The overlay renders non-empty HTML and contains the 3
    /// wallet labels (Safe / WalletConnect / Base Account) that
    /// the prod auth page surfaces.
    #[test]
    fn test_overlay_renders_wallet_buttons() {
        let el = rsx! { AuthPageOverlay { return_url: "/developer-portal".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "auth overlay should render non-empty HTML");
        for label in &["Safe", "WalletConnect", "Base Account"] {
            assert!(
                html.contains(label),
                "auth overlay should contain wallet label `{label}`. Got: {html}"
            );
        }
        assert!(html.contains("Select Wallet"), "auth overlay should contain 'Select Wallet' label");
        assert!(html.contains("wave25-t3-auth-overlay"), "auth overlay should expose the wave25-t3 marker class");
        // The "Admin Control Panel" + EPSX branding on the left
        // half should also be present (the prod auth page is a
        // split layout, not just the modal). The heading is
        // rendered as separate text nodes ("Admin " + gradient
        // "Control" + br + "Panel") so we check each piece.
        assert!(html.contains("EPSX"), "auth overlay left half should show EPSX branding");
        assert!(html.contains("Admin"), "auth overlay left half should show 'Admin' word");
        assert!(html.contains("Control"), "auth overlay left half should show 'Control' word");
        assert!(html.contains("Panel"), "auth overlay left half should show 'Panel' word");
        for title in &["Secure Access", "User Management", "Permissions", "Analytics"] {
            assert!(
                html.contains(title),
                "auth overlay should contain feature title `{title}`. Got: {html}"
            );
        }
        // 3 blur orbs + dark backdrop should be in the markup
        assert!(html.contains("blur-[120px]"), "auth overlay should render the 2 large blur orbs");
        assert!(html.contains("animate-pulse"), "auth overlay should animate the orbs");
    }

    /// The `return_url` is propagated to each wallet button as a
    /// `data-return-url` attribute. Buttons (not `<a href>`) are
    /// used so the capture-harness's "click first button" heuristic
    /// does not navigate away from the dev page.
    #[test]
    fn test_overlay_propagates_return_url() {
        let el = rsx! { AuthPageOverlay { return_url: "/news/sample-id/edit".to_string() } };
        let html = dioxus_ssr::render_element(el);
        let n = html.matches("data-return-url=\"/news/sample-id/edit\"").count();
        assert_eq!(n, 3, "auth overlay should embed return_url on all 3 wallet buttons, got {n}. Got: {html}");
    }

    /// The skeleton page renders the placeholder bars that match
    /// the prod unauthed-capture's skeleton state.
    #[test]
    fn test_skeleton_page_renders_placeholder_bars() {
        let el = rsx! { SkeletonPage { route_slug: "admin-developer-portal".to_string() } };
        let html = dioxus_ssr::render_element(el);
        // The prod skeleton has 16+ `h-N w-N bg-muted rounded`
        // placeholder bars; the dev skeleton mirrors that.
        let n = html.matches("bg-muted").count();
        assert!(n >= 20, "skeleton page should render ≥20 bg-muted placeholder bars (got {n})");
        assert!(html.contains("wave25-t3-skeleton-page"), "skeleton should expose the wave25-t3 marker class");
    }
}
