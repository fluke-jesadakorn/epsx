//! Shared types for the epsx-pay-svc binary.
//!
//! wave49(slice-3): extracted from main.rs as part of the
//! services/pay modularization. All request/response shapes
//! (PayIntent, EscrowRecord, PayLink, CreatePayIntentRequest,
//! ReleaseEscrowRequest, etc.) live here so handlers in the
//! sibling modules can import without pulling in axum routing.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A pay intent (formerly `PaymentIntent`). One row per
/// `POST /api/v1/pay/intents` call. Status lifecycle:
/// `pending` → `escrowed` (after `confirm`) → `released` (escrow
/// released) or `cancelled` (user-cancelled before confirm).
#[derive(Serialize, Deserialize, FromRow, Clone, Debug, PartialEq, Eq)]
pub struct PayIntent {
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
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Escrow record — created when an intent is confirmed
/// (`POST /api/v1/pay/intents/{id}/confirm`). Tracks the
/// on-chain deposit + dispute lifecycle.
#[derive(Serialize, Deserialize, FromRow, Clone, Debug, PartialEq, Eq)]
pub struct EscrowRecord {
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
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Shareable payment link (wave49 slice-3). Created from
/// an existing intent; `slug` is a short URL-safe id (e.g.
/// `epsx-abc123`) that resolves to the intent. `current_uses`
/// is incremented atomically on `redeem`.
#[derive(Serialize, Deserialize, FromRow, Clone, Debug, PartialEq, Eq)]
pub struct PayLink {
    pub id: String,
    pub slug: String,
    pub intent_id: String,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Request bodies
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreatePayIntentRequest {
    pub payer: String,
    pub payee: String,
    pub amount: String,
    pub token: String,
    pub description: Option<String>,
    pub expires_in: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreatePayLinkRequest {
    pub intent_id: String,
    pub max_uses: Option<i32>,
    pub expires_in: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RedeemPayLinkRequest {
    pub payer: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReleaseEscrowRequest {
    pub escrow_id: String,
    pub signature: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RefundEscrowRequest {
    pub escrow_id: String,
    pub reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DisputeEscrowRequest {
    pub escrow_id: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResolveDisputeRequest {
    pub escrow_id: String,
    pub to_payee: bool,
    pub signature: Option<String>,
}

// ============================================================================
// Response bodies
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PayIntentResponse {
    pub intent: PayIntent,
    pub pay_url: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PayLinkResponse {
    pub link: PayLink,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RedeemPayLinkResponse {
    pub intent: PayIntent,
    pub pay_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PayIntentListResponse {
    pub items: Vec<PayIntent>,
    pub total: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EscrowListResponse {
    pub items: Vec<EscrowRecord>,
    pub total: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PayHistoryResponse {
    pub address: String,
    pub intents: Vec<PayIntent>,
    pub escrows: Vec<EscrowRecord>,
    pub total_intents: i64,
    pub total_escrows: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebhookAck {
    pub received: bool,
    pub intent_id: String,
    pub new_status: String,
}