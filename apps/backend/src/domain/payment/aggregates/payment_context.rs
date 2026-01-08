//! Payment Context Aggregate
//!
//! Represents a dynamic payment link that can be associated with various contexts
//! (plans, groups, products, campaigns, or custom purposes).
//!
//! Core Design Philosophy:
//! - Users can unlock granular features by paying through dynamic links
//! - Payments are tied to context (group/plan/link) enabling flexible feature unlocking
//! - Default expiration: 24 hours, multi-use by default

use crate::prelude::*;
use crate::domain::shared_kernel::{AggregateRoot, AggregateBase, DomainEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Context types for dynamic payments
/// Maps to smart contract ContextType enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentContextType {
    Plan = 0,
    Group = 1,
    Product = 2,
    Campaign = 3,
    Custom = 4,
}

impl PaymentContextType {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Plan),
            1 => Some(Self::Group),
            2 => Some(Self::Product),
            3 => Some(Self::Campaign),
            4 => Some(Self::Custom),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Group => "group",
            Self::Product => "product",
            Self::Campaign => "campaign",
            Self::Custom => "custom",
        }
    }
}

impl std::fmt::Display for PaymentContextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for PaymentContextType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plan" => Ok(Self::Plan),
            "group" => Ok(Self::Group),
            "product" => Ok(Self::Product),
            "campaign" => Ok(Self::Campaign),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("Unknown context type: {}", s)),
        }
    }
}

/// Payment Context ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaymentContextId(Uuid);

impl PaymentContextId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for PaymentContextId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PaymentContextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Payment Context Aggregate Root
/// Represents a dynamic payment link that can be used to unlock features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentContext {
    id: PaymentContextId,
    context_type: PaymentContextType,
    context_id: Option<Uuid>,       // UUID of plan, group, etc.
    slug: String,                    // URL-friendly identifier
    name: String,                    // Display name
    description: Option<String>,
    amount: rust_decimal::Decimal,
    currency: String,
    expires_at: Option<DateTime<Utc>>,  // Default: 24 hours from creation
    max_uses: Option<i32>,              // Default: None (multi-use)
    current_uses: i32,
    metadata: serde_json::Value,
    is_active: bool,
    created_by: String,                 // Wallet address of creator
    base: AggregateBase,
}

/// Default expiration duration: 24 hours
pub const DEFAULT_EXPIRATION_HOURS: i64 = 24;

/// Parameters for creating a new payment context
pub struct CreatePaymentContextParams {
    pub context_type: PaymentContextType,
    pub context_id: Option<Uuid>,
    pub slug: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub created_by: String,
}

/// Parameters for loading an existing payment context
pub struct LoadPaymentContextParams {
    pub id: PaymentContextId,
    pub context_type: PaymentContextType,
    pub context_id: Option<Uuid>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<i32>,
    pub current_uses: i32,
    pub metadata: serde_json::Value,
    pub is_active: bool,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
}

/// Parameters for updating a payment context
#[derive(Default)]
pub struct UpdatePaymentContextParams {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub amount: Option<rust_decimal::Decimal>,
    pub currency: Option<String>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
    pub max_uses: Option<Option<i32>>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

impl PaymentContext {
    /// Create a new payment context with default 24-hour expiration
    pub fn create(id: PaymentContextId, params: CreatePaymentContextParams) -> AppResult<Self> {
        // Generate slug if not provided
        let slug = params.slug.unwrap_or_else(|| {
            format!("{}-{}", params.context_type.as_str(), &Uuid::new_v4().to_string()[..8])
        });

        // Default expiration: 24 hours from now
        let expires_at = params.expires_at.or_else(|| {
            Some(Utc::now() + chrono::Duration::hours(DEFAULT_EXPIRATION_HOURS))
        });

        Ok(Self {
            id,
            context_type: params.context_type,
            context_id: params.context_id,
            slug,
            name: params.name,
            description: params.description,
            amount: params.amount,
            currency: params.currency,
            expires_at,
            max_uses: params.max_uses,  // None = multi-use (unlimited)
            current_uses: 0,
            metadata: params.metadata.unwrap_or_else(|| serde_json::json!({})),
            is_active: true,
            created_by: params.created_by,
            base: AggregateBase::new(),
        })
    }

    /// Load existing payment context from database
    pub fn load(params: LoadPaymentContextParams) -> Self {
        Self {
            id: params.id,
            context_type: params.context_type,
            context_id: params.context_id,
            slug: params.slug,
            name: params.name,
            description: params.description,
            amount: params.amount,
            currency: params.currency,
            expires_at: params.expires_at,
            max_uses: params.max_uses,
            current_uses: params.current_uses,
            metadata: params.metadata,
            is_active: params.is_active,
            created_by: params.created_by,
            base: AggregateBase {
                version: params.version,
                created_at: params.created_at,
                updated_at: params.updated_at,
                events: Vec::new(),
            },
        }
    }

    /// Update payment context
    pub fn update(&mut self, params: UpdatePaymentContextParams) -> AppResult<()> {
        if let Some(name) = params.name {
            self.name = name;
        }
        if let Some(desc) = params.description {
            self.description = desc;
        }
        if let Some(amount) = params.amount {
            self.amount = amount;
        }
        if let Some(currency) = params.currency {
            self.currency = currency;
        }
        if let Some(expires_at) = params.expires_at {
            self.expires_at = expires_at;
        }
        if let Some(max_uses) = params.max_uses {
            self.max_uses = max_uses;
        }
        if let Some(is_active) = params.is_active {
            self.is_active = is_active;
        }
        if let Some(metadata) = params.metadata {
            self.metadata = metadata;
        }

        self.base.touch();
        self.base.increment_version();

        Ok(())
    }

    /// Increment usage count when payment is made
    pub fn increment_usage(&mut self) -> AppResult<()> {
        // Check if still usable
        if !self.is_usable() {
            return Err(AppError::validation_error("Payment link is no longer usable"));
        }

        self.current_uses += 1;
        self.base.touch();

        Ok(())
    }

    /// Check if the payment link is still usable
    pub fn is_usable(&self) -> bool {
        if !self.is_active {
            return false;
        }

        // Check expiration
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }

        // Check max uses (None = unlimited)
        if let Some(max) = self.max_uses {
            if self.current_uses >= max {
                return false;
            }
        }

        true
    }

    /// Deactivate the payment link
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.base.touch();
    }

    /// Activate the payment link
    pub fn activate(&mut self) {
        self.is_active = true;
        self.base.touch();
    }

    /// Compute link hash for smart contract verification
    pub fn compute_link_hash(&self) -> [u8; 32] {
        // Simple hash placeholder - in production use sha3::Keccak256
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.slug.hash(&mut hasher);
        let hash = hasher.finish();
        // Pad to 32 bytes
        let mut result = [0u8; 32];
        result[0..8].copy_from_slice(&hash.to_le_bytes());
        result
    }

    // Getters
    pub fn id(&self) -> &PaymentContextId { &self.id }
    pub fn context_type(&self) -> PaymentContextType { self.context_type }
    pub fn context_id(&self) -> Option<&Uuid> { self.context_id.as_ref() }
    pub fn slug(&self) -> &str { &self.slug }
    pub fn name(&self) -> &str { &self.name }
    pub fn description(&self) -> Option<&str> { self.description.as_deref() }
    pub fn amount(&self) -> rust_decimal::Decimal { self.amount }
    pub fn currency(&self) -> &str { &self.currency }
    pub fn expires_at(&self) -> Option<DateTime<Utc>> { self.expires_at }
    pub fn max_uses(&self) -> Option<i32> { self.max_uses }
    pub fn current_uses(&self) -> i32 { self.current_uses }
    pub fn metadata(&self) -> &serde_json::Value { &self.metadata }
    pub fn is_active(&self) -> bool { self.is_active }
    pub fn created_by(&self) -> &str { &self.created_by }
}

impl AggregateRoot for PaymentContext {
    type Id = PaymentContextId;

    fn id(&self) -> &Self::Id { &self.id }
    fn version(&self) -> u64 { self.base.version }
    fn increment_version(&mut self) { self.base.increment_version(); }
    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] { self.base.uncommitted_events() }
    fn mark_events_as_committed(&mut self) { self.base.mark_events_as_committed(); }
    fn created_at(&self) -> DateTime<Utc> { self.base.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.base.updated_at }
    fn touch(&mut self) { self.base.touch(); }
}

/// Payment Context Error
#[derive(Debug, thiserror::Error)]
pub enum PaymentContextError {
    #[error("Payment link expired")]
    Expired,
    #[error("Payment link max uses reached")]
    MaxUsesReached,
    #[error("Payment link is inactive")]
    Inactive,
    #[error("Invalid context type: {0}")]
    InvalidContextType(String),
}
