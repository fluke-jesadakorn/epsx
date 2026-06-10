# Notification System - Monitoring & Metrics

This document provides comprehensive monitoring strategies for the EPSX notification system.

## Table of Contents

1. [System Health Endpoints](#system-health-endpoints)
2. [Database Query Monitoring](#database-query-monitoring)
3. [SSE Connection Monitoring](#sse-connection-monitoring)
4. [Performance Metrics](#performance-metrics)
5. [Alert Thresholds](#alert-thresholds)
6. [Troubleshooting Guide](#troubleshooting-guide)

---

## System Health Endpoints

### SSE Health Check

**Endpoint:** `GET /api/notifications/health`

**Purpose:** Monitor SSE system health and notification statistics

**Response:**
```json
{
  "status": "healthy" | "degraded",
  "redis_healthy": true,
  "timestamp": "2025-10-14T12:00:00Z",
  "stats": {
    "total": 1234,
    "queued": 45,
    "delivered": 1189,
    "acknowledged": 1150
  }
}
```

**Monitoring Strategy:**
```bash
# Check health every 60 seconds
*/1 * * * * curl -s http://localhost:8080/api/notifications/health | jq '.status'

# Alert if status is "degraded" for more than 5 minutes
# Alert if redis_healthy is false
```

### Notification Statistics

**Endpoint:** `GET /api/admin/notifications/stats`

**Purpose:** Get comprehensive notification metrics

**Response:**
```json
{
  "success": true,
  "data": {
    "total_notifications": 5000,
    "sent_today": 250,
    "sent_this_week": 1500,
    "sent_this_month": 4500,
    "delivery_rate": 0.98,
    "read_rate": 0.75,
    "click_rate": 0.45,
    "by_type": {
      "system": 1000,
      "security": 500,
      "payment": 2000
    },
    "by_priority": {
      "critical": 100,
      "high": 500,
      "normal": 3500,
      "low": 900
    },
    "recent_activity": [
      {
        "timestamp": "2025-10-14T12:00:00Z",
        "action": "notification_sent",
        "count": 45
      }
    ]
  }
}
```

---

## Database Query Monitoring

### Critical Queries to Monitor

#### 1. Fetch Queued Notifications (Most Frequent)

**Query:**
```sql
SELECT id, wallet_address, notification_type, title, message, data, priority, timestamp, expires_at
FROM wallet_notifications
WHERE (wallet_address = $1 OR wallet_address = 'all')
  AND deleted_at IS NULL
  AND created_at > NOW() - INTERVAL '30 days'
  AND (expires_at IS NULL OR expires_at > NOW())
ORDER BY timestamp DESC
LIMIT 100
```

**Performance Target:** < 50ms
**Index Used:** `idx_wallet_notifications_queue_fetch`

**Monitoring:**
```sql
-- Check query performance
EXPLAIN ANALYZE
SELECT id, wallet_address, notification_type, title, message, data, priority, timestamp, expires_at
FROM wallet_notifications
WHERE (wallet_address = '0x123...' OR wallet_address = 'all')
  AND deleted_at IS NULL
  AND created_at > NOW() - INTERVAL '30 days'
  AND (expires_at IS NULL OR expires_at > NOW())
ORDER BY timestamp DESC
LIMIT 100;

-- Expected: Index Scan using idx_wallet_notifications_queue_fetch
-- Expected execution time: < 50ms
```

#### 2. Unread Count Query (Very Frequent)

**Query:**
```sql
SELECT COUNT(*)
FROM wallet_notifications
WHERE (wallet_address = $1 OR wallet_address = 'all')
  AND read_at IS NULL
  AND deleted_at IS NULL
```

**Performance Target:** < 10ms
**Index Used:** `idx_wallet_notifications_unread_count`

#### 3. Statistics Aggregation (Periodic)

**Query:**
```sql
SELECT notification_type, COUNT(*) as count
FROM wallet_notifications
WHERE deleted_at IS NULL
GROUP BY notification_type
```

**Performance Target:** < 100ms
**Index Used:** `idx_wallet_notifications_type_stats`

### Index Health Monitoring

```sql
-- Check index usage
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
  AND tablename = 'wallet_notifications'
ORDER BY idx_scan DESC;

-- Unused indexes (should be reviewed)
SELECT
    schemaname,
    tablename,
    indexname,
    pg_size_pretty(pg_relation_size(indexrelid)) as index_size
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
  AND tablename = 'wallet_notifications'
  AND idx_scan = 0
ORDER BY pg_relation_size(indexrelid) DESC;
```

### Table Size Monitoring

```sql
-- Monitor table growth
SELECT
    pg_size_pretty(pg_total_relation_size('wallet_notifications')) as total_size,
    pg_size_pretty(pg_relation_size('wallet_notifications')) as table_size,
    pg_size_pretty(pg_total_relation_size('wallet_notifications') - pg_relation_size('wallet_notifications')) as index_size;

-- Alert if table size > 10GB or index size > table size
```

---

## SSE Connection Monitoring

### Connection Metrics

**Log Patterns to Monitor:**

```bash
# Successful connections
grep "SSE connection opened" /var/log/backend.log | wc -l

# Connection errors
grep "SSE connection error" /var/log/backend.log | wc -l

# Redis pub/sub issues
grep "Failed to get Redis message payload" /var/log/backend.log

# Connection rate (connections per minute)
grep "SSE connection request" /var/log/backend.log | grep "$(date '+%Y-%m-%d %H:%M')" | wc -l
```

### Client-Side Metrics (Frontend Logs)

**Browser Console Patterns:**

```javascript
// Log all SSE events
console.log('🔌 SSE: Initiating connection #123')
console.log('✅ SSE: Connection #123 established')
console.log('🔴 Connection is CLOSED, triggering cleanup')
console.log('🔄 SSE: Scheduling reconnect attempt 1 in 5000ms')
```

**Metrics to Track:**
- Connection establishment time
- Number of reconnection attempts
- Stale callback detections
- Memory leak indicators (increasing connection IDs without cleanup)

### Redis Monitoring

```bash
# Check Redis connection
redis-cli ping

# Monitor pub/sub channels
redis-cli pubsub channels "wallet:*"
redis-cli pubsub numsub "wallet:0x123..."

# Monitor memory usage
redis-cli info memory | grep used_memory_human

# Check for slow operations
redis-cli slowlog get 10
```

---

## Performance Metrics

### Response Time Targets

| Endpoint                          | Target  | Alert Threshold |
|-----------------------------------|---------|-----------------|
| GET /api/notifications            | < 100ms | > 500ms         |
| GET /api/notifications/unread-count | < 50ms  | > 200ms         |
| PUT /api/notifications/:id/read   | < 50ms  | > 200ms         |
| POST /api/admin/notifications/send | < 100ms | > 500ms         |
| GET /api/notifications/stream     | < 50ms  | > 200ms         |

### Database Performance

```sql
-- Find slow queries (> 1 second)
SELECT
    query,
    calls,
    mean_exec_time,
    max_exec_time,
    stddev_exec_time
FROM pg_stat_statements
WHERE query LIKE '%wallet_notifications%'
  AND mean_exec_time > 1000
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Check for table bloat
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size,
    n_dead_tup,
    n_live_tup,
    round(n_dead_tup * 100.0 / NULLIF(n_live_tup + n_dead_tup, 0), 2) AS dead_tuple_percent
FROM pg_stat_user_tables
WHERE tablename = 'wallet_notifications';

-- Alert if dead_tuple_percent > 10%
```

### Memory Monitoring

**Frontend (Browser):**
```javascript
// Check for memory leaks
if (performance.memory) {
    console.log('Used JS Heap:', performance.memory.usedJSHeapSize / 1048576, 'MB')
    console.log('Total JS Heap:', performance.memory.totalJSHeapSize / 1048576, 'MB')
}

// Monitor SSE connections
console.log('Active SSE connections:', document.querySelectorAll('[data-sse-connected]').length)
```

**Backend (Rust):**
```bash
# Memory usage
ps aux | grep backend | awk '{print $4, $6}'

# Connection count
netstat -an | grep :8080 | grep ESTABLISHED | wc -l
```

---

## Alert Thresholds

### Critical Alerts (Immediate Action Required)

1. **SSE Health Degraded > 5 minutes**
   - **Condition:** `status === "degraded"` for > 5 minutes
   - **Action:** Check Redis connection, restart Redis if needed

2. **Database Query > 1 second**
   - **Condition:** Any notification query takes > 1 second
   - **Action:** Check indexes, run ANALYZE, check for bloat

3. **Notification Delivery Failure Rate > 5%**
   - **Condition:** `(total - delivered) / total > 0.05`
   - **Action:** Check SSE connections, Redis health, database connections

4. **Memory Leak Detected**
   - **Condition:** Browser memory growth > 100MB per hour
   - **Action:** Check for unclosed SSE connections, review event listeners

### Warning Alerts (Monitor Closely)

1. **High Unread Count (> 100 per user)**
   - **Condition:** `unread_count > 100` for any user
   - **Action:** Investigate user engagement, review notification frequency

2. **Slow Queries (> 200ms)**
   - **Condition:** Queries taking > 200ms but < 1 second
   - **Action:** Review indexes, consider optimization

3. **High Reconnection Rate**
   - **Condition:** > 10 reconnections per user per hour
   - **Action:** Check network stability, review SSE error patterns

4. **Database Table Size Growing Rapidly**
   - **Condition:** Growth > 1GB per day
   - **Action:** Review cleanup job, check for notification spam

---

## Troubleshooting Guide

### Issue: SSE Connections Failing

**Symptoms:**
- Users not receiving real-time notifications
- "SSE connection error" in logs
- High reconnection attempts

**Diagnosis:**
```bash
# Check backend logs
tail -f /var/log/backend.log | grep SSE

# Check Redis
redis-cli ping
redis-cli pubsub channels "wallet:*"

# Check network
curl -v http://localhost:8080/api/notifications/stream?token=test

# Check browser console
# Look for: "❌ SSE connection error"
```

**Solutions:**
1. Verify Redis is running: `systemctl status redis`
2. Check CORS configuration includes SSE headers
3. Verify JWT token is valid
4. Check firewall rules for SSE port
5. Review backend error logs for specific error messages

### Issue: Slow Notification Queries

**Symptoms:**
- Notification page takes > 5 seconds to load
- High database CPU usage
- Slow API responses

**Diagnosis:**
```sql
-- Check for missing indexes
SELECT * FROM pg_stat_user_indexes
WHERE schemaname = 'public'
  AND tablename = 'wallet_notifications'
  AND idx_scan = 0;

-- Check for bloat
SELECT pg_size_pretty(pg_total_relation_size('wallet_notifications'));

-- Check for slow queries
SELECT query, mean_exec_time
FROM pg_stat_statements
WHERE query LIKE '%wallet_notifications%'
ORDER BY mean_exec_time DESC;
```

**Solutions:**
1. Run `VACUUM ANALYZE wallet_notifications;`
2. Check all 14 indexes are created (see migration 001)
3. Review query plans with `EXPLAIN ANALYZE`
4. Consider increasing database resources
5. Check for table bloat and run VACUUM FULL if needed

### Issue: Memory Leaks in Frontend

**Symptoms:**
- Browser becoming slow over time
- High memory usage in DevTools
- Multiple SSE connections visible

**Diagnosis:**
```javascript
// Check active event listeners
getEventListeners(document)

// Check SSE connections
console.log('SSE connection state:', connectionStateRef.current)
console.log('Connection ID:', connectionIdRef.current)

// Memory profiling
performance.memory.usedJSHeapSize / 1048576 // MB
```

**Solutions:**
1. Verify `cleanupSSEListeners()` is called on disconnect
2. Check for stale callbacks using connection ID
3. Ensure proper cleanup in useEffect return
4. Review browser DevTools memory profiler
5. Test with Chrome DevTools > Memory > Take heap snapshot

### Issue: Notifications Not Persisting

**Symptoms:**
- Notifications disappear on refresh
- Offline queue empty
- Users missing notification history

**Diagnosis:**
```sql
-- Check if notifications are being saved
SELECT COUNT(*) FROM wallet_notifications
WHERE created_at > NOW() - INTERVAL '1 hour';

-- Check for soft-deletes
SELECT COUNT(*) FROM wallet_notifications
WHERE deleted_at IS NOT NULL;

-- Check expiry settings
SELECT COUNT(*) FROM wallet_notifications
WHERE expires_at IS NOT NULL
  AND expires_at < NOW();
```

**Solutions:**
1. Verify database INSERT operations succeed
2. Check cleanup job isn't running too frequently
3. Review notification expiry settings
4. Check soft-delete logic in handlers
5. Verify fetch_queued_notifications filters are correct

---

## Monitoring Dashboard Queries

### Key Metrics Dashboard

```sql
-- Real-time metrics (refresh every 30 seconds)
SELECT
    COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '5 minutes') as notifications_last_5min,
    COUNT(*) FILTER (WHERE delivered_at IS NOT NULL) as total_delivered,
    COUNT(*) FILTER (WHERE read_at IS NOT NULL) as total_read,
    COUNT(*) FILTER (WHERE clicked_at IS NOT NULL) as total_clicked,
    COUNT(*) FILTER (WHERE deleted_at IS NULL AND read_at IS NULL) as current_unread,
    AVG(EXTRACT(EPOCH FROM (delivered_at - created_at))) FILTER (WHERE delivered_at IS NOT NULL) as avg_delivery_time_seconds
FROM wallet_notifications
WHERE created_at > NOW() - INTERVAL '24 hours';

-- Engagement metrics
SELECT
    notification_type,
    COUNT(*) as sent,
    COUNT(*) FILTER (WHERE read_at IS NOT NULL) as read,
    COUNT(*) FILTER (WHERE clicked_at IS NOT NULL) as clicked,
    ROUND(COUNT(*) FILTER (WHERE read_at IS NOT NULL) * 100.0 / COUNT(*), 2) as read_rate_percent,
    ROUND(COUNT(*) FILTER (WHERE clicked_at IS NOT NULL) * 100.0 / COUNT(*), 2) as click_rate_percent
FROM wallet_notifications
WHERE created_at > NOW() - INTERVAL '7 days'
  AND deleted_at IS NULL
GROUP BY notification_type
ORDER BY sent DESC;

-- Performance metrics
SELECT
    DATE_TRUNC('hour', created_at) as hour,
    COUNT(*) as created,
    COUNT(*) FILTER (WHERE delivered_at IS NOT NULL) as delivered,
    AVG(EXTRACT(EPOCH FROM (delivered_at - created_at))) as avg_delivery_seconds
FROM wallet_notifications
WHERE created_at > NOW() - INTERVAL '24 hours'
GROUP BY hour
ORDER BY hour DESC;
```

---

## Automated Monitoring Script

```bash
#!/bin/bash
# notification-monitor.sh
# Run every 5 minutes via cron

ALERT_EMAIL="alerts@epsx.com"
BACKEND_URL="http://localhost:8080"

# Check SSE health
health_status=$(curl -s "${BACKEND_URL}/api/notifications/health" | jq -r '.status')
if [ "$health_status" != "healthy" ]; then
    echo "ALERT: Notification system health degraded: $health_status" | mail -s "EPSX Notification Alert" $ALERT_EMAIL
fi

# Check Redis
redis_health=$(redis-cli ping 2>/dev/null)
if [ "$redis_health" != "PONG" ]; then
    echo "ALERT: Redis is down" | mail -s "EPSX Redis Alert" $ALERT_EMAIL
fi

# Check database performance
slow_queries=$(psql -t -c "SELECT COUNT(*) FROM pg_stat_statements WHERE query LIKE '%wallet_notifications%' AND mean_exec_time > 1000")
if [ "$slow_queries" -gt "0" ]; then
    echo "ALERT: $slow_queries slow notification queries detected" | mail -s "EPSX Database Alert" $ALERT_EMAIL
fi

# Check table size
table_size=$(psql -t -c "SELECT pg_total_relation_size('wallet_notifications')")
if [ "$table_size" -gt "10737418240" ]; then # 10GB
    echo "ALERT: Notification table size exceeds 10GB" | mail -s "EPSX Database Size Alert" $ALERT_EMAIL
fi
```

---

Last Updated: 2025-10-14
