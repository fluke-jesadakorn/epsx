-- User Permission Debugging Report for jesadakorn.kirtnu@gmail.com
-- Run with: psql "$DATABASE_URL" -f debug_user_permissions.sql

\echo '=== USER PERMISSION DEBUGGING REPORT ==='
\echo 'Email: jesadakorn.kirtnu@gmail.com'
\echo 'Generated at:' `date`
\echo ''

\echo '1. USER RECORDS IN DATABASE:'
SELECT 
    'User Record' as record_type,
    id, 
    firebase_uid, 
    email, 
    role, 
    created_at,
    updated_at
FROM users 
WHERE email = 'jesadakorn.kirtnu@gmail.com' 
ORDER BY created_at;

\echo ''
\echo '2. CASBIN ROLE ASSIGNMENTS (g policies):'
SELECT 
    'Role Assignment' as record_type,
    cr.v0 as user_id, 
    cr.v1 as assigned_role
FROM casbin_rule cr 
WHERE cr.ptype = 'g' 
  AND cr.v0 IN (SELECT id::text FROM users WHERE email = 'jesadakorn.kirtnu@gmail.com');

\echo ''
\echo '3. CASBIN PERMISSIONS (p policies for roles):'
SELECT 
    'Permission Rule' as record_type,
    cr.v0 as role_name, 
    cr.v1 as resource, 
    cr.v2 as action
FROM casbin_rule cr 
WHERE cr.ptype = 'p' 
  AND cr.v0 IN ('super_admin', 'admin');

\echo ''
\echo '4. ADMIN PERMISSION PROFILE ASSIGNMENTS:'
SELECT 
    'Profile Assignment' as record_type,
    appa.user_id,
    pp.name as profile_name,
    pp.category,
    appa.status,
    appa.expires_at,
    appa.assignment_reason
FROM admin_permission_profile_assignments appa
JOIN permission_profiles pp ON appa.permission_profile_id = pp.id
WHERE appa.user_id IN (SELECT id FROM users WHERE email = 'jesadakorn.kirtnu@gmail.com')
  AND appa.status = 'active';

\echo ''
\echo '5. ACTIVE SESSIONS (last 5):'
SELECT 
    'Session' as record_type,
    s.id as session_id,
    s.user_id,
    s.expires_at,
    s.is_active,
    s.created_at
FROM sessions s
JOIN users u ON s.user_id = u.id
WHERE u.email = 'jesadakorn.kirtnu@gmail.com'
ORDER BY s.created_at DESC
LIMIT 5;

\echo ''
\echo '6. SUMMARY:'
\echo 'Expected configuration:'
\echo '  - User should have role: super_admin'
\echo '  - Casbin should have user assigned to super_admin role'  
\echo '  - Casbin should have super_admin permissions for /api/v1/*, /api/admin/*, etc.'
\echo '  - User should have Admin Dashboard permission profile'
\echo '  - Sessions should be valid and active'
\echo ''
\echo '=== END OF REPORT ==='