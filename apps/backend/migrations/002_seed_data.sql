-- EPSX Production Seed Data
-- Production seed data with admin user and sample analytics
-- Run after the main schema migration 001_initial_schema.sql

-- ============================================================================
-- PRODUCTION SEED DATA
-- ============================================================================

-- Insert production admin user (production deployment)
INSERT INTO users (firebase_uid, email, display_name, name, package_tier, permissions, is_active) VALUES
('prod_admin_uid_epsx_2024', 'info@epsx.io', 'Production Admin', 'EPSX Administrator', 'PREMIUM', ARRAY['*'], true)
ON CONFLICT (firebase_uid) DO NOTHING;

-- Grant system admin access to production admin
INSERT INTO user_admin_roles (firebase_uid, module_code, granted_by, granted_reason, is_active) VALUES
('prod_admin_uid_epsx_2024', 'system_admin', 'prod_admin_uid_epsx_2024', 'Production system administrator', true)
ON CONFLICT (firebase_uid, module_code) DO NOTHING;

-- Insert sample EPS analytics data for testing
INSERT INTO eps_growth_analytics (symbol, name, country, sector, exchange, current_eps, qoq_growth_rate, price_current, market_cap, volume, ranking_score) VALUES
('AAPL', 'Apple Inc.', 'US', 'Technology', 'NASDAQ', 6.15, 8.50, 175.25, 2800000000000, 45000000, 95.8),
('MSFT', 'Microsoft Corporation', 'US', 'Technology', 'NASDAQ', 9.65, 12.30, 335.50, 2500000000000, 28000000, 94.2),
('GOOGL', 'Alphabet Inc.', 'US', 'Technology', 'NASDAQ', 5.80, 15.75, 138.75, 1750000000000, 22000000, 92.5)
ON CONFLICT DO NOTHING;

-- Insert sample notification for development
INSERT INTO notifications (id, user_id, title, message, notification_type, priority, is_read) 
SELECT 
    gen_random_uuid(),
    u.id,
    'Welcome to EPSX',
    'Your account has been set up successfully. Explore the platform and start analyzing EPS growth data.',
    'welcome',
    'high',
    false
FROM users u 
WHERE u.email = 'info@epsx.io'
ON CONFLICT DO NOTHING;

-- ============================================================================
-- ADMIN MODULE PERMISSIONS (EXTENDED SET)
-- ============================================================================

-- Insert remaining module permissions that weren't in the main migration
INSERT INTO admin_module_permissions (module_code, api_endpoints, frontend_routes, permissions, resource_patterns, access_level, description) VALUES

-- Role & Policy Manager
('role_policy_manager',
 ARRAY['/api/v1/admin/casbin/*', '/api/v1/admin/roles/*'],
 ARRAY['/iam', '/iam/*'],
 ARRAY['role:read', 'role:write', 'policy:manage', 'casbin:admin'],
 ARRAY['casbin_rules/*', 'roles/*'],
 'write',
 'Casbin policy and role management with testing capabilities'),

-- Developer Relations
('developer_relations',
 ARRAY['/api/v1/admin/api-keys/*', '/api/v1/admin/developer-portal/*'],
 ARRAY['/developer-portal', '/developer-portal/*'],
 ARRAY['api_key:manage', 'developer:tools', 'documentation:manage'],
 ARRAY['api_keys/*', 'developer_tools/*'],
 'write',
 'API key management and developer resource administration'),

-- Module Coordinator
('module_coordinator',
 ARRAY['/api/v1/admin/modules/*', '/api/v1/admin/users/*/modules'],
 ARRAY['/modules', '/modules/*'],
 ARRAY['module:read', 'module:write', 'module:assign', 'feature:manage'],
 ARRAY['modules/*', 'user_modules/*'],
 'write',
 'Feature module assignments and access control management'),

-- Compliance & Audit Officer  
('compliance_audit',
 ARRAY['/api/v1/admin/permissions/audit-report', '/api/v1/admin/permissions/system-backup/*', '/api/v1/admin/analytics/security-risks'],
 ARRAY['/compliance', '/audit/*'],
 ARRAY['audit:read', 'compliance:manage', 'backup:create', 'security:analyze'],
 ARRAY['audit/*', 'compliance/*', 'backups/*'],
 'read',
 'Security auditing, compliance reporting, and system backup management'),

-- Support Specialist
('support_specialist',
 ARRAY['/api/v1/admin/users/*/activity', '/api/v1/admin/support/*'],
 ARRAY['/support', '/users/*/support'],
 ARRAY['user:read', 'support:tickets', 'activity:view'],
 ARRAY['users/*/read', 'support/*'],
 'read',
 'Read-only user access for support and troubleshooting purposes')

ON CONFLICT DO NOTHING;

-- ============================================================================
-- PRODUCTION DEPLOYMENT NOTES
-- ============================================================================

-- This seed data file contains:
-- 1. Production admin user setup (info@epsx.io)
-- 2. Sample analytics data for initial content
-- 3. Extended module permissions for admin system
-- 4. Welcome notification for admin user
--
-- Production configuration:
-- - Uses info@epsx.io as production admin
-- - Firebase UID: prod_admin_uid_epsx_2024
-- - Full system administrator privileges