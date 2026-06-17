//! NavActions — port of `apps-old/frontend/components/nav/nav-actions.tsx`.
//!
//! The action cluster on the right side of the navbar. Three responsive
//! tiers: full desktop, compact tablet, mobile (just the hamburger).
//!
//! All third-party widgets (theme toggle, chain selector, notification
//! bell, wallet button) are passed in as `Element` slots so this component
//! stays decoupled from those implementations.
//!
//! Wave 2 a11y: the mobile hamburger is rendered by `<MobileNav>` which
//! sets `aria-label="Open menu"`. Action cluster is `role="toolbar"` so
//! screen readers can announce the group.

use dioxus::prelude::*;

use super::mobile_nav::MobileNav;

/// Right-side action cluster. Mirrors the TS `NavActions` component
/// 1:1, but takes widget slots as `Element` props instead of importing
/// the components directly. This keeps Track C's `wallet_button` and the
/// notification-bell + chain-selector + theme-toggle from being compile
/// dependencies of this module.
///
/// - `is_authenticated` — controls whether the notification bell renders
///   (matches the TS `isAuthenticated` prop).
/// - `current_path` — required so `MobileNav` can highlight the active
///   nav group inside the sheet. Default `"/"`.
/// - `is_connected` — passed through to `MobileNav`; controls the
///   "wallet card" inside the mobile sheet. Default `false`.
/// - `wallet_address` — optional formatted address; when present, the
///   mobile sheet shows a wallet card with the address + status dot.
/// - `auth_status` — passed to `MobileNav`; flips the wallet card's
///   status text between "Connected" and "Authenticated".
/// - `theme_toggle` — slot for the `UnifiedThemeToggle` (desktop + tablet).
/// - `chain_selector` — slot for the `ChainSelector` (desktop only).
/// - `notification_bell` — slot for the `NotificationBellClient` (desktop
///   + tablet, when authenticated).
/// - `wallet_button_desktop` — full-size wallet button for desktop.
/// - `wallet_button_tablet` — compact wallet button for tablet.
/// - `on_sign_in` — fired when the user clicks the "Sign In with Wallet"
///   button in the banner. Optional.
#[component]
pub fn NavActions(
    is_authenticated: bool,
    current_path: Option<String>,
    is_connected: Option<bool>,
    wallet_address: Option<String>,
    auth_status: Option<bool>,
    theme_toggle: Option<Element>,
    chain_selector: Option<Element>,
    notification_bell: Option<Element>,
    wallet_button_desktop: Option<Element>,
    wallet_button_tablet: Option<Element>,
    #[props(default)] on_sign_in: Option<EventHandler<MouseEvent>>,
) -> Element {
    let path = current_path.unwrap_or_else(|| "/".to_string());
    let connected = is_connected.unwrap_or(false);
    let authd = auth_status.unwrap_or(false);

    rsx! {
        div {
            class: "flex items-center gap-2",
            role: "toolbar",
            "aria-label": "Page actions",
            // Desktop actions — md+
            div { class: "hidden md:flex items-center gap-1.5",
                if is_authenticated {
                    if let Some(bell) = notification_bell.clone() {
                        {bell}
                    }
                }
                if let Some(toggle) = theme_toggle.clone() {
                    {toggle}
                }
                if let Some(chain) = chain_selector.clone() {
                    {chain}
                }
                if let Some(wallet) = wallet_button_desktop.clone() {
                    {wallet}
                }
            }
            // Tablet actions — sm only (md:hidden to hide on desktop)
            div { class: "hidden sm:flex md:hidden items-center gap-1.5",
                if is_authenticated {
                    if let Some(bell) = notification_bell.clone() {
                        {bell}
                    }
                }
                if let Some(toggle) = theme_toggle.clone() {
                    {toggle}
                }
                if let Some(wallet) = wallet_button_tablet.clone() {
                    {wallet}
                }
            }
            // Mobile hamburger — lg:hidden (matches TS `lg:hidden` on the
            // hamburger button inside <MobileNav>)
            MobileNav {
                is_authenticated,
                current_path: Some(path.clone()),
                is_connected: Some(connected),
                wallet_address: wallet_address.clone(),
                auth_status: Some(authd),
                on_sign_in,
            }
        }
    }
}
