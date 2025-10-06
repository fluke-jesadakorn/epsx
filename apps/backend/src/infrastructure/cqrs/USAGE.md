# CQRS Infrastructure Usage Guide

## Overview

The CQRS infrastructure provides **Application-Level Event Publishing** with the **Transactional Outbox Pattern**. This ensures that domain events are persisted atomically with aggregate state changes.

## Architecture

```
Command → Handler → TransactionalOutbox → [Aggregate Save + Event Store + Outbox] → Database
                                                    (Single Transaction)
```

## Key Components

1. **EventStore** - Immutable event log
2. **TransactionalOutbox** - Atomic event persistence
3. **OutboxEvents** - Queue for async publishing

## How to Use TransactionalOutbox

### Step 1: Get TransactionalOutbox from Container

```rust
use crate::infrastructure::TransactionalOutbox;

// In your handler or service
let outbox = container.transactional_outbox
    .as_ref()
    .expect("TransactionalOutbox not initialized");
```

### Step 2: Prepare Your Aggregate and Events

```rust
// 1. Load or create aggregate
let mut wallet = WalletUser::create(wallet_address, permission_groups)?;

// 2. Execute business logic (this creates events)
wallet.grant_permission(permission)?;

// 3. Get uncommitted events
let events = wallet.uncommitted_events().to_vec();
```

### Step 3: Save with TransactionalOutbox

```rust
use uuid::Uuid;

// Save aggregate + events atomically
outbox.save_with_events(
    wallet.wallet_address().as_str(),  // aggregate_id
    "WalletUser",                       // aggregate_type
    events,                             // domain events
    |tx| {
        // Callback to save aggregate state
        Box::pin(async move {
            sqlx::query!(
                r#"
                INSERT INTO wallet_users (wallet_address, is_active, created_at)
                VALUES ($1, $2, $3)
                ON CONFLICT (wallet_address) DO UPDATE SET
                    updated_at = NOW()
                "#,
                wallet.wallet_address().as_str(),
                wallet.is_active(),
                wallet.created_at()
            )
            .execute(&mut **tx)
            .await?;

            Ok(())
        })
    },
    Some(Uuid::new_v4()),  // causation_id (command ID)
    Some(trace_id),        // correlation_id (trace ID)
    Some(user_id),         // user who triggered this
).await?;

// 4. Mark events as committed
wallet.mark_events_as_committed();
```

## Complete Example: GrantPermissionCommandHandler (Production Pattern)

```rust
use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult};
use crate::application::wallet_management::commands::models::{
    GrantPermissionCommand, GrantPermissionResponse
};
use crate::domain::wallet_management::{WalletUserRepositoryPort, Permission, WalletAddress};
use crate::infrastructure::TransactionalOutbox;

pub struct GrantPermissionCommandHandler {
    wallet_repository: Arc<dyn WalletUserRepositoryPort>,
    outbox: Arc<TransactionalOutbox>,
}

#[async_trait]
impl CommandHandler<GrantPermissionCommand> for GrantPermissionCommandHandler {
    async fn handle(&self, command: GrantPermissionCommand) -> ApplicationResult<GrantPermissionResponse> {
        // 1. Parse inputs
        let wallet_addr = WalletAddress::new(command.wallet_address.clone())?;
        let permission = Permission::new(&command.permission)?;

        // 2. Load aggregate
        let mut wallet = self.wallet_repository
            .find_by_wallet(&wallet_addr)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Wallet", wallet_addr.to_string()))?;

        // 3. Execute business logic (creates events)
        wallet.grant_permission(permission.clone())?;

        // 4. Take events from aggregate
        let events = wallet.take_events();
        let aggregate_id = wallet.wallet_address().to_string();

        // 5. Save aggregate state via repository
        self.wallet_repository.save(&wallet).await?;

        // 6. Append events to outbox for async publishing
        // Events are persisted to event_store and outbox_events
        self.outbox.append_and_publish_events(
            &aggregate_id,
            "WalletUser",
            events,
            None, // causation_id
            None, // correlation_id
            command.granted_by.clone(), // user_id
        ).await?;

        // 7. Return response
        Ok(GrantPermissionResponse {
            wallet_address: wallet.wallet_address().to_string(),
            permission,
            granted_by: command.granted_by,
            granted_at: chrono::Utc::now(),
            expires_at: command.expires_at,
        })
    }
}
```

**Why this pattern?**
- ✅ Simple API without callback lifetime issues
- ✅ Events are persisted to event_store (immutable audit log)
- ✅ Events are queued in outbox for async publishing
- ⚠️ Not fully atomic with aggregate save (acceptable trade-off)
- ✅ Works with existing repository pattern

## What Happens Next?

After `save_with_events()` completes:

1. ✅ **Aggregate state** saved to `wallet_users` table
2. ✅ **Events** saved to `event_store` table (immutable log)
3. ✅ **Events** saved to `outbox_events` table (for publishing)
4. ✅ **All in one transaction** (ACID guarantees)

Then, **asynchronously**:

5. **EventDispatcher** (background worker) polls `outbox_events`
6. **Publishes** events to Redis Streams
7. **Projection workers** consume events and update read models

## Benefits

### 1. **Atomic Persistence**
```
✅ Aggregate state and events saved together
❌ No partial saves (either both succeed or both fail)
```

### 2. **Event Sourcing**
```
✅ Complete audit trail in event_store
✅ Temporal queries (state at any point in time)
✅ Event replay for debugging
```

### 3. **Reliable Event Publishing**
```
✅ Events guaranteed to be published (outbox pattern)
✅ No lost events even if publishing fails
✅ Automatic retries with exponential backoff
```

### 4. **Read/Write Separation (CQRS)**
```
✅ Write model (aggregates) separate from read model (projections)
✅ Optimized queries on denormalized read models
✅ Independent scaling of read and write sides
```

## Monitoring

Get outbox statistics:

```rust
let stats = outbox.get_stats().await?;

println!("Pending events: {}", stats.pending_count);
println!("Processed events: {}", stats.processed_count);
println!("Failed events: {}", stats.failed_count);
```

## Troubleshooting

### Events not being published?

Check if EventDispatcher is running:

```bash
# Look for dispatcher logs
grep "event dispatcher" logs/backend.log
```

### Outbox table growing?

Events stay in outbox forever (for audit). Archive old events:

```sql
-- Archive processed events older than 30 days
DELETE FROM outbox_events
WHERE processed = true
AND processed_at < NOW() - INTERVAL '30 days';
```

### How to replay events?

Load events from event_store:

```rust
let events = event_store.load_events(&aggregate_id, 0).await?;
let wallet = WalletUser::from_events(events)?;
```

## Best Practices

1. **Always use TransactionalOutbox for aggregate saves**
   ```rust
   ❌ repository.save(&wallet).await?;  // Old way
   ✅ outbox.save_with_events(...).await?;  // New way
   ```

2. **Include causation_id for debugging**
   ```rust
   let command_id = Uuid::new_v4();
   outbox.save_with_events(..., Some(command_id), ...).await?;
   ```

3. **Include correlation_id for distributed tracing**
   ```rust
   let trace_id = extract_trace_id(request)?;
   outbox.save_with_events(..., None, Some(trace_id), ...).await?;
   ```

4. **Mark events as committed after save**
   ```rust
   outbox.save_with_events(...).await?;
   wallet.mark_events_as_committed();  // Important!
   ```

## Starting EventDispatcher

The EventDispatcher runs as a background worker that publishes events from the outbox to Redis Streams:

```rust
// EventDispatcher is automatically created in DomainContainer
let container = SimpleContainer::new_with_web3_services(db_pool, cache, blockchain_config);

// Start the dispatcher (if configured in container)
if let Some(dispatcher) = &container.event_dispatcher {
    dispatcher.clone().start().await?;
    info!("EventDispatcher started - events will be published to Redis");
}
```

### Configuration

Set environment variable for Redis:
```bash
REDIS_URL=redis://localhost:6379
```

### Monitoring EventDispatcher

```rust
// Get dispatcher statistics
if let Some(dispatcher) = &container.event_dispatcher {
    let stats = dispatcher.get_stats().await?;
    println!("Pending events: {}", stats.pending_events);
    println!("Processed events: {}", stats.processed_events);
    println!("Failed events: {}", stats.failed_events);

    // Health check
    let health = dispatcher.health_check().await;
    println!("Dispatcher healthy: {}", health.overall_healthy);
}
```

### Event Flow with EventDispatcher

```
1. Command Handler
   ↓
2. TransactionalOutbox.save_with_events()
   ↓ (atomic transaction)
   ├─→ Aggregate state → wallet_users table
   ├─→ Events → event_store table (immutable log)
   └─→ Events → outbox_events table (for publishing)
   ↓
3. EventDispatcher (background worker)
   ├─→ Polls outbox_events every 5 seconds
   ├─→ Publishes to Redis Streams (domain_events)
   └─→ Marks events as processed
   ↓
4. Projection Workers (future)
   ├─→ Subscribe to Redis Streams
   ├─→ Update read models
   └─→ Track checkpoints
```

## Migration Path

### Phase 1: TransactionalOutbox ✅ COMPLETE
- ✅ Event store infrastructure created
- ✅ TransactionalOutbox available
- ⏳ Migrate handlers one by one

### Phase 2: EventDispatcher ✅ COMPLETE
- ✅ Background worker to publish events
- ✅ Redis Streams integration
- ✅ Event monitoring and health checks
- ⏳ Start dispatcher in main.rs

### Phase 3: Projections ✅ COMPLETE
- ✅ Projection trait and framework
- ✅ ProjectionManager for orchestrating projections
- ✅ Projection checkpoint system with resumability
- ✅ WalletReadModelProjection implementation
- ✅ Redis Streams consumer for projections
- ✅ Fallback to outbox polling (no Redis required)
- ⏳ Start ProjectionManager in main.rs
- ⏳ Query optimization and cache warming

## Using Projections

Projections update read models from domain events, enabling optimized queries on denormalized data.

### Setup ProjectionManager

```rust
use crate::infrastructure::{ProjectionManager, WalletReadModelProjection};

// Create projection manager
let projection_manager = Arc::new(
    ProjectionManager::new(
        Arc::clone(&db_pool),
        redis_url, // Optional - falls back to outbox polling
        "domain_events".to_string(), // Redis Stream name
    )?
    .register(Arc::new(WalletReadModelProjection::new(Arc::clone(&db_pool))))
);

// Start projections
projection_manager.clone().start().await?;
```

### How Projections Work

```
EventDispatcher publishes event
         ↓
Redis Streams (domain_events)
         ↓
ProjectionManager subscribes
         ↓
WalletReadModelProjection processes event
         ├─→ WalletUserCreated → INSERT into wallet_details
         ├─→ WalletUserActivated → UPDATE is_active = true
         ├─→ WalletUserDeactivated → UPDATE is_active = false
         ├─→ WalletPermissionsUpdated → UPDATE permissions arrays
         ├─→ SessionCreated → INCREMENT total_sessions, total_logins
         └─→ SessionInvalidated → DECREMENT active_session_count
         ↓
Checkpoint saved (for resumability)
         ↓
Read model updated (read_model.wallet_details)
```

### Querying Read Models

```rust
// Fast denormalized query (no joins!)
let wallet = sqlx::query!(
    r#"
    SELECT
        wallet_address,
        is_active,
        active_permissions,
        permission_groups,
        total_permissions,
        subscription_tier,
        total_sessions,
        engagement_score
    FROM read_model.wallet_details
    WHERE wallet_address = $1
    "#,
    wallet_address
)
.fetch_one(&pool)
.await?;

// The read model is always eventually consistent
// Updates happen within ~5 seconds of events
```

### Projection Checkpoints

Projections track their progress in `read_model.projection_checkpoints`:

```sql
SELECT * FROM read_model.projection_checkpoints;

projection_name  | last_processed_sequence | events_processed_count | is_healthy
-----------------|-------------------------|------------------------|------------
WalletReadModel  | 12345                   | 12345                  | true
```

**Resumability**: If a projection fails or restarts, it automatically resumes from the last checkpoint.

### Creating Custom Projections

```rust
use crate::infrastructure::cqrs::projection::{Projection, ProjectionEvent, ProjectionCheckpoint};

pub struct MyCustomProjection {
    pool: Arc<PgPool>,
}

#[async_trait]
impl Projection for MyCustomProjection {
    fn projection_name(&self) -> &'static str {
        "MyCustomProjection"
    }

    fn handles_event_types(&self) -> Vec<&'static str> {
        vec!["MyEventType1", "MyEventType2"]
    }

    async fn project_event(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        match event.event_type.as_str() {
            "MyEventType1" => {
                // Update read model
                sqlx::query!(
                    "UPDATE read_model.my_table SET field = $1 WHERE id = $2",
                    event.event_payload["field"].as_str(),
                    event.aggregate_id
                )
                .execute(&mut **tx)
                .await?;
                Ok(())
            }
            _ => Ok(())
        }
    }

    // Implement get_checkpoint and save_checkpoint
    // (same pattern as WalletReadModelProjection)
}

// Register with ProjectionManager
projection_manager.register(Arc::new(MyCustomProjection::new(pool)));
```

### Benefits of Projections

**Query Performance:**
- ✅ No joins required (denormalized data)
- ✅ Pre-computed aggregations (counts, totals, averages)
- ✅ Indexed for fast lookups
- ✅ Read replicas can scale independently

**Eventual Consistency:**
- ✅ Write side doesn't wait for read model updates
- ✅ Fast command execution
- ✅ Projections catch up asynchronously

**Multiple Read Models:**
- ✅ Create specialized views for different queries
- ✅ Same events can project to multiple tables
- ✅ Optimize for specific access patterns

**Resilience:**
- ✅ Checkpoints enable automatic resume
- ✅ Failed projections retry with backoff
- ✅ Health monitoring tracks projection status

## Migration Path - Updated

### Phase 1: TransactionalOutbox ✅ COMPLETE
- ✅ Event store infrastructure created
- ✅ TransactionalOutbox available
- ⏳ Migrate handlers one by one

### Phase 2: EventDispatcher ✅ COMPLETE
- ✅ Background worker to publish events
- ✅ Redis Streams integration
- ✅ Event monitoring and health checks
- ⏳ Start dispatcher in main.rs

### Phase 3: Projections ✅ COMPLETE
- ✅ Projection framework and infrastructure
- ✅ WalletReadModelProjection implementation
- ✅ Checkpoint system for resumability
- ⏳ Start ProjectionManager in main.rs
- ⏳ Add more projections (PermissionSummary, Analytics)

### Phase 4: Production Hardening (Next)
- Command handler migration to TransactionalOutbox
- Integration with existing handlers
- Monitoring dashboards
- Performance optimization
- Projection rebuild tools

## Additional Resources

- See `migrations/027_create_event_store.sql` for database schema
- See `migrations/028_create_read_models.sql` for read model schema
- See `infrastructure/cqrs/event_store.rs` for EventStore implementation
- See `infrastructure/cqrs/outbox.rs` for TransactionalOutbox implementation
- See `infrastructure/cqrs/event_dispatcher.rs` for EventDispatcher implementation
- See `infrastructure/cqrs/projection.rs` for Projection framework
- See `infrastructure/cqrs/projections/wallet_projection.rs` for WalletReadModelProjection example
