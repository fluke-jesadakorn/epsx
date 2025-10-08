# Database Cleanup - Quick Start Guide

**⚡ 5-Minute Quick Start for Experienced Users**

---

## TL;DR

Remove legacy tables from pre-Web3 migration safely.

### Prerequisites
✅ Backend working
✅ Web3 auth working
✅ CQRS handlers migrated

---

## 🚀 Quick Steps

### 1. Backup (REQUIRED)
```bash
pg_dump $DATABASE_URL > backup_$(date +%Y%m%d).sql
```

### 2. Inspect
```bash
cd apps/backend
psql $DATABASE_URL -f migrations/INSPECT_DATABASE.sql
```

### 3. Cleanup
```bash
psql $DATABASE_URL -f migrations/029_cleanup_unused_tables.sql
```

### 4. Verify
```bash
# Test backend
cargo run

# Test endpoints
curl http://localhost:8080/health
curl http://localhost:8080/api/v1/public/plans
```

### 5. Monitor
Wait 7-14 days, then:
```sql
DROP SCHEMA archive CASCADE;
```

---

## 🔄 Rollback

```sql
-- Restore one table
ALTER TABLE archive.users SET SCHEMA public;

-- Or full restore
psql $DATABASE_URL < backup_<date>.sql
```

---

## 📋 What Gets Archived

### Legacy Auth
- `users`, `oidc_users`, `user_sessions`
- `email_verification`, `password_resets`

### Legacy Permissions
- `admin_modules`, `user_permissions`, `role_permissions`

### Legacy Tiers
- `user_tiers`, `tier_permissions`, `tier_limits`

### Backups
- `backup_*`, `*_backup`, `*_old`

---

## ✅ Active Tables (Protected)

20 tables remain active:
- `wallet_users`, `permissions`, `permission_groups`
- `sessions`, `web3_auth_nonces`
- `event_store`, `outbox_events` (CQRS)
- `read_model.*` (4 tables)
- and 10 more...

---

## 📚 Full Documentation

Read `CLEANUP_README.md` for complete guide.

---

## ⚠️ Critical Warnings

❌ **DO NOT** skip database backup
❌ **DO NOT** drop archive immediately
❌ **DO NOT** run in production without testing
✅ **DO** monitor application after cleanup
✅ **DO** wait 7-14 days before final deletion

---

**Safe, reversible, well-tested** ✨
