# EPSX Database Migration Management

## Current Status: Database Cleanup Completed ✅

The EPSX database has been successfully cleaned up, removing all legacy/unrelated tables while maintaining the core Web3-first authentication architecture.

## Migration History

### What Happened
1. **Initial Consolidation**: Consolidated 26 migration files into unified schema
2. **Schema Repair**: Fixed existing database to match target architecture (002_fix_existing_schema_to_consolidated.sql)
3. **Legacy Cleanup**: Removed unrelated tables and legacy user system (003_remove_unrelated_legacy_tables.sql)

### Latest Cleanup Migration (003_remove_unrelated_legacy_tables.sql)

This migration successfully:
- ✅ **Removed Legacy User System**: users, user_group_memberships, user_permission_cache tables
- ✅ **Removed Diesel Dependencies**: __diesel_schema_migrations table  
- ✅ **Removed Unrelated Features**: device_fingerprints, api_keys, permission_delegations
- ✅ **Removed Enterprise Overhead**: enterprise_teams, enterprise_team_members
- ✅ **Removed Duplicate Tables**: web3_auth_nonces, wallet_migrations
- ✅ **Maintained Core Functionality**: All Web3 tables preserved and functional

## Current Database State (Post-Cleanup)

### Core Web3 Infrastructure ✅
- `wallet_identities` - Primary wallet authentication
- `request_nonces` - Replay attack protection  
- `wallet_auth_log` - Authentication audit trail

### Group-Based Permissions ✅ 
- `permission_groups` - System permission groups
- `wallet_group_memberships` - Wallet membership management
- `web3_group_assignment_rules` - Auto-assignment rules
- `group_assignment_history` - Complete audit trail
- `wallet_permission_cache` - Performance optimization
- `web3_verification_cache` - Blockchain verification caching

### Payment & Billing ✅
- `payment_records` - Web3 payment tracking
- `payment_verification_attempts` - Blockchain verification
- `active_subscriptions` - Subscription management
- `wallet_token_balances` - Token balance tracking  
- `tier_pricing` - Subscription tiers
- `supported_payment_methods` - Payment configuration
- `usage_metrics` - Usage tracking
- `billing_discounts` - Discount management
- `billing_audit_log` - Billing audit trail

### Enterprise Features ✅
- **DAO Governance**: `dao_configurations`, `dao_proposals`, `dao_votes`
- **Marketplace**: `marketplace_products`, `purchase_records`, `professional_services`
- **Compliance**: KYC/AML tables, security monitoring, fraud detection

### Database Statistics (After Cleanup)

**Tables Removed**: 16 legacy/unrelated tables
- `users` (replaced by `wallet_identities`)
- `user_group_memberships` (replaced by `wallet_group_memberships`)
- `user_permission_cache` (replaced by `wallet_permission_cache`)
- `__diesel_schema_migrations` (using SQLx migrations)
- `api_keys`, `api_requests` (using Web3 signatures)
- `device_fingerprints` (not needed for Web3 auth)
- `permission_delegations` (not implemented)
- `user_behavior_profiles` (not part of core system)
- `enterprise_teams`, `enterprise_team_members` (using DAO governance)
- `web3_auth_nonces` (duplicate of `request_nonces`)
- `wallet_migrations` (not needed)
- `client_tier_mappings` (using permission groups)
- `user_subscription_activations` (replaced by Web3 system)
- `enterprise_compliance_cache` (keeping main compliance tables)
- `enterprise_team_members` (using DAO system)

**Current Tables**: 45 core Web3-focused tables
**Current Views**: 7 performance and monitoring views

## Architecture

### Pure Web3-First System ✅
The database now supports **pure Web3 authentication only**:
- **Primary Authentication**: `wallet_identities` table
- **Permission Management**: `wallet_group_memberships` + `permission_groups`
- **Automatic Asset-Based Assignment**: Web3 asset verification rules
- **Complete Audit Trail**: All actions tracked with wallet addresses
- **Performance Optimized**: Caching layers and indexed queries

## Working Features

### ✅ Functional Components
- **Wallet Authentication**: SIWE + EIP712 support
- **Group-Based Permissions**: Auto-expiry, Web3 asset verification
- **Payment Processing**: Multi-token support with verification tracking
- **DAO Governance**: Proposal and voting system
- **Enterprise Features**: Marketplace, compliance, security monitoring
- **Rate Limiting**: Web3 wallet-based rate limiting
- **Security Monitoring**: Fraud detection and risk indicators

### 🔧 Available Utilities
```sql
-- View active wallet groups
SELECT * FROM active_wallet_groups;

-- Check group membership statistics
SELECT * FROM group_membership_stats;

-- Monitor payment verification
SELECT * FROM payment_verification_stats;

-- View enterprise security metrics
SELECT * FROM enterprise_security_metrics;
```

## Development Workflow

### For New Features
1. Create incremental migrations (004_, 005_, etc.)
2. Follow Web3-first architecture patterns
3. Use `wallet_address` instead of `user_id` for all references
4. Include appropriate indexes and constraints
5. Test with actual wallet addresses

### Database Standards
- **Primary Keys**: Use `wallet_address` for user-related data
- **Foreign Keys**: Reference `wallet_identities(wallet_address)`
- **Timestamps**: Use `granted_at` instead of `assigned_at`
- **Audit Fields**: Include `granted_by_wallet` for tracking
- **Permissions**: Use structured "platform:resource:action" format

## Future Migrations

### Recommended Next Steps
1. **Web3 Enhanced Features**: Additional blockchain integrations
2. **Advanced DAO Features**: Enhanced governance mechanisms
3. **Cross-Chain Support**: Multi-chain wallet management
4. **Performance Optimization**: Additional caching layers

### Migration Guidelines
- Always test on development database first
- Use `IF EXISTS` when dropping tables/views
- Handle wallet address references properly
- Document all schema changes thoroughly
- Include verification queries in migrations

---

**Database Cleanup Date**: December 24, 2024  
**Database Status**: ✅ Clean, optimized, and fully Web3-first  
**Tables Count**: 45 core tables (reduced from 61)  
**Next Migration Number**: 004_