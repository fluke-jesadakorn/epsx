//! Account components — `PaymentHistoryTab` is the slice-4
//! entry point. Future slices can add `AccountHeader`,
//! `AccessPlanCard`, etc. here.

pub mod payment_history_tab;

pub use payment_history_tab::{PayHistory, PayHistoryEscrow, PayHistoryIntent, PaymentHistoryTab};