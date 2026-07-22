//! Account components — `PaymentHistoryTab` is the slice-4
//! entry point. Future slices can add `AccountHeader`,
//! `AccessPlanCard`, etc. here.

pub mod payment_history_tab;

pub use payment_history_tab::{
    decode_pay_history, PayHistory, PayHistoryEscrow, PayHistoryIntent, PaymentHistoryLoad,
    PaymentHistoryTab, ACCOUNT_PAYMENT_HISTORY_DATA_PARAM, ACCOUNT_PAYMENT_HISTORY_EMPTY,
    ACCOUNT_PAYMENT_HISTORY_MALFORMED, ACCOUNT_PAYMENT_HISTORY_MAX_ITEMS,
    ACCOUNT_PAYMENT_HISTORY_READY, ACCOUNT_PAYMENT_HISTORY_STATE_PARAM,
    ACCOUNT_PAYMENT_HISTORY_UNAVAILABLE,
};
