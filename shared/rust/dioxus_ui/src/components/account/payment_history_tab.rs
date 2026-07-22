//! Truthful, server-resolved payment history for `/account`.
//!
//! The component deliberately has no client fetch path. The BFF supplies a
//! bounded payload plus an explicit outcome, and this module validates the
//! complete pay-service wire shape before any value reaches RSX.

use chrono::DateTime;
use dioxus::prelude::*;
use serde_json::Value;

use crate::primitives::icon::Icon;

pub const ACCOUNT_PAYMENT_HISTORY_DATA_PARAM: &str = "data_account_payment_history";
pub const ACCOUNT_PAYMENT_HISTORY_STATE_PARAM: &str = "data_account_payment_history_state";
pub const ACCOUNT_PAYMENT_HISTORY_MAX_ITEMS: usize = 10;
pub const ACCOUNT_PAYMENT_HISTORY_READY: &str = "ready";
pub const ACCOUNT_PAYMENT_HISTORY_EMPTY: &str = "empty";
pub const ACCOUNT_PAYMENT_HISTORY_UNAVAILABLE: &str = "unavailable";
pub const ACCOUNT_PAYMENT_HISTORY_MALFORMED: &str = "malformed";

const MAX_ADDRESS_LEN: usize = 128;
const MAX_ID_LEN: usize = 128;
const MAX_CHAIN_ID_LEN: usize = 64;
const MAX_AMOUNT_LEN: usize = 128;
const MAX_STATUS_LEN: usize = 64;
const MAX_DESCRIPTION_LEN: usize = 1_024;
const MAX_TIMESTAMP_LEN: usize = 64;

/// Complete `PayIntent` JSON shape returned by the pay service.
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PayHistoryIntent {
    pub id: String,
    pub chain_id: String,
    pub payer: String,
    pub payee: String,
    pub amount: String,
    pub token_address: String,
    pub status: String,
    pub escrow_id: Option<String>,
    pub tx_hash: Option<String>,
    pub description: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Complete `EscrowRecord` JSON shape returned by the pay service.
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PayHistoryEscrow {
    pub id: String,
    pub chain_id: String,
    pub payer: String,
    pub payee: String,
    pub amount: String,
    pub token_address: String,
    pub fee_amount: String,
    pub status: String,
    pub on_chain_id: Option<String>,
    pub tx_hash: Option<String>,
    pub dispute_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Complete `/api/v1/pay/history/{address}` response shape.
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PayHistory {
    pub address: String,
    pub intents: Vec<PayHistoryIntent>,
    pub escrows: Vec<PayHistoryEscrow>,
    pub total_intents: i64,
    pub total_escrows: i64,
}

/// Explicit account-history outcome. Absence is never interpreted as empty.
#[derive(Clone, Debug, PartialEq)]
pub enum PaymentHistoryLoad {
    SignedOut,
    Ready(PayHistory),
    Empty,
    Unavailable,
    Malformed,
}

/// Strictly decodes one bounded owner-scoped pay-history response.
///
/// The response owner and every row are checked against the authenticated
/// owner. Empty collections may only report a zero total, preventing a
/// truncated or failed upstream response from being presented as "no data".
pub fn decode_pay_history(
    value: Value,
    expected_owner: &str,
    max_items: usize,
) -> Option<PayHistory> {
    if max_items == 0 || !valid_required(expected_owner, MAX_ADDRESS_LEN) {
        return None;
    }

    let history: PayHistory = serde_json::from_value(value).ok()?;
    if !valid_required(&history.address, MAX_ADDRESS_LEN)
        || !history.address.eq_ignore_ascii_case(expected_owner)
    {
        return None;
    }

    let total_intents = usize::try_from(history.total_intents).ok()?;
    let total_escrows = usize::try_from(history.total_escrows).ok()?;
    if history.intents.len() > max_items
        || history.escrows.len() > max_items
        || total_intents < history.intents.len()
        || total_escrows < history.escrows.len()
        || (history.intents.is_empty() && total_intents != 0)
        || (history.escrows.is_empty() && total_escrows != 0)
    {
        return None;
    }

    if !history
        .intents
        .iter()
        .all(|intent| valid_intent(intent, expected_owner))
        || !history
            .escrows
            .iter()
            .all(|escrow| valid_escrow(escrow, expected_owner))
    {
        return None;
    }

    Some(history)
}

fn valid_intent(intent: &PayHistoryIntent, owner: &str) -> bool {
    row_belongs_to_owner(&intent.payer, &intent.payee, owner)
        && valid_required(&intent.id, MAX_ID_LEN)
        && valid_required(&intent.chain_id, MAX_CHAIN_ID_LEN)
        && valid_required(&intent.payer, MAX_ADDRESS_LEN)
        && valid_required(&intent.payee, MAX_ADDRESS_LEN)
        && valid_required(&intent.amount, MAX_AMOUNT_LEN)
        && valid_required(&intent.token_address, MAX_ADDRESS_LEN)
        && valid_required(&intent.status, MAX_STATUS_LEN)
        && valid_optional(&intent.escrow_id, MAX_ID_LEN)
        && valid_optional(&intent.tx_hash, MAX_ID_LEN)
        && valid_optional(&intent.description, MAX_DESCRIPTION_LEN)
        && intent.expires_at.as_deref().is_none_or(valid_timestamp)
        && valid_timestamp(&intent.created_at)
        && valid_timestamp(&intent.updated_at)
}

fn valid_escrow(escrow: &PayHistoryEscrow, owner: &str) -> bool {
    row_belongs_to_owner(&escrow.payer, &escrow.payee, owner)
        && valid_required(&escrow.id, MAX_ID_LEN)
        && valid_required(&escrow.chain_id, MAX_CHAIN_ID_LEN)
        && valid_required(&escrow.payer, MAX_ADDRESS_LEN)
        && valid_required(&escrow.payee, MAX_ADDRESS_LEN)
        && valid_required(&escrow.amount, MAX_AMOUNT_LEN)
        && valid_required(&escrow.token_address, MAX_ADDRESS_LEN)
        && valid_required(&escrow.fee_amount, MAX_AMOUNT_LEN)
        && valid_required(&escrow.status, MAX_STATUS_LEN)
        && valid_optional(&escrow.on_chain_id, MAX_ID_LEN)
        && valid_optional(&escrow.tx_hash, MAX_ID_LEN)
        && valid_optional(&escrow.dispute_reason, MAX_DESCRIPTION_LEN)
        && valid_timestamp(&escrow.created_at)
        && valid_timestamp(&escrow.updated_at)
}

fn row_belongs_to_owner(payer: &str, payee: &str, owner: &str) -> bool {
    payer.eq_ignore_ascii_case(owner) || payee.eq_ignore_ascii_case(owner)
}

fn valid_required(value: &str, max_len: usize) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.len() <= max_len
        && !value.chars().any(char::is_control)
}

fn valid_optional(value: &Option<String>, max_len: usize) -> bool {
    value
        .as_deref()
        .is_none_or(|value| valid_required(value, max_len))
}

fn valid_timestamp(value: &str) -> bool {
    valid_required(value, MAX_TIMESTAMP_LEN) && DateTime::parse_from_rfc3339(value).is_ok()
}

fn direction(payer: &str, owner: &str) -> &'static str {
    if payer.eq_ignore_ascii_case(owner) {
        "Paid"
    } else {
        "Received"
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PaymentHistoryTabProps {
    pub address: Option<String>,
    pub load: PaymentHistoryLoad,
    #[props(default)]
    pub class: Option<String>,
}

#[component]
pub fn PaymentHistoryTab(props: PaymentHistoryTabProps) -> Element {
    let address = props.address.clone().unwrap_or_default();
    let state = match &props.load {
        PaymentHistoryLoad::SignedOut => "signed-out",
        PaymentHistoryLoad::Ready(_) => "ready",
        PaymentHistoryLoad::Empty => "empty",
        PaymentHistoryLoad::Unavailable => "unavailable",
        PaymentHistoryLoad::Malformed => "malformed",
    };

    rsx! {
        section {
            class: props.class.clone().unwrap_or_else(|| "payment-history-tab card card-glass p-6 sm:p-8 lg:p-10 shadow-2xl border-2 border-blue-200/50".to_string()),
            "data-section": "payment-history-tab",
            "data-address": address.clone(),
            "data-state": state,

            div { class: "payment-history-tab-header flex flex-wrap items-center gap-3 mb-8",
                div { class: "p-3 bg-blue-100 dark:bg-blue-900/30 rounded-2xl",
                    Icon { name: "credit-card".to_string(), size: Some(24), class_name: Some("text-blue-600 dark:text-blue-400".to_string()) }
                }
                div {
                    h2 { class: "text-2xl sm:text-3xl font-bold text-foreground", "Transaction History" }
                    if !address.is_empty() {
                        p { class: "payment-history-tab-address font-mono text-xs text-muted-foreground mt-1 break-all",
                            "{address}"
                        }
                    }
                }
                if matches!(props.load, PaymentHistoryLoad::Ready(_)) {
                    a { class: "payment-history-tab-refresh btn btn-outline ml-auto", href: "/account", "Refresh" }
                }
            }

            match props.load {
                PaymentHistoryLoad::Ready(history) => rsx! {
                    div { class: "payment-history-tab-ready", "data-history-state": "ready",
                        dl { class: "payment-history-tab-counts grid grid-cols-2 gap-3 mb-6",
                            div { class: "rounded-xl border border-border p-3",
                                dt { class: "text-xs text-muted-foreground", "Payment intents" }
                                dd { class: "text-xl font-bold", "{history.total_intents}" }
                            }
                            div { class: "rounded-xl border border-border p-3",
                                dt { class: "text-xs text-muted-foreground", "Escrows" }
                                dd { class: "text-xl font-bold", "{history.total_escrows}" }
                            }
                        }

                        if !history.intents.is_empty() {
                            section { class: "payment-history-tab-intents mb-8", aria_label: "Payment intents",
                                h3 { class: "text-sm font-semibold uppercase tracking-wide text-muted-foreground mb-3", "Payment intents" }
                                ol { class: "space-y-3",
                                    for intent in history.intents.iter() {
                                        li {
                                            article { class: "payment-history-tab-intent-row rounded-xl border border-border bg-secondary/50 p-4",
                                                div { class: "flex flex-wrap items-start justify-between gap-3",
                                                    div {
                                                        p { class: "font-semibold", "{direction(&intent.payer, &history.address)}" }
                                                        p { class: "font-mono text-xs text-muted-foreground break-all", "{intent.id}" }
                                                    }
                                                    span { class: "payment-history-tab-status inline-flex rounded-full border border-border px-2 py-0.5 text-xs font-medium", "{intent.status}" }
                                                }
                                                p { class: "mt-3 font-semibold break-all",
                                                    span { "{intent.amount}" }
                                                    span { class: "ml-2 font-mono text-xs text-muted-foreground", "{intent.token_address}" }
                                                }
                                                div { class: "mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground",
                                                    span { "Chain {intent.chain_id}" }
                                                    time { datetime: intent.created_at.clone(), "{intent.created_at}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if !history.escrows.is_empty() {
                            section { class: "payment-history-tab-escrows", aria_label: "Escrows",
                                h3 { class: "text-sm font-semibold uppercase tracking-wide text-muted-foreground mb-3", "Escrows" }
                                ol { class: "space-y-3",
                                    for escrow in history.escrows.iter() {
                                        li {
                                            article { class: "payment-history-tab-escrow-row rounded-xl border border-border bg-secondary/50 p-4",
                                                div { class: "flex flex-wrap items-start justify-between gap-3",
                                                    div {
                                                        p { class: "font-semibold", "{direction(&escrow.payer, &history.address)}" }
                                                        p { class: "font-mono text-xs text-muted-foreground break-all", "{escrow.id}" }
                                                    }
                                                    span { class: "payment-history-tab-status inline-flex rounded-full border border-border px-2 py-0.5 text-xs font-medium", "{escrow.status}" }
                                                }
                                                p { class: "mt-3 font-semibold break-all",
                                                    span { "{escrow.amount}" }
                                                    span { class: "ml-2 font-mono text-xs text-muted-foreground", "{escrow.token_address}" }
                                                }
                                                div { class: "mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground",
                                                    span { "Chain {escrow.chain_id}" }
                                                    time { datetime: escrow.created_at.clone(), "{escrow.created_at}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                PaymentHistoryLoad::SignedOut => rsx! {
                    HistoryMessage {
                        state: "signed-out",
                        title: "Sign in to view payment history",
                        detail: "Payment history is private to the wallet that owns it.",
                        action_label: "Sign in",
                        action_href: "/auth?return_url=%2Faccount",
                    }
                },
                PaymentHistoryLoad::Empty => rsx! {
                    HistoryMessage {
                        state: "empty",
                        title: "No payment history yet",
                        detail: "Payments and escrows owned by this wallet will appear here.",
                        action_label: "Refresh",
                        action_href: "/account",
                    }
                },
                PaymentHistoryLoad::Unavailable => rsx! {
                    HistoryMessage {
                        state: "unavailable",
                        title: "Payment history is temporarily unavailable",
                        detail: "The payment service could not be reached. No empty history was assumed.",
                        action_label: "Retry",
                        action_href: "/account",
                    }
                },
                PaymentHistoryLoad::Malformed => rsx! {
                    HistoryMessage {
                        state: "malformed",
                        title: "Payment history could not be displayed safely",
                        detail: "The payment service returned an unexpected response. No payment data was shown.",
                        action_label: "Retry",
                        action_href: "/account",
                    }
                },
            }
        }
    }
}

#[component]
fn HistoryMessage(
    state: &'static str,
    title: &'static str,
    detail: &'static str,
    action_label: &'static str,
    action_href: &'static str,
) -> Element {
    let role = if matches!(state, "unavailable" | "malformed") {
        "alert"
    } else {
        "status"
    };
    rsx! {
        div {
            class: "payment-history-tab-message p-8 text-center",
            "data-history-state": state,
            role,
            Icon { name: "credit-card".to_string(), size: Some(40), class_name: Some("text-muted-foreground".to_string()) }
            h3 { class: "mt-3 font-semibold text-foreground", "{title}" }
            p { class: "mt-1 text-sm text-muted-foreground", "{detail}" }
            a { class: "btn btn-outline mt-5", href: action_href, "{action_label}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const OWNER: &str = "0x1234aBcD";

    fn payload(status: &str) -> Value {
        json!({
            "address": "0x1234abcd",
            "intents": [{
                "id": "intent-1",
                "chain_id": "1",
                "payer": "0x1234ABCD",
                "payee": "0xmerchant",
                "amount": "12.50",
                "token_address": "0xtoken",
                "status": status,
                "escrow_id": null,
                "tx_hash": null,
                "description": "Quarterly plan",
                "expires_at": "2026-08-22T10:11:12Z",
                "created_at": "2026-07-22T10:11:12Z",
                "updated_at": "2026-07-22T10:12:13Z"
            }],
            "escrows": [{
                "id": "escrow-1",
                "chain_id": "1",
                "payer": "0xmerchant",
                "payee": "0x1234abcd",
                "amount": "8.25",
                "token_address": "0xtoken",
                "fee_amount": "0.25",
                "status": "released",
                "on_chain_id": "42",
                "tx_hash": null,
                "dispute_reason": null,
                "created_at": "2026-07-21T09:10:11Z",
                "updated_at": "2026-07-22T09:10:11Z"
            }],
            "total_intents": 4,
            "total_escrows": 3
        })
    }

    fn render_load(load: PaymentHistoryLoad) -> String {
        dioxus_ssr::render_element(rsx! {
            PaymentHistoryTab {
                address: Some(OWNER.to_string()),
                load,
            }
        })
    }

    #[test]
    fn decoder_accepts_full_owner_scoped_wire_shape() {
        let decoded = decode_pay_history(payload("pending"), OWNER, 10).expect("valid payload");
        assert_eq!(decoded.total_intents, 4);
        assert_eq!(decoded.intents[0].token_address, "0xtoken");
        assert_eq!(decoded.escrows[0].fee_amount, "0.25");
    }

    #[test]
    fn decoder_rejects_wrong_response_owner_and_foreign_rows() {
        let mut wrong_owner = payload("pending");
        wrong_owner["address"] = json!("0xother");
        assert!(decode_pay_history(wrong_owner, OWNER, 10).is_none());

        let mut foreign_intent = payload("pending");
        foreign_intent["intents"][0]["payer"] = json!("0xother");
        foreign_intent["intents"][0]["payee"] = json!("0xmerchant");
        assert!(decode_pay_history(foreign_intent, OWNER, 10).is_none());

        let mut foreign_escrow = payload("pending");
        foreign_escrow["escrows"][0]["payer"] = json!("0xother");
        foreign_escrow["escrows"][0]["payee"] = json!("0xmerchant");
        assert!(decode_pay_history(foreign_escrow, OWNER, 10).is_none());
    }

    #[test]
    fn decoder_rejects_invalid_counts_and_truncated_empty_collections() {
        let mut negative = payload("pending");
        negative["total_intents"] = json!(-1);
        assert!(decode_pay_history(negative, OWNER, 10).is_none());

        let mut fewer_than_rows = payload("pending");
        fewer_than_rows["total_intents"] = json!(0);
        assert!(decode_pay_history(fewer_than_rows, OWNER, 10).is_none());

        let mut hidden_escrows = payload("pending");
        hidden_escrows["escrows"] = json!([]);
        hidden_escrows["total_escrows"] = json!(1);
        assert!(decode_pay_history(hidden_escrows, OWNER, 10).is_none());

        let mut too_many = payload("pending");
        let row = too_many["intents"][0].clone();
        too_many["intents"] = Value::Array(vec![row; 11]);
        too_many["total_intents"] = json!(11);
        assert!(decode_pay_history(too_many, OWNER, 10).is_none());
    }

    #[test]
    fn decoder_only_accepts_true_empty_and_valid_text_and_timestamps() {
        let empty = json!({
            "address": OWNER,
            "intents": [],
            "escrows": [],
            "total_intents": 0,
            "total_escrows": 0
        });
        let decoded = decode_pay_history(empty, OWNER, 10).expect("true empty payload");
        assert!(decoded.intents.is_empty() && decoded.escrows.is_empty());

        let mut control_text = payload("pending");
        control_text["intents"][0]["status"] = json!("pending\nforged");
        assert!(decode_pay_history(control_text, OWNER, 10).is_none());

        let mut bad_timestamp = payload("pending");
        bad_timestamp["intents"][0]["created_at"] = json!("22 July 2026");
        assert!(decode_pay_history(bad_timestamp, OWNER, 10).is_none());
    }

    #[test]
    fn ready_rows_render_direction_counts_token_and_semantic_time_with_escaping() {
        let history = decode_pay_history(payload("<script>alert(1)</script>"), OWNER, 10)
            .expect("safe bounded text");
        let html = render_load(PaymentHistoryLoad::Ready(history));
        assert!(html.contains("Payment intents"));
        assert!(html.contains(">4<"));
        assert!(html.contains("Paid"));
        assert!(html.contains("Received"));
        assert!(html.contains("12.50"));
        assert!(html.contains("0xtoken"));
        assert!(html.contains("<time datetime=\"2026-07-22T10:11:12Z\""));
        assert!(!html.contains("<script>"));
        assert!(
            html.contains("&#60;script&#62;alert(1)&#60;/script&#62;"),
            "status must be HTML-escaped. Got: {html}"
        );
    }

    #[test]
    fn explicit_non_ready_outcomes_are_truthful_and_retry_natively() {
        let signed_out = render_load(PaymentHistoryLoad::SignedOut);
        assert!(signed_out.contains("Sign in to view payment history"));
        assert!(signed_out.contains("return_url=%2Faccount"));
        assert!(signed_out.contains("role=\"status\""));

        let empty = render_load(PaymentHistoryLoad::Empty);
        assert!(empty.contains("No payment history yet"));
        assert!(empty.contains("href=\"/account\""));
        assert!(empty.contains("role=\"status\""));

        let unavailable = render_load(PaymentHistoryLoad::Unavailable);
        assert!(unavailable.contains("temporarily unavailable"));
        assert!(unavailable.contains("No empty history was assumed"));
        assert!(unavailable.contains("role=\"alert\""));

        let malformed = render_load(PaymentHistoryLoad::Malformed);
        assert!(malformed.contains("could not be displayed safely"));
        assert!(malformed.contains("No payment data was shown"));
        assert!(malformed.contains("role=\"alert\""));
    }
}
