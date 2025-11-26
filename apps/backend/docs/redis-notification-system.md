# Redis Notification System Implementation

## Overview

Complete Redis pub/sub notification system replacing tokio broadcast channels. Enables horizontal scaling with multi-instance support and offline message queuing.

**Status:** ✅ Implementation Complete (Phases 1-4)
**Date:** 2025-10-13
**Breaking Change:** Complete replacement, no backward compatibility

## Architecture

### Flow

```
Admin → Backend API → Redis Pub/Sub → SSE Stream → Frontend
                    ↓
                 Database (offline queue + tracking)
```

### Channels

- `notifications:wallet:{address}` - Wallet-specific notifications
- `notifications:all` - Broadcast to all users

### Components

**Backend:**
- `RedisNotificationBroadcaster` - Pub/sub publisher
- `offline_queue` - Database persistence layer
- `sse_handlers` - SSE streaming endpoint

**Frontend:**
- `NotificationsAPIClient` - Auto-acknowledgment on receipt

## Database Schema

### Migration 035: Delivery Tracking

```sql
ALTER TABLE wallet_notifications ADD COLUMN:
- queued_at TIMESTAMP
- delivery_attempts INTEGER
- last_delivery_attempt_at TIMESTAMP
- delivery_error TEXT
- acknowledged_at TIMESTAMP
```

### Migration 036: Connection Tracking

```sql
CREATE TABLE notification_subscriptions:
- id UUID
- wallet_address VARCHAR(255)
- instance_id VARCHAR(255)
- connection_id VARCHAR(255)
- redis_channel VARCHAR(255)
- connected_at TIMESTAMP
- last_ping_at TIMESTAMP
```

## Key Files

### Infrastructure

- `/apps/backend/src/infrastructure/redis/pool.rs` - Redis connection pool
- `/apps/backend/src/infrastructure/redis/mod.rs` - Module exports

### Notification System

- `/apps/backend/src/web/notifications/redis_broadcaster.rs` - Redis pub/sub
- `/apps/backend/src/web/notifications/offline_queue.rs` - Database persistence
- `/apps/backend/src/web/notifications/sse_handlers.rs` - SSE endpoint (complete rewrite)
- `/apps/backend/src/web/notifications/mod.rs` - Exports (removed old broadcaster)

### Admin Handlers

- `/apps/backend/src/web/admin/notification_handlers.rs` - Send + acknowledgment handlers
- `/apps/backend/src/web/admin/routes.rs` - Route registration

### Container Updates

- `/apps/backend/src/infrastructure/container/simple_container.rs` - Redis initialization
- `/apps/backend/src/infrastructure/container/stateless_service_factory.rs` - Serverless support
- `/apps/backend/src/web/auth/app_state.rs` - Redis in app state
- `/apps/backend/src/web/routes/unified_router.rs` - Route builder updates
- `/apps/backend/src/main.rs` - Async container init

### Frontend

- `/shared/api/notifications.ts` - API client with auto-acknowledgment

## API Endpoints

### SSE Connection

```
GET /api/notifications/stream
Query: types (csv), timeout (seconds), token (bearer)
Response: Server-Sent Events stream
```

### Send Notification

```
POST /api/notifications/send
Body: {
  wallet_address: string | "all"
  notification_type: string
  title: string
  message: string
  data?: object
  priority?: string
  expires_at?: timestamp
}
```

### Acknowledge Notification

```
PUT /api/notifications/{id}/acknowledge
Response: { success: bool, message: string }
```

### Statistics

```
GET /api/notifications/stats
Response: {
  total: number
  queued: number
  delivered: number
  acknowledged: number
}
```

## Dependencies Added

```toml
redis = { version = "0.24", features = ["tokio-comp", "connection-manager", "streams"] }
async-stream = "0.3"
```

## SSE Flow

1. Client connects to `/api/notifications/stream`
2. Backend extracts wallet from Bearer token
3. Fetch queued notifications from database
4. Subscribe to Redis channels (wallet + broadcast)
5. Send queued notifications first
6. Stream real-time from Redis pub/sub
7. Auto-acknowledge on frontend receipt

## Offline Queue

**Storage:** All notifications saved to PostgreSQL
**Delivery:** Queued notifications sent first on reconnect
**Expiry:** Automatic filtering of expired notifications
**Limit:** 100 notifications per fetch

## Tracking

- `queued_at` - Notification created
- `delivered_at` - Sent via SSE
- `acknowledged_at` - Confirmed by client
- `delivery_attempts` - Retry counter
- `last_delivery_attempt_at` - Last attempt timestamp

## Health Check

```
GET /api/notifications/health
Response: {
  status: "healthy" | "degraded"
  redis_healthy: bool
  stats: { ... }
}
```

## Environment Variables

```bash
REDIS_URL=redis://localhost:6379  # Required for notifications
```

## Migration Steps

**Before deployment:**

```bash
# Using helper script (recommended - loads .env automatically)
cd apps/backend
./scripts/migrate.sh

# Or manually with sqlx
cd apps/backend
export DATABASE_URL="postgresql://user:pass@host:5432/db"
sqlx migrate run

# Or direct SQL
psql $DATABASE_URL -f migrations/035_add_notification_delivery_tracking.sql
psql $DATABASE_URL -f migrations/036_create_notification_subscriptions.sql
```

**Compilation:**

```bash
# With database access
cargo build

# Without database (offline mode)
SQLX_OFFLINE=true cargo build
```

## Helper Scripts

The backend includes helper scripts that automatically load environment variables from `.env`:

```bash
# Check environment configuration
./scripts/check-env.sh

# Run migrations with .env loaded
./scripts/migrate.sh

# Start backend with .env loaded
./scripts/run.sh
```

See `scripts/README.md` for detailed documentation.

## Scalability

**Old system:**
- Single instance only (tokio broadcast)
- No offline support
- In-memory state loss on restart

**New system:**
- Horizontal scaling via Redis pub/sub
- Multi-instance support
- Persistent offline queue
- Delivery tracking and analytics

## Performance

- Keep-alive: 15 seconds
- Redis pub/sub: Sub-millisecond latency
- Batch fetch: 100 notifications limit
- Auto-delivery: Background async marking

## Security

- Bearer token authentication required
- Wallet-specific channels (isolation)
- Token extraction from JWT or legacy format
- Optional type filtering

## Monitoring

```rust
// Get statistics
let stats = get_notification_stats(&db_pool).await?;

// Check subscriber count
let count = redis_broadcaster.get_subscriber_count(wallet).await?;

// Cleanup old notifications
cleanup_old_notifications(&db_pool, 30).await?; // 30 days
```

## Frontend Integration

```typescript
// Connect to SSE
const disconnect = notificationClient.connectToSSE(
  { types: "security,payment" },
  (notification) => {
    console.log("Received:", notification);
    // Auto-acknowledged in background
  },
  (error) => console.error("SSE error:", error),
  () => console.log("SSE connected")
);

// Manual acknowledgment (if needed)
await notificationClient.acknowledgeNotification(notificationId);

// Cleanup
disconnect();
```

## Testing

**Prerequisites:** Redis and PostgreSQL must be running

```bash
# Start Redis
redis-server

# Run migrations
sqlx migrate run

# Test SSE connection
curl -N -H "Authorization: Bearer {token}" \
  http://localhost:8080/api/notifications/stream

# Send test notification
curl -X POST http://localhost:8080/api/notifications/send \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_address": "0x123...",
    "notification_type": "general",
    "title": "Test",
    "message": "Hello",
    "priority": "normal"
  }'
```

## Troubleshooting

**Redis connection failed:**
- Check `REDIS_URL` environment variable
- Ensure Redis server is running
- Check network connectivity

**Notifications not delivered:**
- Verify Redis pub/sub subscription
- Check wallet address format (lowercase)
- Review delivery tracking in database

**SSE connection drops:**
- Check keep-alive interval (15s)
- Verify Bearer token validity
- Review proxy/load balancer timeouts

**Database errors:**
- Apply migrations: `sqlx migrate run`
- Or use: `SQLX_OFFLINE=true` for compilation
- Check column existence in wallet_notifications table

## Removed Components

- ❌ `NotificationBroadcaster` (tokio broadcast)
- ❌ `InMemoryNotificationStore`
- ❌ `apps/backend/src/web/admin/notification_utils.rs`
- ❌ All tokio broadcast channel code

## Next Steps (Optional)

- [ ] Add notification templates
- [ ] Implement retry logic for failed deliveries
- [ ] Add rate limiting per wallet
- [ ] Create admin dashboard for monitoring
- [ ] Add notification preferences per user
- [ ] Implement notification categories
- [ ] Add push notification support (FCM/APNS)

---

**Implementation Complete:** All phases 1-4 finished, cleaned, and documented.
