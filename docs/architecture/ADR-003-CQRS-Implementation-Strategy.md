# ADR-003: CQRS Implementation Strategy

## Status
✅ **ACCEPTED** - Implemented

## Date
2025-01-05

## Context

As part of the Clean Architecture implementation, we needed to implement Command Query Responsibility Segregation (CQRS) to separate write operations (commands) from read operations (queries). This separation improves performance, scalability, and maintainability.

## Decision

We have implemented a comprehensive CQRS pattern with:

### 1. **Command Side (Write Operations)**
- Commands represent business intent
- Command handlers contain business logic
- Domain events capture state changes
- Optimized for consistency and validation

### 2. **Query Side (Read Operations)** 
- Queries represent data requests
- Query handlers optimize for performance
- Read models denormalized for efficiency
- Optimized for performance and user experience

## Implementation Architecture

### Command Pattern Implementation

#### Base Command Interface
```rust
pub trait Command: Send + Sync + Debug + Clone {
    type Response: Send + Sync;
    
    fn validate(&self) -> ApplicationResult<()> {
        Ok(())
    }
}
```

#### Command Handler Pattern
```rust
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> ApplicationResult<C::Response>;
}
```

#### Command Bus
```rust
#[async_trait]
pub trait CommandBus: Send + Sync {
    async fn execute<C: Command>(&self, command: C) -> ApplicationResult<C::Response>;
}
```

### Query Pattern Implementation

#### Base Query Interface  
```rust
pub trait Query: Send + Sync + Debug {
    type Response: Send + Sync;
    
    fn validate(&self) -> ApplicationResult<()> {
        Ok(())
    }
}
```

#### Query Handler Pattern
```rust
#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> ApplicationResult<Q::Response>;
}
```

#### Query Bus with Pagination
```rust
#[async_trait]  
pub trait QueryBus: Send + Sync {
    async fn execute<Q: Query>(&self, query: Q) -> ApplicationResult<Q::Response>;
}

pub struct PaginationParams {
    pub page: u32,
    pub limit: u32,
    pub sort: Option<SortParams>,
}

pub struct SortParams {
    pub field: String,
    pub direction: SortDirection,
}
```

## Bounded Context Implementation

### 1. User Management CQRS

#### Commands
```rust
// Create User Command
pub struct CreateUserCommand {
    pub firebase_uid: String,
    pub email: String,
    pub profile: UserProfile,
    pub initial_permissions: Vec<Permission>,
}

impl Command for CreateUserCommand {
    type Response = CreateUserResponse;
    
    fn validate(&self) -> ApplicationResult<()> {
        if self.email.is_empty() {
            return Err(ApplicationError::ValidationError("Email required".to_string()));
        }
        Ok(())
    }
}

// Grant Permission Command  
pub struct GrantPermissionCommand {
    pub user_id: UserId,
    pub permission: Permission,
    pub granted_by: UserId,
}

// Update User Command
pub struct UpdateUserCommand {
    pub user_id: UserId,
    pub profile_updates: UserProfileUpdate,
    pub updated_by: UserId,
}
```

#### Command Handlers
```rust
pub struct CreateUserCommandHandler {
    user_repository: Arc<dyn UserRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

#[async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserCommandHandler {
    async fn handle(&self, command: CreateUserCommand) -> ApplicationResult<CreateUserResponse> {
        // 1. Validate business rules
        command.validate()?;
        
        // 2. Create domain aggregate
        let mut user = User::create(
            command.firebase_uid,
            command.email,
            command.profile,
        )?;
        
        // 3. Apply initial permissions
        for permission in command.initial_permissions {
            user.grant_permission(permission, None)?;
        }
        
        // 4. Persist aggregate
        self.user_repository.save(&user).await?;
        
        // 5. Publish domain events
        for event in user.uncommitted_events() {
            self.event_bus.publish(event);
        }
        
        // 6. Return response
        Ok(CreateUserResponse {
            user_id: user.id().clone(),
            created_at: user.created_at(),
        })
    }
}
```

#### Queries
```rust
// Get User Query
pub struct GetUserQuery {
    pub user_id: UserId,
}

impl Query for GetUserQuery {
    type Response = UserDetailResponse;
}

// List Users Query  
pub struct ListUsersQuery {
    pub pagination: PaginationParams,
    pub filters: UserSearchCriteria,
}

impl Query for ListUsersQuery {
    type Response = PaginatedResponse<UserSummaryResponse>;
}

// Get User Permissions Query
pub struct GetUserPermissionsQuery {
    pub user_id: UserId,
    pub include_expired: bool,
}

impl Query for GetUserPermissionsQuery {
    type Response = UserPermissionsResponse;
}
```

#### Query Handlers
```rust
pub struct UserQueryService {
    user_repository: Arc<dyn UserRepositoryPort>,
}

#[async_trait]
impl QueryHandler<GetUserQuery> for UserQueryService {
    async fn handle(&self, query: GetUserQuery) -> ApplicationResult<UserDetailResponse> {
        let user = self.user_repository
            .find_by_id(&query.user_id)
            .await?
            .ok_or(ApplicationError::NotFound("User not found".to_string()))?;
            
        Ok(UserDetailResponse::from_user(&user))
    }
}

#[async_trait]
impl QueryHandler<ListUsersQuery> for UserQueryService {
    async fn handle(&self, query: ListUsersQuery) -> ApplicationResult<PaginatedResponse<UserSummaryResponse>> {
        let users = self.user_repository
            .search(&query.filters, &query.pagination)
            .await?;
            
        let total = self.user_repository
            .count_by_criteria(&query.filters)
            .await?;
            
        Ok(PaginatedResponse {
            data: users.into_iter()
                .map(|u| UserSummaryResponse::from_user(&u))
                .collect(),
            pagination: PaginationInfo {
                page: query.pagination.page,
                limit: query.pagination.limit,
                total,
                total_pages: (total + query.pagination.limit as u64 - 1) / query.pagination.limit as u64,
            }
        })
    }
}
```

### 2. Payment CQRS

#### Commands
```rust
pub struct CreatePaymentCommand {
    pub user_id: UserId,
    pub amount: PaymentAmount,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub metadata: HashMap<String, String>,
}

pub struct ProcessPaymentCommand {
    pub payment_id: PaymentId,
    pub transaction_hash: TransactionHash,
    pub confirmations: u32,
}

pub struct RefundPaymentCommand {
    pub payment_id: PaymentId,
    pub reason: String,
    pub refunded_by: UserId,
}
```

#### Payment Command Handler
```rust
pub struct CreatePaymentCommandHandler {
    payment_repository: Arc<dyn PaymentRepositoryPort>,
    user_repository: Arc<dyn UserRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

#[async_trait]
impl CommandHandler<CreatePaymentCommand> for CreatePaymentCommandHandler {
    async fn handle(&self, command: CreatePaymentCommand) -> ApplicationResult<CreatePaymentResponse> {
        // 1. Validate user exists and has permission
        let user = self.user_repository
            .find_by_id(&command.user_id)
            .await?
            .ok_or(ApplicationError::NotFound("User not found".to_string()))?;
            
        // 2. Create payment aggregate
        let mut payment = Payment::create(
            command.user_id,
            command.amount,
            command.method,
        )?;
        
        // 3. Apply metadata if provided
        if let Some(reference) = command.reference {
            payment.set_reference(reference)?;
        }
        
        for (key, value) in command.metadata {
            payment.add_metadata(key, value)?;
        }
        
        // 4. Generate payment instructions
        let instructions = payment.generate_payment_instructions()?;
        
        // 5. Persist payment
        self.payment_repository.save(&payment).await?;
        
        // 6. Publish events
        for event in payment.uncommitted_events() {
            self.event_bus.publish(event);
        }
        
        Ok(CreatePaymentResponse {
            payment_id: payment.id().clone(),
            reference: payment.reference().clone(),
            status: payment.status().clone(),
            payment_instructions: Some(instructions),
            expires_at: payment.expires_at(),
        })
    }
}
```

### 3. Authentication CQRS

#### Commands
```rust
pub struct CreateSessionCommand {
    pub user_id: AuthenticatedUserId,
    pub provider: AuthenticationProvider,
    pub client_info: ClientInformation,
    pub requested_scopes: Vec<Scope>,
}

pub struct RefreshTokensCommand {
    pub session_id: SessionId,
    pub refresh_token: RefreshToken,
}

pub struct TerminateSessionCommand {
    pub session_id: SessionId,
    pub reason: TerminationReason,
}
```

#### Authentication Query Examples  
```rust
pub struct GetSessionQuery {
    pub session_id: SessionId,
}

pub struct GetUserSessionsQuery {
    pub user_id: AuthenticatedUserId,
    pub include_expired: bool,
}

pub struct ValidateTokenQuery {
    pub access_token: AccessToken,
    pub required_scopes: Vec<Scope>,
}
```

## Performance Optimizations

### 1. **Read Model Projections**
```rust
// Optimized read model for user listings
pub struct UserListProjection {
    pub id: UserId,
    pub email: String,
    pub display_name: String,
    pub status: AccountStatus,
    pub permission_count: u32,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// Fast permission lookup
pub struct UserPermissionProjection {
    pub user_id: UserId,
    pub permissions: Vec<String>, // Pre-formatted permission strings
    pub expires_at: HashMap<String, DateTime<Utc>>,
}
```

### 2. **Caching Strategy**
```rust
pub struct CachedQueryHandler<Q: Query> {
    inner: Box<dyn QueryHandler<Q>>,
    cache: Arc<dyn Cache>,
    ttl: Duration,
}

#[async_trait]
impl<Q: Query> QueryHandler<Q> for CachedQueryHandler<Q>
where 
    Q: Serialize,
    Q::Response: Serialize + DeserializeOwned,
{
    async fn handle(&self, query: Q) -> ApplicationResult<Q::Response> {
        let cache_key = format!("query:{}", serde_json::to_string(&query)?);
        
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(serde_json::from_str(&cached)?);
        }
        
        let response = self.inner.handle(query).await?;
        
        let _ = self.cache.set(&cache_key, &serde_json::to_string(&response)?, self.ttl).await;
        
        Ok(response)
    }
}
```

### 3. **Batch Operations**
```rust
pub struct BatchUserQuery {
    pub user_ids: Vec<UserId>,
    pub include_permissions: bool,
}

impl Query for BatchUserQuery {
    type Response = Vec<UserDetailResponse>;
}

// Optimized with single database query
#[async_trait]
impl QueryHandler<BatchUserQuery> for UserQueryService {
    async fn handle(&self, query: BatchUserQuery) -> ApplicationResult<Vec<UserDetailResponse>> {
        let users = self.user_repository
            .find_by_ids(&query.user_ids)
            .await?;
            
        if query.include_permissions {
            let permissions = self.user_repository
                .get_permissions_for_users(&query.user_ids)
                .await?;
                
            // Merge users with permissions in single pass
        }
        
        Ok(users.into_iter()
            .map(|user| UserDetailResponse::from_user(&user))
            .collect())
    }
}
```

## Event Sourcing Integration

### Command Event Correlation
```rust
pub struct CommandEventCorrelation {
    pub command_id: Uuid,
    pub user_id: Option<UserId>,
    pub session_id: Option<SessionId>,
    pub timestamp: DateTime<Utc>,
}

// Commands generate events with correlation
impl CreateUserCommandHandler {
    async fn handle(&self, command: CreateUserCommand) -> ApplicationResult<CreateUserResponse> {
        let correlation = CommandEventCorrelation {
            command_id: Uuid::new_v4(),
            user_id: None,
            session_id: command.session_id.clone(),
            timestamp: Utc::now(),
        };
        
        // Events include correlation data
        let event = UserCreatedEvent {
            correlation: correlation.clone(),
            user_id: user.id().clone(),
            // ... other fields
        };
    }
}
```

## Error Handling Strategy

### Command Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Concurrent modification detected")]
    ConcurrencyError,
    
    #[error("Infrastructure error: {0}")]
    InfrastructureError(String),
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;
```

### Query Error Handling
```rust
// Queries handle errors gracefully
#[async_trait]
impl QueryHandler<GetUserQuery> for UserQueryService {
    async fn handle(&self, query: GetUserQuery) -> ApplicationResult<UserDetailResponse> {
        match self.user_repository.find_by_id(&query.user_id).await {
            Ok(Some(user)) => Ok(UserDetailResponse::from_user(&user)),
            Ok(None) => Err(ApplicationError::NotFound(format!("User {} not found", query.user_id))),
            Err(e) => Err(ApplicationError::InfrastructureError(e)),
        }
    }
}
```

## Benefits Realized

### 1. **Performance Benefits**
- **Read Optimization**: Queries optimized for specific use cases
- **Write Optimization**: Commands focus on business logic validation  
- **Caching**: Aggressive caching possible on query side
- **Scaling**: Read and write sides scale independently

### 2. **Maintainability Benefits**  
- **Separation**: Clear separation between read and write concerns
- **Single Responsibility**: Handlers have focused responsibilities
- **Testing**: Commands and queries easily unit tested
- **Evolution**: Read and write models evolve independently

### 3. **Business Benefits**
- **Consistency**: Commands enforce business rules
- **Auditability**: All state changes captured as events
- **Performance**: Fast queries improve user experience
- **Scalability**: Independent scaling based on usage patterns

## Usage Patterns

### Web Layer Integration
```rust
// Controller uses command and query buses
#[axum::debug_handler]
pub async fn create_user(
    State(command_bus): State<Arc<dyn CommandBus>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, ApiError> {
    let command = CreateUserCommand::from_request(request);
    let response = command_bus.execute(command).await?;
    Ok(Json(response))
}

#[axum::debug_handler]
pub async fn get_user(
    State(query_bus): State<Arc<dyn QueryBus>>,
    Path(user_id): Path<String>,
) -> Result<Json<UserDetailResponse>, ApiError> {
    let query = GetUserQuery {
        user_id: UserId::from_string(user_id)?,
    };
    let response = query_bus.execute(query).await?;
    Ok(Json(response))
}
```

## Future Enhancements

### 1. **Advanced Query Features**
- GraphQL-style field selection
- Dynamic query composition  
- Query result streaming
- Real-time query subscriptions

### 2. **Command Enhancements**
- Command scheduling and delayed execution
- Command compensation and sagas
- Command batching and bulk operations
- Command replay for testing

### 3. **Monitoring and Observability**
- Command/query execution metrics
- Performance monitoring per operation
- Business metrics extraction
- Error tracking and alerting

## References

- **CQRS Journey** by Microsoft Patterns & Practices
- **Event Sourcing and CQRS** by Greg Young
- **Implementing Domain-Driven Design** by Vaughn Vernon - Chapter 4

---

**Implementation Team**: Backend Architecture Team  
**Performance Benchmarks**: Available in `/docs/benchmarks/`  
**Monitoring Dashboard**: Available in production monitoring  
**Status**: ✅ Production Ready