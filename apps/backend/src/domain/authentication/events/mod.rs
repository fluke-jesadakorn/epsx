use crate::domain::shared_kernel::value_objects::UserId;use crate::domain::shared_kernel::value_objects::SessionId;use chrono::{DateTime, Utc};// Authentication Domain Events
// Events published by the Authentication bounded context

use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::domain::shared_kernel::DomainEvent;
use super::value_objects::*;
use super::aggregates::authentication_session::TerminationReason;

/// Event published when a new authentication session is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationSessionCreatedEvent {
    event_id: Uuid,
    session_id: SessionId,
    user_id: AuthenticatedUserId,
    occurred_at: DateTime<Utc>,
    aggregate_version: u64,
}

impl AuthenticationSessionCreatedEvent {
    pub fn new(
        session_id: SessionId,
        user_id: AuthenticatedUserId,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            session_id,
            user_id,
            occurred_at,
            aggregate_version: 1,
        }
    }
    
    pub fn session_id(&self) -> &SessionId { &self.session_id }
    pub fn user_id(&self) -> &AuthenticatedUserId { &self.user_id }
}

impl Into<Box<dyn crate::domain::shared_kernel::DomainEvent>> for AuthenticationSessionCreatedEvent {
    fn into(self) -> Box<dyn crate::domain::shared_kernel::DomainEvent> {
        Box::new(self)
    }
}

impl DomainEvent for AuthenticationSessionCreatedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "authentication.session_created"
    }
    
    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Event published when tokens are issued for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokensIssuedEvent {
    event_id: Uuid,
    session_id: SessionId,
    user_id: AuthenticatedUserId,
    scopes: Vec<Scope>,
    occurred_at: DateTime<Utc>,
    aggregate_version: u64,
}

impl TokensIssuedEvent {
    pub fn new(
        session_id: SessionId,
        user_id: AuthenticatedUserId,
        scopes: Vec<Scope>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            session_id,
            user_id,
            scopes,
            occurred_at: Utc::now(),
            aggregate_version: 1,
        }
    }
    
    pub fn session_id(&self) -> &SessionId { &self.session_id }
    pub fn user_id(&self) -> &AuthenticatedUserId { &self.user_id }
    pub fn scopes(&self) -> &[Scope] { &self.scopes }
}

impl Into<Box<dyn crate::domain::shared_kernel::DomainEvent>> for TokensIssuedEvent {
    fn into(self) -> Box<dyn crate::domain::shared_kernel::DomainEvent> {
        Box::new(self)
    }
}

impl DomainEvent for TokensIssuedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "authentication.tokens_issued"
    }
    
    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Event published when tokens are refreshed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokensRefreshedEvent {
    event_id: Uuid,
    session_id: SessionId,
    user_id: AuthenticatedUserId,
    occurred_at: DateTime<Utc>,
    aggregate_version: u64,
}

impl TokensRefreshedEvent {
    pub fn new(
        session_id: SessionId,
        user_id: AuthenticatedUserId,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            session_id,
            user_id,
            occurred_at: Utc::now(),
            aggregate_version: 1,
        }
    }
    
    pub fn session_id(&self) -> &SessionId { &self.session_id }
    pub fn user_id(&self) -> &AuthenticatedUserId { &self.user_id }
}

impl Into<Box<dyn crate::domain::shared_kernel::DomainEvent>> for TokensRefreshedEvent {
    fn into(self) -> Box<dyn crate::domain::shared_kernel::DomainEvent> {
        Box::new(self)
    }
}

impl DomainEvent for TokensRefreshedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "authentication.tokens_refreshed"
    }
    
    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Event published when an authentication session is terminated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationSessionTerminatedEvent {
    event_id: Uuid,
    session_id: SessionId,
    user_id: AuthenticatedUserId,
    reason: TerminationReason,
    occurred_at: DateTime<Utc>,
    aggregate_version: u64,
}

impl AuthenticationSessionTerminatedEvent {
    pub fn new(
        session_id: SessionId,
        user_id: AuthenticatedUserId,
        reason: TerminationReason,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            session_id,
            user_id,
            reason,
            occurred_at: Utc::now(),
            aggregate_version: 1,
        }
    }
    
    pub fn session_id(&self) -> &SessionId { &self.session_id }
    pub fn user_id(&self) -> &AuthenticatedUserId { &self.user_id }
    pub fn reason(&self) -> &TerminationReason { &self.reason }
}

impl Into<Box<dyn crate::domain::shared_kernel::DomainEvent>> for AuthenticationSessionTerminatedEvent {
    fn into(self) -> Box<dyn crate::domain::shared_kernel::DomainEvent> {
        Box::new(self)
    }
}

impl DomainEvent for AuthenticationSessionTerminatedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "authentication.session_terminated"
    }
    
    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Event published when suspicious activity is detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousActivityDetectedEvent {
    event_id: Uuid,
    session_id: SessionId,
    user_id: AuthenticatedUserId,
    client_info: ClientInformation,
    occurred_at: DateTime<Utc>,
    aggregate_version: u64,
}

impl SuspiciousActivityDetectedEvent {
    pub fn new(
        session_id: SessionId,
        user_id: AuthenticatedUserId,
        client_info: ClientInformation,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            session_id,
            user_id,
            client_info,
            occurred_at: Utc::now(),
            aggregate_version: 1,
        }
    }
    
    pub fn session_id(&self) -> &SessionId { &self.session_id }
    pub fn user_id(&self) -> &AuthenticatedUserId { &self.user_id }
    pub fn client_info(&self) -> &ClientInformation { &self.client_info }
}

impl Into<Box<dyn crate::domain::shared_kernel::DomainEvent>> for SuspiciousActivityDetectedEvent {
    fn into(self) -> Box<dyn crate::domain::shared_kernel::DomainEvent> {
        Box::new(self)
    }
}

impl DomainEvent for SuspiciousActivityDetectedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "authentication.suspicious_activity_detected"
    }
    
    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Event published when authentication fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationFailedEvent {
    event_id: Uuid,
    attempted_user_id: Option<String>,
    failure_reason: AuthenticationFailureReason,
    client_info: ClientInformation,
    occurred_at: DateTime<Utc>,
    aggregate_version: u64,
}

impl AuthenticationFailedEvent {
    pub fn new(
        attempted_user_id: Option<String>,
        failure_reason: AuthenticationFailureReason,
        client_info: ClientInformation,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            attempted_user_id,
            failure_reason,
            client_info,
            occurred_at: Utc::now(),
            aggregate_version: 1,
        }
    }
    
    pub fn attempted_user_id(&self) -> Option<&str> { self.attempted_user_id.as_deref() }
    pub fn failure_reason(&self) -> &AuthenticationFailureReason { &self.failure_reason }
    pub fn client_info(&self) -> &ClientInformation { &self.client_info }
}

impl DomainEvent for AuthenticationFailedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "authentication.authentication_failed"
    }
    
    fn aggregate_id(&self) -> String {
        self.attempted_user_id
            .as_ref()
            .map(|id| format!("auth_failure:{}", id))
            .unwrap_or_else(|| format!("auth_failure:unknown:{}", Utc::now().timestamp()))
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Reasons for authentication failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationFailureReason {
    InvalidCredentials,
    ExpiredCredentials,
    AccountLocked,
    TooManyAttempts,
    InvalidClient,
    InvalidScope,
    UnsupportedGrantType,
    InvalidGrant,
    UnauthorizedClient,
    AccessDenied,
    ServerError,
}

/// Event published when a user session is upgraded (e.g., MFA completed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUpgradedEvent {
    event_id: Uuid,
    session_id: SessionId,
    user_id: AuthenticatedUserId,
    upgrade_type: SessionUpgradeType,
    new_scopes: Vec<Scope>,
    occurred_at: DateTime<Utc>,
    aggregate_version: u64,
}

impl SessionUpgradedEvent {
    pub fn new(
        session_id: SessionId,
        user_id: AuthenticatedUserId,
        upgrade_type: SessionUpgradeType,
        new_scopes: Vec<Scope>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            session_id,
            user_id,
            upgrade_type,
            new_scopes,
            occurred_at: Utc::now(),
            aggregate_version: 1,
        }
    }
}

impl DomainEvent for SessionUpgradedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn event_type(&self) -> &'static str {
        "authentication.session_upgraded"
    }
    
    fn aggregate_id(&self) -> String {
        self.session_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Types of session upgrades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionUpgradeType {
    MfaCompleted,
    AdminRightsGranted,
    AdditionalScopesGranted,
    SecurityLevelIncreased,
}

// All events are already public - no need for re-export
// Events defined above are automatically available for import