//! Admin `PolicyBuilder` — Wave 38b T2 admin domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/policies/policy-builder.tsx`,
//! which renders the policy builder page (template picker +
//! configuration + target actions + conditions + actions/responses
//! + test results). The full implementation has 8+ sub-components
//! and lives behind a hook; we port the 5 most-reused building
//! blocks as drop-in Dioxus components.
//!
//! | Component | Use case |
//! | --- | --- |
//! | `PolicyBuilderHeader` | Header row (templates toggle + test + save buttons) |
//! | `PolicyBuilderConfigurationCard` | Configuration card (name + description + priority + scope) |
//! | `PolicyBuilderTargetActionsCard` | Target actions card (list + add/remove) |
//! | `PolicyBuilderConditionsCard` | Conditions card (list of conditions) |
//! | `PolicyBuilderActionsResponsesCard` | Actions/responses card |
//!
//! ## Tests
//!
//! Each component gets a colocated `#[cfg(test)] mod tests` block
//! with smoke render + key prop handling.

use dioxus::prelude::*;

// ============================================================================
// Data shapes
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct PolicyConfig {
    pub name: String,
    pub description: String,
    pub priority: u32,
    pub scope: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolicyTargetAction {
    pub id: String,
    pub action_type: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolicyCondition {
    pub id: String,
    pub field: String,
    pub operator: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolicyActionResponse {
    pub id: String,
    pub action: String,
    pub response: String,
}

// ============================================================================
// PolicyBuilderHeader
// ============================================================================
//
// Header row with templates toggle + Test + Save buttons.

#[component]
pub fn PolicyBuilderHeader(
    show_templates: Option<bool>,
    on_toggle_templates: EventHandler<bool>,
    on_test: EventHandler<()>,
    on_save: EventHandler<()>,
    saving: Option<bool>,
) -> Element {
    let show_templates = show_templates.unwrap_or(false);
    let saving = saving.unwrap_or(false);
    rsx! {
        div { class: "policy-builder-header flex items-center justify-between mb-6",
            div { class: "flex items-center gap-3",
                button {
                    class: "flex items-center gap-2 px-4 py-2 bg-card border border-border/40 hover:border-[#1fc7d4]/40 text-foreground rounded-xl font-semibold text-sm transition-all",
                    r#type: "button",
                    onclick: move |_| on_toggle_templates.call(!show_templates),
                    if show_templates { "Hide Templates" } else { "Show Templates" }
                }
                button {
                    class: "flex items-center gap-2 px-4 py-2 bg-muted/30 border border-border/40 hover:border-[#7645d9]/40 text-foreground rounded-xl font-semibold text-sm transition-all",
                    r#type: "button",
                    onclick: move |_| on_test.call(()),
                    "Test"
                }
            }
            button {
                class: "flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] hover:opacity-90 text-white rounded-xl font-semibold text-sm transition-all",
                r#type: "button",
                disabled: saving,
                onclick: move |_| on_save.call(()),
                if saving { "Saving..." } else { "Save Policy" }
            }
        }
    }
}

// ============================================================================
// PolicyBuilderConfigurationCard
// ============================================================================
//
// Configuration card (name + description + priority + scope).

#[component]
pub fn PolicyBuilderConfigurationCard(
    config: PolicyConfig,
    on_change: EventHandler<PolicyConfig>,
) -> Element {
    rsx! {
        div { class: "policy-builder-configuration-card bg-card rounded-2xl border border-border/20 p-6",
            h2 { class: "text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em] mb-4", "Policy Configuration" }
            div { class: "space-y-4",
                div {
                    label { class: "block text-sm font-medium text-muted-foreground mb-2", "Name" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{config.name}",
                        placeholder: "e.g., High-value transaction block"
                    }
                }
                div {
                    label { class: "block text-sm font-medium text-muted-foreground mb-2", "Description" }
                    textarea {
                        class: "input",
                        rows: "2",
                        placeholder: "What does this policy do?",
                        "{config.description}"
                    }
                }
                div { class: "grid grid-cols-2 gap-4",
                    div {
                        label { class: "block text-sm font-medium text-muted-foreground mb-2", "Priority" }
                        select {
                            class: "input",
                            value: "{config.priority}",
                            option { value: "1", "Critical" }
                            option { value: "2", "High" }
                            option { value: "3", "Normal" }
                            option { value: "4", "Low" }
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-muted-foreground mb-2", "Scope" }
                        select {
                            class: "input",
                            value: "{config.scope}",
                            option { value: "global", "Global" }
                            option { value: "plan", "Plan" }
                            option { value: "user", "User" }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// PolicyBuilderTargetActionsCard
// ============================================================================
//
// Target actions card (list + add/remove).

#[component]
pub fn PolicyBuilderTargetActionsCard(
    actions: Vec<PolicyTargetAction>,
    on_add: EventHandler<()>,
    on_remove: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "policy-builder-target-actions-card bg-card rounded-2xl border border-border/20 p-6",
            div { class: "flex items-center justify-between mb-4",
                h2 { class: "text-xs font-bold text-[#7645d9] uppercase tracking-[0.2em]", "Target Actions" }
                button {
                    class: "px-3 py-1.5 text-xs font-semibold rounded-lg bg-[#7645d9]/10 text-[#7645d9] hover:bg-[#7645d9]/20 transition-colors",
                    r#type: "button",
                    onclick: move |_| on_add.call(()),
                    "+ Add Action"
                }
            }
            if actions.is_empty() {
                p { class: "text-sm text-muted-foreground italic", "No target actions configured." }
            } else {
                div { class: "space-y-2",
                    for action in actions.iter() {
                        div { key: "{action.id}", class: "flex items-center justify-between p-3 rounded-xl bg-muted/30 border border-border/20",
                            div { class: "flex-1 min-w-0",
                                div { class: "text-sm font-medium text-foreground", "{action.action_type}" }
                                div { class: "text-xs text-muted-foreground font-mono truncate", "{action.target}" }
                            }
                            button {
                                class: "p-1 rounded-lg hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition-colors",
                                r#type: "button",
                                onclick: {
                                    let id = action.id.clone();
                                    move |_| on_remove.call(id.clone())
                                },
                                "Remove"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// PolicyBuilderConditionsCard
// ============================================================================
//
// Conditions card (list of conditions).

#[component]
pub fn PolicyBuilderConditionsCard(
    conditions: Vec<PolicyCondition>,
    on_add: EventHandler<()>,
    on_remove: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "policy-builder-conditions-card bg-card rounded-2xl border border-border/20 p-6",
            div { class: "flex items-center justify-between mb-4",
                h2 { class: "text-xs font-bold text-[#31d0aa] uppercase tracking-[0.2em]", "Conditions" }
                button {
                    class: "px-3 py-1.5 text-xs font-semibold rounded-lg bg-[#31d0aa]/10 text-[#31d0aa] hover:bg-[#31d0aa]/20 transition-colors",
                    r#type: "button",
                    onclick: move |_| on_add.call(()),
                    "+ Add Condition"
                }
            }
            if conditions.is_empty() {
                p { class: "text-sm text-muted-foreground italic", "No conditions configured (policy always applies)." }
            } else {
                div { class: "space-y-2",
                    for c in conditions.iter() {
                        div { key: "{c.id}", class: "flex items-center gap-2 p-3 rounded-xl bg-muted/30 border border-border/20",
                            div { class: "flex-1 grid grid-cols-3 gap-2 text-sm",
                                span { class: "font-mono text-xs text-foreground", "{c.field}" }
                                span { class: "text-xs text-muted-foreground", "{c.operator}" }
                                span { class: "font-mono text-xs text-foreground truncate", "{c.value}" }
                            }
                            button {
                                class: "p-1 rounded-lg hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition-colors text-xs",
                                r#type: "button",
                                onclick: {
                                    let id = c.id.clone();
                                    move |_| on_remove.call(id.clone())
                                },
                                "Remove"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// PolicyBuilderActionsResponsesCard
// ============================================================================
//
// Actions/responses card.

#[component]
pub fn PolicyBuilderActionsResponsesCard(
    items: Vec<PolicyActionResponse>,
    on_add: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "policy-builder-actions-responses-card bg-card rounded-2xl border border-border/20 p-6",
            div { class: "flex items-center justify-between mb-4",
                h2 { class: "text-xs font-bold text-[#ed4b9e] uppercase tracking-[0.2em]", "Actions & Responses" }
                button {
                    class: "px-3 py-1.5 text-xs font-semibold rounded-lg bg-[#ed4b9e]/10 text-[#ed4b9e] hover:bg-[#ed4b9e]/20 transition-colors",
                    r#type: "button",
                    onclick: move |_| on_add.call(()),
                    "+ Add Action"
                }
            }
            if items.is_empty() {
                p { class: "text-sm text-muted-foreground italic", "No actions configured." }
            } else {
                div { class: "space-y-2",
                    for item in items.iter() {
                        div { key: "{item.id}", class: "p-3 rounded-xl bg-muted/30 border border-border/20",
                            div { class: "text-sm font-medium text-foreground", "{item.action}" }
                            div { class: "text-xs text-muted-foreground mt-1", "{item.response}" }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> PolicyConfig {
        PolicyConfig {
            name: "Block high-value txn".to_string(),
            description: "Block transactions over 10k USDT".to_string(),
            priority: 1,
            scope: "global".to_string(),
        }
    }

    /// `PolicyBuilderHeader` renders templates toggle + test + save buttons.
    #[test]
    fn policy_builder_header_renders_buttons() {
        fn harness() -> Element {
            rsx! {
                PolicyBuilderHeader {
                    show_templates: Some(false),
                    on_toggle_templates: move |_: bool| {},
                    on_test: move |_: ()| {},
                    on_save: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("policy-builder-header"), "PolicyBuilderHeader must render container class. Got: {html}");
        assert!(html.contains("Show Templates"), "PolicyBuilderHeader must render Show Templates. Got: {html}");
        assert!(html.contains("Test"), "PolicyBuilderHeader must render Test button. Got: {html}");
        assert!(html.contains("Save Policy"), "PolicyBuilderHeader must render Save Policy button. Got: {html}");
    }

    /// `PolicyBuilderHeader` shows "Hide Templates" when toggled.
    #[test]
    fn policy_builder_header_shows_hide_when_active() {
        fn harness() -> Element {
            rsx! {
                PolicyBuilderHeader {
                    show_templates: Some(true),
                    on_toggle_templates: move |_: bool| {},
                    on_test: move |_: ()| {},
                    on_save: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Hide Templates"), "PolicyBuilderHeader with show_templates=true must render Hide Templates. Got: {html}");
    }

    /// `PolicyBuilderHeader` shows "Saving..." when saving=true.
    #[test]
    fn policy_builder_header_shows_saving_state() {
        fn harness() -> Element {
            rsx! {
                PolicyBuilderHeader {
                    saving: Some(true),
                    on_toggle_templates: move |_: bool| {},
                    on_test: move |_: ()| {},
                    on_save: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Saving..."), "PolicyBuilderHeader saving must show Saving... text. Got: {html}");
    }

    /// `PolicyBuilderConfigurationCard` renders all 4 fields.
    #[test]
    fn policy_builder_configuration_card_renders_all() {
        fn harness() -> Element {
            rsx! {
                PolicyBuilderConfigurationCard {
                    config: sample_config(),
                    on_change: move |_: PolicyConfig| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("policy-builder-configuration-card"), "PolicyBuilderConfigurationCard must render container class. Got: {html}");
        assert!(html.contains("Block high-value txn"), "PolicyBuilderConfigurationCard must render name. Got: {html}");
        assert!(html.contains("Block transactions over 10k USDT"), "PolicyBuilderConfigurationCard must render description. Got: {html}");
        assert!(html.contains("Priority"), "PolicyBuilderConfigurationCard must render Priority label. Got: {html}");
        assert!(html.contains("Scope"), "PolicyBuilderConfigurationCard must render Scope label. Got: {html}");
    }

    /// `PolicyBuilderTargetActionsCard` with empty actions shows empty state.
    #[test]
    fn policy_builder_target_actions_card_empty_renders() {
        fn harness() -> Element {
            rsx! {
                PolicyBuilderTargetActionsCard {
                    actions: vec![],
                    on_add: move |_: ()| {},
                    on_remove: move |_: String| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("policy-builder-target-actions-card"), "PolicyBuilderTargetActionsCard must render container class. Got: {html}");
        assert!(html.contains("No target actions configured"), "PolicyBuilderTargetActionsCard empty must show empty state. Got: {html}");
    }

    /// `PolicyBuilderTargetActionsCard` with actions shows them.
    #[test]
    fn policy_builder_target_actions_card_renders_actions() {
        fn harness() -> Element {
            rsx! {
                PolicyBuilderTargetActionsCard {
                    actions: vec![PolicyTargetAction {
                        id: "a1".to_string(),
                        action_type: "block_transaction".to_string(),
                        target: "wallet:0xABC".to_string(),
                    }],
                    on_add: move |_: ()| {},
                    on_remove: move |_: String| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("block_transaction"), "PolicyBuilderTargetActionsCard must render action type. Got: {html}");
        assert!(html.contains("wallet:0xABC"), "PolicyBuilderTargetActionsCard must render target. Got: {html}");
    }

    /// `PolicyBuilderConditionsCard` empty shows "No conditions" message.
    #[test]
    fn policy_builder_conditions_card_empty_renders() {
        fn harness() -> Element {
            rsx! {
                PolicyBuilderConditionsCard {
                    conditions: vec![],
                    on_add: move |_: ()| {},
                    on_remove: move |_: String| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("policy-builder-conditions-card"), "PolicyBuilderConditionsCard must render container class. Got: {html}");
        assert!(html.contains("No conditions configured"), "PolicyBuilderConditionsCard empty must show empty state. Got: {html}");
    }

    /// `PolicyBuilderActionsResponsesCard` empty shows empty state.
    #[test]
    fn policy_builder_actions_responses_card_empty_renders() {
        fn harness() -> Element {
            rsx! {
                PolicyBuilderActionsResponsesCard {
                    items: vec![],
                    on_add: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("policy-builder-actions-responses-card"), "PolicyBuilderActionsResponsesCard must render container class. Got: {html}");
        assert!(html.contains("No actions configured"), "PolicyBuilderActionsResponsesCard empty must show empty state. Got: {html}");
    }
}
