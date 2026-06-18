//! Admin `PaymentLinksUI` family — Wave 38b T2 admin domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/payments/payment-links-ui.tsx`,
//! which exports 5 payment-links UI components:
//!
//! | Component | Use case |
//! | --- | --- |
//! | `PaymentLinksFilterSection` | Context-type + status filter row |
//! | `PaymentLinksActionCards` | New Link + Refresh action buttons |
//! | `CreatePaymentLinkModal` | Modal form for creating a new payment link |
//! | `PaymentLinksLoadingState` | Loading skeleton for the page |
//! | `PaymentLinksEmptyState` | Empty state (no links) |
//!
//! The modal form is rendered as a static placeholder on SSR; the
//! hydration logic that wires up the form fields lives in the
//! BFF. The structure (rows + fields + buttons) is preserved 1:1
//! so the rendered HTML matches prod visually.
//!
//! ## Tests
//!
//! Each component gets a colocated `#[cfg(test)] mod tests` block
//! with smoke render + key prop handling.

use dioxus::prelude::*;
use crate::primitives::icon::Icon;

// ============================================================================
// Constants
// ============================================================================
//
// Mirror the source's `CONTEXT_TYPES` + `CURRENCIES` arrays.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaymentContextType {
    Plan,
    Group,
    Product,
    Campaign,
    Custom,
}

impl PaymentContextType {
    pub fn label(&self) -> &'static str {
        match self {
            PaymentContextType::Plan => "Plan",
            PaymentContextType::Group => "Group",
            PaymentContextType::Product => "Product",
            PaymentContextType::Campaign => "Campaign",
            PaymentContextType::Custom => "Custom",
        }
    }
    pub fn description(&self) -> &'static str {
        match self {
            PaymentContextType::Plan => "Plan payment",
            PaymentContextType::Group => "Permission group access",
            PaymentContextType::Product => "One-time product purchase",
            PaymentContextType::Campaign => "Promotional campaign",
            PaymentContextType::Custom => "Custom payment link",
        }
    }
    pub fn value_str(&self) -> &'static str {
        match self {
            PaymentContextType::Plan => "plan",
            PaymentContextType::Group => "group",
            PaymentContextType::Product => "product",
            PaymentContextType::Campaign => "campaign",
            PaymentContextType::Custom => "custom",
        }
    }
}

pub fn all_context_types() -> Vec<PaymentContextType> {
    vec![
        PaymentContextType::Plan,
        PaymentContextType::Group,
        PaymentContextType::Product,
        PaymentContextType::Campaign,
        PaymentContextType::Custom,
    ]
}

pub fn all_currencies() -> Vec<&'static str> {
    vec!["USDT", "USDC", "BNB"]
}

// ============================================================================
// PaymentLinksFilterSection
// ============================================================================
//
// Filter row with Context Type + Status + Reset.

#[component]
pub fn PaymentLinksFilterSection(
    filter_type: Option<String>,
    filter_active: Option<String>,
    on_filter_type_change: EventHandler<String>,
    on_filter_active_change: EventHandler<String>,
    on_reset: EventHandler<()>,
) -> Element {
    let ft = filter_type.unwrap_or_default();
    let fa = filter_active.unwrap_or_default();
    rsx! {
        div { class: "payment-links-filter-section rounded-xl border border-border/20 bg-card p-4 mb-6",
            div { class: "grid grid-cols-1 sm:grid-cols-3 gap-4",
                // Context Type
                div {
                    label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Context Type" }
                    select {
                        class: "w-full px-4 py-2.5 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-1 focus:ring-[#1fc7d4] transition-all text-sm",
                        value: "{ft}",
                        option { value: "", "All Types" }
                        for ctx in all_context_types().iter() {
                            option { value: "{ctx.value_str()}", "{ctx.label()}" }
                        }
                    }
                }
                // Status
                div {
                    label { class: "block text-xs font-bold text-muted-foreground uppercase tracking-[0.15em] mb-2", "Status" }
                    select {
                        class: "w-full px-4 py-2.5 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-1 focus:ring-[#1fc7d4] transition-all text-sm",
                        value: "{fa}",
                        option { value: "", "All Status" }
                        option { value: "true", "Active" }
                        option { value: "false", "Inactive" }
                    }
                }
                // Reset button
                div { class: "flex items-end",
                    button {
                        class: "w-full px-4 py-2.5 font-semibold rounded-xl bg-muted hover:bg-muted/80 text-muted-foreground transition-all border border-border/50 text-sm",
                        r#type: "button",
                        onclick: move |_| on_reset.call(()),
                        "Reset"
                    }
                }
            }
        }
    }
}

// ============================================================================
// PaymentLinksActionCards
// ============================================================================
//
// "New Link" + "Refresh" action buttons.

#[component]
pub fn PaymentLinksActionCards(
    on_create_click: EventHandler<()>,
    on_refresh_click: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "payment-links-action-cards flex items-center gap-3 mb-8",
            button {
                class: "flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] hover:opacity-90 text-white rounded-xl font-semibold text-sm transition-all",
                r#type: "button",
                onclick: move |_| on_create_click.call(()),
                Icon { name: "plus".to_string(), size: Some(16), class_name: Some("w-4 h-4".to_string()) }
                "New Link"
            }
            button {
                class: "flex items-center gap-2 px-4 py-2.5 bg-muted/30 border border-border/40 hover:border-[#7645d9]/40 text-foreground rounded-xl font-semibold text-sm transition-all",
                r#type: "button",
                onclick: move |_| on_refresh_click.call(()),
                Icon { name: "refresh-cw".to_string(), size: Some(16), class_name: Some("w-4 h-4 text-[#7645d9]".to_string()) }
                "Refresh"
            }
        }
    }
}

// ============================================================================
// CreatePaymentLinkModal
// ============================================================================
//
// Modal form for creating a new payment link. Mirrors the source's
// `CreatePaymentLinkModal`. On SSR the modal renders a static
// placeholder; the BFF hydrates the form with real state on click.

#[component]
pub fn CreatePaymentLinkModal(
    is_open: Option<bool>,
    /// Current form values. Strings (the form is text-input driven).
    form_context_type: Option<String>,
    form_context_id: Option<String>,
    form_name: Option<String>,
    form_description: Option<String>,
    form_amount: Option<String>,
    form_currency: Option<String>,
    form_expires_in_hours: Option<String>,
    form_max_uses: Option<String>,
    form_slug: Option<String>,
    is_loading: Option<bool>,
    error: Option<String>,
    on_close: EventHandler<()>,
) -> Element {
    let is_open = is_open.unwrap_or(false);
    let is_loading = is_loading.unwrap_or(false);
    if !is_open {
        return rsx! { Fragment {} };
    }
    let ft = form_context_type.unwrap_or_else(|| "plan".to_string());
    let fid = form_context_id.unwrap_or_default();
    let fname = form_name.unwrap_or_default();
    let fdesc = form_description.unwrap_or_default();
    let famount = form_amount.unwrap_or_default();
    let fcur = form_currency.unwrap_or_else(|| "USDT".to_string());
    let fexp = form_expires_in_hours.unwrap_or_default();
    let fmax = form_max_uses.unwrap_or_default();
    let fslug = form_slug.unwrap_or_default();
    let err = error.unwrap_or_default();
    let show_context_id = ft == "plan" || ft == "group";
    let context_label = if ft == "plan" { "Plan ID" } else { "Group ID" };
    rsx! {
        div { class: "create-payment-link-modal fixed inset-0 z-50 overflow-y-auto",
            div { class: "flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0",
                // Backdrop
                div { class: "fixed inset-0 transition-opacity", aria_hidden: "true", onclick: move |_| on_close.call(()),
                    div { class: "absolute inset-0 bg-background/95" }
                }
                // Modal
                div { class: "inline-block align-bottom bg-card rounded-3xl text-left overflow-hidden shadow-2xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full border border-border/50",
                    form {
                        action: "/api/v1/payment-links",
                        method: "POST",
                        div { class: "px-6 pt-6 pb-4",
                            // Modal header
                            div { class: "flex items-center justify-between mb-6",
                                h3 { class: "text-xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent", "Create Payment Link" }
                                button {
                                    r#type: "button",
                                    onclick: move |_| on_close.call(()),
                                    class: "p-2 rounded-xl text-muted-foreground hover:text-foreground hover:bg-muted transition-colors",
                                    Icon { name: "x".to_string(), size: Some(24), class_name: Some("w-6 h-6".to_string()) }
                                }
                            }
                            // Error message
                            if !err.is_empty() {
                                div { class: "mb-4 bg-destructive/10 border border-destructive/20 text-destructive px-4 py-3 rounded-xl text-sm", "{err}" }
                            }
                            // ModalFormFields
                            div { class: "space-y-4",
                                // Context Type
                                div {
                                    label { class: "block text-sm font-medium text-muted-foreground mb-2", "Context Type *" }
                                    select {
                                        class: "w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all",
                                        name: "context_type",
                                        value: "{ft}",
                                        required: true,
                                        for ctx in all_context_types().iter() {
                                            option { value: "{ctx.value_str()}", "{ctx.label()} \u{2013} {ctx.description()}" }
                                        }
                                    }
                                }
                                // Plan/Group ID (conditional)
                                if show_context_id {
                                    div {
                                        label { class: "block text-sm font-medium text-muted-foreground mb-2", "{context_label}" }
                                        input {
                                            class: "w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all",
                                            r#type: "text",
                                            name: "context_id",
                                            value: "{fid}",
                                            placeholder: "UUID of the linked entity"
                                        }
                                    }
                                }
                                // Name
                                div {
                                    label { class: "block text-sm font-medium text-muted-foreground mb-2", "Name *" }
                                    input {
                                        class: "w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all",
                                        r#type: "text",
                                        name: "name",
                                        value: "{fname}",
                                        placeholder: "e.g., Pro Plan Monthly",
                                        required: true,
                                    }
                                }
                                // Description
                                div {
                                    label { class: "block text-sm font-medium text-muted-foreground mb-2", "Description" }
                                    textarea {
                                        class: "w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all",
                                        name: "description",
                                        rows: "2",
                                        placeholder: "Optional description",
                                        "{fdesc}"
                                    }
                                }
                                // Amount + Currency
                                div { class: "grid grid-cols-2 gap-4",
                                    div {
                                        label { class: "block text-sm font-medium text-muted-foreground mb-2", "Amount *" }
                                        input {
                                            class: "w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all",
                                            r#type: "number",
                                            step: "0.01",
                                            min: "0.01",
                                            name: "amount",
                                            value: "{famount}",
                                            placeholder: "0.00",
                                            required: true,
                                        }
                                    }
                                    div {
                                        label { class: "block text-sm font-medium text-muted-foreground mb-2", "Currency" }
                                        select {
                                            class: "w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all",
                                            name: "currency",
                                            value: "{fcur}",
                                            for cur in all_currencies().iter() {
                                                option { value: "{cur}", "{cur}" }
                                            }
                                        }
                                    }
                                }
                                // Expires In + Max Uses
                                div { class: "grid grid-cols-2 gap-4",
                                    div {
                                        label { class: "block text-sm font-medium text-muted-foreground mb-2", "Expires In (hours)" }
                                        input {
                                            class: "w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all",
                                            r#type: "number",
                                            min: "1",
                                            name: "expires_in_hours",
                                            value: "{fexp}",
                                            placeholder: "24"
                                        }
                                        p { class: "text-xs text-muted-foreground mt-1", "Leave empty for no expiration" }
                                    }
                                    div {
                                        label { class: "block text-sm font-medium text-muted-foreground mb-2", "Max Uses" }
                                        input {
                                            class: "w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all",
                                            r#type: "number",
                                            min: "1",
                                            name: "max_uses",
                                            value: "{fmax}",
                                            placeholder: "Unlimited"
                                        }
                                        p { class: "text-xs text-muted-foreground mt-1", "Leave empty for unlimited" }
                                    }
                                }
                                // Custom Slug
                                div {
                                    label { class: "block text-sm font-medium text-muted-foreground mb-2", "Custom Slug (optional)" }
                                    input {
                                        class: "w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all",
                                        r#type: "text",
                                        name: "slug",
                                        value: "{fslug}",
                                        placeholder: "Auto-generated if empty"
                                    }
                                }
                            }
                        }
                        // Footer
                        div { class: "px-6 py-4 bg-muted/30 flex flex-row-reverse gap-3 rounded-b-3xl",
                            button {
                                r#type: "submit",
                                disabled: is_loading,
                                class: "px-6 py-3 font-semibold rounded-xl bg-primary text-primary-foreground hover:opacity-90 disabled:opacity-50 transition-all border border-border/50",
                                if is_loading { "Creating..." } else { "Create Link" }
                            }
                            button {
                                r#type: "button",
                                onclick: move |_| on_close.call(()),
                                class: "px-6 py-3 font-semibold rounded-xl bg-muted hover:bg-muted/80 text-muted-foreground transition-all border border-border/50",
                                "Cancel"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// PaymentLinksLoadingState
// ============================================================================
//
// Loading skeleton for the payment-links page.

#[component]
pub fn PaymentLinksLoadingState() -> Element {
    rsx! {
        div { class: "payment-links-loading max-w-7xl mx-auto space-y-8 animate-pulse",
            div { class: "text-center mb-12",
                div { class: "h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-6" }
                div { class: "h-6 bg-muted rounded-full w-64 mx-auto" }
            }
            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8",
                for i in 0..4 {
                    div { key: "skel-{i}", class: "bg-card rounded-3xl h-32 border border-border/50",
                        div { class: "sr-only", "skel-{i}" }
                    }
                }
            }
            div { class: "bg-card rounded-3xl h-96 border border-border/50" }
        }
    }
}

// ============================================================================
// PaymentLinksEmptyState
// ============================================================================
//
// Empty state (no payment links yet).

#[component]
pub fn PaymentLinksEmptyState() -> Element {
    rsx! {
        div { class: "payment-links-empty-state text-center py-12 sm:py-16",
            div { class: "h-20 w-20 bg-primary/10 rounded-2xl flex items-center justify-center mx-auto mb-4",
                Icon { name: "link".to_string(), size: Some(40), class_name: Some("w-10 h-10 text-primary".to_string()) }
            }
            h3 { class: "text-xl font-semibold text-foreground mb-2", "No payment links yet" }
            p { class: "text-muted-foreground", "Create your first payment link to get started" }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// `PaymentLinksFilterSection` renders all 3 filter controls.
    #[test]
    fn payment_links_filter_section_renders_3_controls() {
        fn harness() -> Element {
            rsx! {
                PaymentLinksFilterSection {
                    on_filter_type_change: move |_: String| {},
                    on_filter_active_change: move |_: String| {},
                    on_reset: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payment-links-filter-section"), "PaymentLinksFilterSection must render container class. Got: {html}");
        assert!(html.contains("Context Type"), "PaymentLinksFilterSection must render Context Type label. Got: {html}");
        assert!(html.contains("Status"), "PaymentLinksFilterSection must render Status label. Got: {html}");
        assert!(html.contains("Reset"), "PaymentLinksFilterSection must render Reset. Got: {html}");
        for ctx in all_context_types().iter() {
            assert!(html.contains(ctx.label()), "PaymentLinksFilterSection must render context type `{}`. Got: {html}", ctx.label());
        }
    }

    /// `PaymentLinksActionCards` renders New Link + Refresh.
    #[test]
    fn payment_links_action_cards_renders_buttons() {
        fn harness() -> Element {
            rsx! {
                PaymentLinksActionCards {
                    on_create_click: move |_: ()| {},
                    on_refresh_click: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payment-links-action-cards"), "PaymentLinksActionCards must render container class. Got: {html}");
        assert!(html.contains("New Link"), "PaymentLinksActionCards must render New Link button. Got: {html}");
        assert!(html.contains("Refresh"), "PaymentLinksActionCards must render Refresh button. Got: {html}");
    }

    /// `CreatePaymentLinkModal` with `is_open=false` renders nothing.
    #[test]
    fn create_payment_link_modal_hidden_when_closed() {
        fn harness() -> Element {
            rsx! {
                CreatePaymentLinkModal {
                    is_open: Some(false),
                    on_close: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(!html.contains("create-payment-link-modal"), "CreatePaymentLinkModal must hide when closed. Got: {html}");
    }

    /// `CreatePaymentLinkModal` with `is_open=true` renders the full form.
    #[test]
    fn create_payment_link_modal_renders_form_fields() {
        fn harness() -> Element {
            rsx! {
                CreatePaymentLinkModal {
                    is_open: Some(true),
                    on_close: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("create-payment-link-modal"), "CreatePaymentLinkModal must render container class. Got: {html}");
        assert!(html.contains("Create Payment Link"), "CreatePaymentLinkModal must render header. Got: {html}");
        assert!(html.contains("Context Type *"), "CreatePaymentLinkModal must render Context Type label. Got: {html}");
        assert!(html.contains("Name *"), "CreatePaymentLinkModal must render Name label. Got: {html}");
        assert!(html.contains("Description"), "CreatePaymentLinkModal must render Description label. Got: {html}");
        assert!(html.contains("Amount *"), "CreatePaymentLinkModal must render Amount label. Got: {html}");
        assert!(html.contains("Currency"), "CreatePaymentLinkModal must render Currency label. Got: {html}");
        assert!(html.contains("Expires In"), "CreatePaymentLinkModal must render Expires In label. Got: {html}");
        assert!(html.contains("Max Uses"), "CreatePaymentLinkModal must render Max Uses label. Got: {html}");
        assert!(html.contains("Custom Slug"), "CreatePaymentLinkModal must render Custom Slug label. Got: {html}");
        assert!(html.contains("Cancel"), "CreatePaymentLinkModal must render Cancel button. Got: {html}");
        assert!(html.contains("Create Link"), "CreatePaymentLinkModal must render Create Link button. Got: {html}");
    }

    /// `CreatePaymentLinkModal` with `form_context_type="plan"` shows Plan ID.
    #[test]
    fn create_payment_link_modal_shows_plan_id_when_plan() {
        fn harness() -> Element {
            rsx! {
                CreatePaymentLinkModal {
                    is_open: Some(true),
                    form_context_type: Some("plan".to_string()),
                    on_close: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Plan ID"), "CreatePaymentLinkModal with plan context must render Plan ID label. Got: {html}");
    }

    /// `CreatePaymentLinkModal` with `form_context_type="group"` shows Group ID.
    #[test]
    fn create_payment_link_modal_shows_group_id_when_group() {
        fn harness() -> Element {
            rsx! {
                CreatePaymentLinkModal {
                    is_open: Some(true),
                    form_context_type: Some("group".to_string()),
                    on_close: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Group ID"), "CreatePaymentLinkModal with group context must render Group ID label. Got: {html}");
    }

    /// `CreatePaymentLinkModal` with `form_context_type="product"` omits ID field.
    #[test]
    fn create_payment_link_modal_omits_id_field_when_product() {
        fn harness() -> Element {
            rsx! {
                CreatePaymentLinkModal {
                    is_open: Some(true),
                    form_context_type: Some("product".to_string()),
                    on_close: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(!html.contains("Plan ID"), "CreatePaymentLinkModal with product context must omit Plan ID label. Got: {html}");
        assert!(!html.contains("Group ID"), "CreatePaymentLinkModal with product context must omit Group ID label. Got: {html}");
    }

    /// `CreatePaymentLinkModal` with error shows the error banner.
    #[test]
    fn create_payment_link_modal_shows_error_banner() {
        fn harness() -> Element {
            rsx! {
                CreatePaymentLinkModal {
                    is_open: Some(true),
                    error: Some("Amount is required".to_string()),
                    on_close: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Amount is required"), "CreatePaymentLinkModal must render error banner. Got: {html}");
        assert!(html.contains("bg-destructive/10"), "CreatePaymentLinkModal must use destructive bg. Got: {html}");
    }

    /// `PaymentLinksLoadingState` renders the animated skeleton.
    #[test]
    fn payment_links_loading_state_renders_skeleton() {
        fn harness() -> Element {
            rsx! { PaymentLinksLoadingState { } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payment-links-loading"), "PaymentLinksLoadingState must render container class. Got: {html}");
        assert!(html.contains("animate-pulse"), "PaymentLinksLoadingState must animate. Got: {html}");
        for i in 0..4 {
            assert!(html.contains(&format!("skel-{i}")), "PaymentLinksLoadingState must render skel-{i}. Got: {html}");
        }
    }

    /// `PaymentLinksEmptyState` renders the empty message.
    #[test]
    fn payment_links_empty_state_renders_message() {
        fn harness() -> Element {
            rsx! { PaymentLinksEmptyState { } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payment-links-empty-state"), "PaymentLinksEmptyState must render container class. Got: {html}");
        assert!(html.contains("No payment links yet"), "PaymentLinksEmptyState must render headline. Got: {html}");
        assert!(html.contains("Create your first payment link"), "PaymentLinksEmptyState must render subhead. Got: {html}");
    }
}
