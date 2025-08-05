# EPSX Casbin Authorization Migration Guide

## Overview

This guide covers the complete migration from the legacy EPSX authorization system to the new Casbin-based policy enforcement system. The migration provides enhanced security, better performance, and more flexible permission management.

## Table of Contents

1. [Migration Benefits](#migration-benefits)
2. [Pre-Migration Checklist](#pre-migration-checklist)
3. [Step-by-Step Migration](#step-by-step-migration)
4. [API Changes](#api-changes)
5. [Configuration Updates](#configuration-updates)
6. [Testing and Validation](#testing-and-validation)
7. [Rollback Procedures](#rollback-procedures)
8. [Troubleshooting](#troubleshooting)

## Migration Benefits

### Enhanced Security
- **Policy-Based Access Control (PBAC)**: Fine-grained permissions with flexible policy definitions
- **Role-Based Access Control (RBAC)**: Hierarchical role inheritance (admin → moderator → premium_user → basic_user)
- **Wildcard Pattern Matching**: Flexible resource and action matching with `*` patterns
- **Audit Trail**: Comprehensive logging of all authorization decisions

### Improved Performance
- **Policy Caching**: 5-minute TTL with LRU eviction for sub-millisecond response times
- **Database Persistence**: SQLx adapter for reliable policy storage
- **Batch Operations**: Efficient bulk policy management
- **Circuit Breaker**: Automatic failover protection

### Better Management
- **Admin Interface**: 12 comprehensive admin endpoints for policy management
- **Dynamic Updates**: Real-time policy changes without service restart
- **Health Monitoring**: Detailed system health and performance metrics
- **Error Handling**: Advanced error recovery and retry mechanisms

## Pre-Migration Checklist

### Infrastructure Requirements
- [ ] PostgreSQL 12+ database with migration capabilities
- [ ] Redis instance for session and cache storage (optional but recommended)
- [ ] Application server with Rust 1.75+ support
- [ ] Monitoring system for observability (Prometheus/Grafana recommended)

### Data Backup
- [ ] **CRITICAL**: Full database backup of existing permissions and user roles
- [ ] Export current IAM profiles and mappings
- [ ] Document existing API endpoints and their permission requirements
- [ ] Create rollback scripts for emergency recovery

### Environment Preparation
- [ ] Set up staging environment identical to production
- [ ] Configure environment variables (see [Configuration Updates](#configuration-updates))
- [ ] Deploy Casbin migration scripts
- [ ] Verify database connectivity and permissions

## Step-by-Step Migration

### Phase 1: Database Setup (Estimated: 30 minutes)

```bash
# 1. Apply Casbin database migration
sqlx migrate run --database-url $DATABASE_URL

# 2. Verify migration success
sqlx migrate info --database-url $DATABASE_URL

# 3. Load initial RBAC policies
psql $DATABASE_URL -f scripts/casbin_initial_policies.sql
```

**Expected Output:**
```
Applied 1 migration: 20241205_create_casbin_policy_tables
✅ Migration successful - 27 initial policies loaded
```

### Phase 2: Service Deployment (Estimated: 15 minutes)

```bash
# 1. Deploy new backend with Casbin integration
docker-compose -f docker-compose.production.yml up -d epsx-backend

# 2. Verify service health
curl http://localhost:8080/api/v1/health/readiness

# 3. Check Casbin functionality
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
     http://localhost:8080/api/v1/admin/casbin/policies
```

**Expected Response:**
```json
{
  "status": "success",
  "data": {
    "total_policies": 27,
    "total_role_inheritances": 4
  }
}
```

### Phase 3: Data Migration (Estimated: 45 minutes)

```sql
-- 1. Migrate existing user roles
INSERT INTO casbin_rule (ptype, v0, v1)
SELECT 'g', user_id, 
  CASE 
    WHEN profile_name = 'admin-full-004' THEN 'admin'
    WHEN profile_name = 'moderator-standard-003' THEN 'moderator'
    WHEN profile_name = 'user-premium-002' THEN 'premium_user'
    ELSE 'basic_user'
  END as role
FROM users u
JOIN user_profiles up ON u.profile_id = up.id;

-- 2. Migrate custom permissions (if any)
INSERT INTO casbin_rule (ptype, v0, v1, v2)
SELECT 'p', user_id, resource_path, action_type
FROM legacy_permissions lp
JOIN users u ON lp.user_id = u.id;

-- 3. Verify migration
SELECT ptype, COUNT(*) FROM casbin_rule GROUP BY ptype;
```

### Phase 4: API Endpoint Updates (Estimated: 20 minutes)

#### Frontend API Calls to Update

**Before (Legacy):**
```javascript
// Old permission check
const hasPermission = await fetch('/api/v1/auth/check-permission', {
  method: 'POST',
  body: JSON.stringify({
    userId: user.id,
    resource: '/admin/users',
    action: 'read'
  })
});
```

**After (Casbin):**
```javascript
// New Casbin permission check
const hasPermission = await fetch(`/api/v1/iam/check-permission/${user.id}/admin/users/read`);

// Or use the evaluation endpoint for complex checks
const evaluation = await fetch('/api/v1/iam/evaluate-permission', {
  method: 'POST',
  body: JSON.stringify({
    user_id: user.id,
    resource: '/admin/users',
    action: 'read'
  })
});
```

### Phase 5: Configuration Updates

#### Environment Variables

Add these to your `.env` file:

```bash
# Casbin Configuration
CASBIN_MODEL_PATH=./casbin_model.conf
CASBIN_CACHE_TTL_SECONDS=300
CASBIN_CACHE_MAX_ENTRIES=10000

# Health Check Configuration
HEALTH_CHECK_INTERVAL=30
METRICS_ENABLED=true

# Error Handling Configuration
CIRCUIT_BREAKER_THRESHOLD=5
RETRY_MAX_ATTEMPTS=3
RETRY_BASE_DELAY_MS=100
```

#### Application Configuration

Update your application startup to include Casbin service:

```rust
// Add to main.rs or app initialization
let casbin_service = Arc::new(
    CasbinService::new(db_pool.clone())
        .await
        .expect("Failed to initialize Casbin service")
);

let app_state = AppState {
    casbin_service,
    // ... other services
};
```

## API Changes

### Deprecated Endpoints (Remove After Migration)

| Old Endpoint | New Endpoint | Notes |
|-------------|-------------|-------|
| `POST /api/v1/auth/check-permission` | `GET /api/v1/iam/check-permission/{user}/{resource}/{action}` | Path parameters instead of body |
| `POST /api/v1/admin/assign-profile` | `POST /api/v1/iam/assign-role/{user}/{role}` | Role-based instead of profile-based |
| `GET /api/v1/admin/user-permissions` | `GET /api/v1/admin/casbin/user-permissions/{user_id}` | Enhanced with role inheritance |

### New Endpoints Available

| Endpoint | Purpose | Example |
|----------|---------|---------|
| `GET /api/v1/health/health-check` | Comprehensive health monitoring | System status and metrics |
| `POST /api/v1/admin/casbin/batch-policies` | Bulk policy management | Add multiple policies at once |
| `GET /api/v1/admin/casbin/cache-stats` | Cache performance monitoring | Hit ratios and utilization |
| `POST /api/v1/admin/casbin/reload-policies` | Dynamic policy refresh | Reload from database |

## Testing and Validation

### Automated Testing

```bash
# Run integration tests
cargo test --test casbin_integration_tests

# Run load tests (optional)
cargo test --test casbin_load_tests --release

# Run benchmark tests (optional)
cargo bench --bench casbin_performance_benchmarks
```

### Manual Validation Checklist

#### Basic Functionality
- [ ] Admin users can access all endpoints
- [ ] Basic users can only access trading endpoints
- [ ] Premium users can access analytics features
- [ ] Unauthorized access is properly denied

#### Role Inheritance
- [ ] Admin inherits all permissions from lower roles
- [ ] Moderator inherits premium and basic permissions
- [ ] Premium users inherit basic permissions
- [ ] Role changes take effect immediately

#### Performance
- [ ] Policy enforcement responds within 10ms (cached)
- [ ] Cache hit ratio above 80% under normal load
- [ ] System handles 1000+ concurrent requests
- [ ] Database queries complete within 100ms

#### Error Handling
- [ ] Circuit breaker activates under high error rates
- [ ] Graceful degradation when database is unavailable
- [ ] Comprehensive error logging for debugging
- [ ] Health checks report accurate system status

### Load Testing Commands

```bash
# Basic load test (100 users, 60 seconds)
cargo test test_normal_load --release -- --ignored

# High load test (500 users, 2 minutes)  
cargo test test_high_load --release -- --ignored

# Stress test (find breaking point)
cargo test test_stress_limits --release -- --ignored
```

## Rollback Procedures

### Emergency Rollback (< 5 minutes)

If critical issues arise, follow this procedure:

```bash
# 1. Switch back to legacy authorization service
kubectl rollout undo deployment/epsx-backend

# 2. Restore database if needed
pg_restore --clean --if-exists -d $DATABASE_URL backup_pre_migration.sql

# 3. Verify service health
curl http://localhost:8080/api/v1/health

# 4. Notify team and users
echo "Migration rollback completed - investigating issues"
```

### Planned Rollback (30 minutes)

For planned rollbacks with data preservation:

```sql
-- 1. Export Casbin policies for future use
COPY (SELECT * FROM casbin_rule) TO '/tmp/casbin_policies_backup.csv' WITH CSV HEADER;

-- 2. Restore legacy permission tables
DROP TABLE IF EXISTS casbin_rule;
-- Restore legacy tables from backup

-- 3. Update application configuration
-- Remove Casbin service initialization
-- Restore legacy authorization middleware
```

## Troubleshooting

### Common Issues and Solutions

#### Database Connection Errors
**Symptom:** `Failed to connect to PostgreSQL database`
**Solution:** 
1. Verify DATABASE_URL environment variable
2. Check database service status: `systemctl status postgresql`
3. Test connectivity: `psql $DATABASE_URL -c "SELECT 1;"`

#### Policy Evaluation Failures
**Symptom:** All permissions denied, 500 errors on enforcement
**Solution:**
1. Check policy count: `SELECT COUNT(*) FROM casbin_rule;`
2. Verify model file: `cat casbin_model.conf`
3. Reload policies: `curl -X POST /api/v1/admin/casbin/reload-policies`

#### Cache Performance Issues
**Symptom:** Slow response times, high database load
**Solution:**
1. Check cache stats: `curl /api/v1/admin/casbin/cache-stats`
2. Increase cache size in configuration
3. Monitor hit ratios and adjust TTL

#### Circuit Breaker Activation
**Symptom:** `Service temporarily unavailable` errors
**Solution:**
1. Check error rates: `curl /api/v1/health/metrics`
2. Reset circuit breaker: `curl -X POST /api/v1/admin/casbin/reset-circuit-breaker`
3. Investigate root cause of failures

### Debug Commands

```bash
# Check system health
curl -s http://localhost:8080/api/v1/health/health-check | jq '.'

# View policy details
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
     http://localhost:8080/api/v1/admin/casbin/policies | jq '.'

# Test specific permission
curl http://localhost:8080/api/v1/iam/check-permission/test_user/api/v1/trading/GET

# View cache performance
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
     http://localhost:8080/api/v1/admin/casbin/cache-stats | jq '.'

# Get diagnostic information
curl http://localhost:8080/api/v1/health/diagnostic | jq '.'
```

### Log Analysis

Important log patterns to monitor:

```bash
# Authorization successes
grep "Permission granted" /var/log/epsx/application.log

# Authorization failures  
grep "Permission denied" /var/log/epsx/application.log

# Policy changes
grep "Policy added\|Policy removed" /var/log/epsx/application.log

# Performance issues
grep "response_time_ms.*[5-9][0-9][0-9]" /var/log/epsx/application.log
```

## Post-Migration Tasks

### Monitoring Setup

1. **Set up Grafana dashboards** for Casbin metrics
2. **Configure alerts** for high error rates, slow responses
3. **Monitor cache hit ratios** and adjust configuration as needed
4. **Track policy changes** for audit compliance

### Performance Optimization

1. **Analyze usage patterns** and optimize cache settings
2. **Review policy complexity** and simplify where possible
3. **Monitor database performance** and add indexes if needed
4. **Consider read replicas** for high-traffic deployments

### Documentation Updates

1. **Update API documentation** with new endpoints
2. **Create runbooks** for common operations
3. **Document troubleshooting procedures** for operations team
4. **Update security documentation** with new RBAC model

## Support and Resources

### Documentation
- [Casbin Official Documentation](https://casbin.org/docs/en/overview)
- [EPSX Casbin API Reference](./API_REFERENCE.md)
- [Performance Tuning Guide](./PERFORMANCE_TUNING.md)

### Emergency Contacts
- Backend Team Lead: [Contact Information]
- DevOps Team: [Contact Information]  
- Security Team: [Contact Information]

### Useful Commands Reference

```bash
# Service Management
systemctl status epsx-backend
journalctl -u epsx-backend -f

# Database Operations
psql $DATABASE_URL
SELECT * FROM casbin_rule LIMIT 10;

# Performance Monitoring
htop
iostat -x 1
netstat -tuln | grep 8080

# Docker Operations
docker-compose logs -f epsx-backend
docker exec -it epsx-backend bash
```

---

**Migration Timeline:** 2-3 hours for production deployment
**Rollback Window:** 24 hours with full support  
**Documentation Version:** 1.0
**Last Updated:** December 2024