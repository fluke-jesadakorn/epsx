# ADR-002: Bounded Contexts Design and Implementation

## Status
✅ **ACCEPTED** - Implemented

## Date
2025-01-05

## Context

As part of the Domain-Driven Design implementation, we needed to identify and implement proper bounded contexts for the EPSX platform. Each bounded context should have clear boundaries, unified language, and isolated domain models.

## Decision

We have implemented **7 primary bounded contexts** based on business capabilities and data ownership:

## 1. Authentication Bounded Context

**Responsibility**: User authentication, session creation, token management

### Domain Model
```rust
// Aggregates
pub struct AuthenticationSession {
    id: SessionId,
    user_id: AuthenticatedUserId,
    provider: AuthenticationProvider,
    security_context: SecurityContext,
    tokens: TokenSet,
    status: SessionStatus,
}

// Value Objects  
pub struct SessionId(Uuid);
pub struct AuthenticatedUserId(Uuid);
pub struct SecurityContext { /* IP tracking, anomaly detection */ }

// Domain Events
pub struct AuthenticationSessionCreatedEvent;
pub struct TokensIssuedEvent;
pub struct SessionTerminatedEvent;
```

### Responsibilities
- ✅ Session lifecycle management
- ✅ Multi-provider authentication (Firebase, OIDC)
- ✅ Security context tracking
- ✅ Token issuance and validation
- ✅ Anomaly detection

## 2. User Management Bounded Context

**Responsibility**: User profiles, permissions, account lifecycle

### Domain Model
```rust
// Aggregates
pub struct User {
    id: UserId,
    profile: UserProfile,
    permissions: Vec<Permission>,
    account_status: AccountStatus,
    audit_trail: Vec<AuditEvent>,
}

pub struct Session {
    id: SessionId,
    user_id: UserId,
    device_info: DeviceInfo,
    permissions: Vec<Permission>,
}

// Value Objects
pub struct Permission {
    platform: String,
    resource: String, 
    action: String,
    expires_at: Option<DateTime<Utc>>,
}

// Domain Events
pub struct UserCreatedEvent;
pub struct PermissionGrantedEvent;
pub struct UserProfileUpdatedEvent;
```

### Responsibilities
- ✅ User account management
- ✅ Permission system (platform:resource:action format)
- ✅ Embedded timestamp permissions
- ✅ User profile data
- ✅ Account status management

## 3. Session Management Bounded Context

**Responsibility**: Session persistence, monitoring, analytics

### Domain Model
```rust
// Aggregates
pub struct UserSessionManager {
    user_id: AuthenticatedUserId,
    sessions: SessionCollection,
    security_assessment: SecurityAssessment,
    anomaly_patterns: Vec<SuspiciousPattern>,
}

// Value Objects
pub struct SessionCollection {
    active_sessions: HashMap<SessionId, SessionMetadata>,
    max_concurrent: u32,
}

pub struct SuspiciousPattern {
    pattern_type: PatternType,
    severity_level: SeverityLevel,
    confidence_score: u8,
    evidence: Vec<Evidence>,
}

// Domain Events
pub struct SessionActivityRecorded;
pub struct SuspiciousActivityDetected;
```

### Responsibilities  
- ✅ Session persistence and TTL management
- ✅ Multi-session user tracking
- ✅ Behavioral analysis and anomaly detection
- ✅ Session analytics and reporting
- ✅ Device fingerprinting

## 4. Payment Bounded Context

**Responsibility**: Payment processing, crypto addresses, transactions

### Domain Model
```rust
// Aggregates
pub struct Payment {
    id: PaymentId,
    user_id: UserId,
    amount: PaymentAmount,
    method: PaymentMethod,
    status: PaymentStatus,
    transactions: Vec<TransactionHash>,
}

// Value Objects
pub struct PaymentAmount {
    amount: Decimal,
    currency: Currency,
}

pub struct CryptoNetwork {
    name: String,
    chain_id: u64,
    confirmation_blocks: u32,
}

pub struct PaymentMethodId(Uuid);
pub struct CryptoAddressId(Uuid);

// Domain Events
pub struct PaymentInitiatedEvent;
pub struct PaymentConfirmedEvent;
pub struct PaymentFailedEvent;
```

### Responsibilities
- ✅ Payment lifecycle management  
- ✅ Multiple payment methods (crypto, fiat)
- ✅ Multi-network crypto support
- ✅ Transaction tracking and confirmation
- ✅ Exchange rate management

## 5. Notification Bounded Context

**Responsibility**: Multi-channel notifications, preferences, delivery

### Domain Model
```rust
// Aggregates
pub struct Notification {
    id: NotificationId,
    recipient: UserId,
    content: NotificationContent,
    channels: Vec<DeliveryChannel>,
    status: NotificationStatus,
    delivery_attempts: u32,
}

// Value Objects
pub struct NotificationContent {
    title: String,
    body: String,
    data: HashMap<String, String>,
}

pub struct UserPreferences {
    quiet_hours: Option<QuietPeriod>,
    enabled_channels: Vec<DeliveryChannel>,
    topics: Vec<NotificationTopic>,
}

// Domain Events  
pub struct NotificationCreatedEvent;
pub struct NotificationDeliveredEvent;
pub struct DeliveryFailedEvent;
```

### Responsibilities
- ✅ Multi-channel delivery (FCM, email, SMS, in-app)
- ✅ User notification preferences
- ✅ Topic-based subscriptions
- ✅ Delivery tracking and retry logic
- ✅ Quiet hours and DND management

## 6. Realtime Events Bounded Context

**Responsibility**: Live event broadcasting, WebSocket connections, SSE

### Domain Model
```rust
// Aggregates  
pub struct RealtimeEvent {
    id: EventId,
    payload: EventPayload,
    target_users: Vec<UserId>,
    channel: String,
    priority: EventPriority,
    delivery_attempts: u32,
}

// Value Objects
pub struct ConnectionInfo {
    user_agent: Option<String>,
    ip_address: String,
    connection_type: ConnectionType,
    connected_at: DateTime<Utc>,
}

pub struct EventChannel {
    name: String,
    subscribers: Vec<ConnectionId>,
}

// Domain Events
pub struct EventCreatedEvent;
pub struct EventDeliveredEvent;  
pub struct ConnectionEstablishedEvent;
```

### Responsibilities
- ✅ WebSocket and SSE connection management
- ✅ Real-time event broadcasting
- ✅ Channel-based subscriptions
- ✅ Connection lifecycle tracking
- ✅ Event delivery and retry logic

## 7. Trading Analytics Bounded Context

**Responsibility**: EPS analysis, stock rankings, financial metrics

### Domain Model
```rust
// Aggregates
pub struct EPSRanking {
    stock_symbol: StockSymbol,
    eps_value: EPSValue,
    growth_factor: GrowthFactor,
    market_sector: MarketSector,
    country: Country,
    ranking_position: u32,
}

pub struct StockAnalysis {
    symbol: StockSymbol,
    analysis_date: DateTime<Utc>,
    eps_metrics: EPSMetrics,
    growth_analysis: GrowthAnalysis,
}

// Value Objects
pub struct EPSValue {
    value: f64,
}

pub struct GrowthFactor {
    percentage: f64,
}

// Domain Events
pub struct EPSRankingUpdatedEvent;
pub struct StockAnalysisCompletedEvent;
```

### Responsibilities
- ✅ EPS (Earnings Per Share) calculations
- ✅ Growth factor analysis  
- ✅ Stock ranking algorithms
- ✅ Sector and country categorization
- ✅ Performance analytics

## Context Boundaries and Integration

### Interaction Patterns

#### 1. Authentication → User Management
```rust
// Authentication creates user sessions
pub struct AuthenticationSessionCreatedEvent {
    session_id: SessionId,
    user_id: AuthenticatedUserId, // Shared identifier
}
```

#### 2. User Management ← Session Management  
```rust
// Session management monitors user sessions
pub struct SessionActivityRecorded {
    user_id: AuthenticatedUserId, // Reference to User Management
    session_data: SessionMetadata,
}
```

#### 3. User Management → Payment
```rust
// Users initiate payments
pub struct PaymentInitiatedEvent {
    user_id: UserId, // Shared user reference
    payment_id: PaymentId,
}
```

#### 4. User Management → Notification
```rust  
// Notifications target users
pub struct NotificationCreatedEvent {
    recipient: UserId, // Reference to User Management
    content: NotificationContent,
}
```

#### 5. All Contexts → Realtime Events
```rust
// Any context can publish real-time events
pub struct RealtimeEvent {
    source_context: BoundedContext,
    target_users: Vec<UserId>,
    payload: EventPayload,
}
```

### Integration Principles

#### 1. **Shared Kernel**
- `UserId` - Universal user identifier
- `SessionId` - Session reference across contexts
- `DomainEvent` - Event interface
- `AggregateRoot` - Common aggregate behavior

#### 2. **Anti-Corruption Layer**
- Each context translates external events to internal models
- Legacy system integration isolated to adapters
- Clear transformation boundaries

#### 3. **Published Language**
- Domain events use consistent naming
- Shared value objects for common concepts
- Ubiquitous language within each context

### Context Communication

#### Synchronous Integration
```rust
// Repository ports for direct queries
pub trait UserRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, String>;
}

// Used by other contexts for user validation
```

#### Asynchronous Integration
```rust
// Domain events for eventual consistency  
pub trait DomainEventBus: Send + Sync {
    fn publish(&self, event: &Box<dyn DomainEvent>);
}

// Events flow between contexts
```

## Implementation Benefits

### 1. **Clear Boundaries**
- Each context owns specific data
- No shared mutable state between contexts
- Clear APIs for inter-context communication

### 2. **Independent Evolution**  
- Contexts can evolve at different rates
- Technology choices isolated per context
- Team ownership per bounded context

### 3. **Scalability**
- Contexts can be extracted to microservices
- Independent scaling based on load patterns
- Database per context if needed

### 4. **Maintainability**
- Changes contained within context boundaries
- Clear ownership and responsibility
- Reduced cognitive load per context

## Context Size and Complexity

| Context | Aggregates | Value Objects | Events | Complexity |
|---------|------------|---------------|---------|------------|
| Authentication | 1 | 6 | 7 | Medium |
| User Management | 2 | 8 | 6 | High |
| Session Management | 1 | 12 | 4 | Medium |
| Payment | 1 | 9 | 5 | Medium |
| Notification | 1 | 7 | 3 | Low |  
| Realtime Events | 1 | 8 | 7 | Medium |
| Trading Analytics | 2 | 6 | 4 | Low |

## Future Evolution

### Microservices Extraction
1. **Authentication Service** - High security isolation
2. **User Management Service** - Core platform service  
3. **Payment Service** - PCI compliance isolation
4. **Analytics Service** - Compute-heavy workloads

### Context Refinement
- Monitor context boundaries for seams
- Split contexts that grow too large
- Merge contexts that are too coupled

## References

- **Domain-Driven Design** by Eric Evans - Chapter 14 (Bounded Contexts)
- **Implementing Domain-Driven Design** by Vaughn Vernon - Part II
- **Microservices Patterns** by Chris Richardson

---

**Domain Experts**: Product Team, Architecture Team  
**Implementation Date**: 2025-01-05  
**Review Date**: 2025-04-05  
**Status**: ✅ Production Ready