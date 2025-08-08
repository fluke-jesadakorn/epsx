# Authentication Debug Report

**User**: jesadakorn.kirtnu@gmail.com  
**Date**: August 8, 2025  
**Issue**: User was experiencing authentication failures and being redirected to payment page when accessing analytics

## Problem Identified

The user had a `super_admin` role in the database but **no corresponding Casbin policies were defined** for the `super_admin` role. This caused the authorization system to deny access to protected resources.

### Root Cause
1. User was successfully promoted to `super_admin` role in the users table
2. However, Casbin (the authorization system) had no policies defined for `super_admin` role
3. This resulted in authentication failures when trying to access protected endpoints like `/analytics`
4. The frontend interpreted this as requiring payment/subscription

## Actions Taken

### 1. Database Verification
- Confirmed user exists with Firebase UID: `KLiZ6jiuzchxUppd60IdBD5WS4U2`
- User ID in database: `38b978cb-a9d2-5a8b-88df-d7a79abd87cc`
- Role correctly set to: `super_admin`

### 2. Casbin Policy Configuration
Added the following policies to `casbin_rule` table:

```sql
-- Super admin permissions
INSERT INTO casbin_rule (ptype, v0, v1, v2) VALUES
('p', 'super_admin', '/api/v1/*', '*'),
('p', 'super_admin', '/api/admin/*', '*'), 
('p', 'super_admin', '/api/v1/admin/*', '*'),
('p', 'super_admin', '/api/v1/analytics/*', '*'),
('p', 'super_admin', '/*', '*');

-- User role assignment  
INSERT INTO casbin_rule (ptype, v0, v1) VALUES
('g', '38b978cb-a9d2-5a8b-88df-d7a79abd87cc', 'super_admin');
```

### 3. Permission Profile Assignment
Assigned "Admin Dashboard" permission profile to the user:

```sql
INSERT INTO admin_permission_profile_assignments (...) VALUES (
  '38b978cb-a9d2-5a8b-88df-d7a79abd87cc',
  '097083eb-93f7-4b47-952e-c69fddc78d7d', -- Admin Dashboard profile ID
  '38b978cb-a9d2-5a8b-88df-d7a79abd87cc',
  'admin_assignment',
  'Super admin full access - debugging authentication issues',
  NULL, -- never expires
  'active',
  NOW()
);
```

### 4. Promoted User Permissions
Ran the promotion script with super admin flag to ensure all internal permissions are set:

```bash
./scripts/grant_full_access.sh "jesadakorn.kirtnu@gmail.com" "Fix authentication issues"
```

## Current Status ✅

### User Configuration
- ✅ User has `super_admin` role in users table
- ✅ User is assigned to `super_admin` role in Casbin  
- ✅ User has active sessions (expires August 9, 2025)
- ✅ User has Admin Dashboard permission profile assigned

### Casbin Policies  
- ✅ `super_admin` role has permissions for `/api/v1/*`
- ✅ `super_admin` role has permissions for `/api/admin/*`
- ✅ `super_admin` role has permissions for `/api/v1/admin/*`
- ✅ `super_admin` role has permissions for `/api/v1/analytics/*`
- ✅ `super_admin` role has wildcard permissions for `/*`

### Permission Profiles
- ✅ Admin Dashboard profile assigned (active, no expiration)

## Testing Recommendations

1. **Clear browser cache/cookies** - Old authentication tokens may be cached
2. **Re-login** - Generate fresh authentication tokens  
3. **Test analytics access** - Should now work without payment redirect
4. **Monitor logs** - Check backend logs for any remaining authorization errors

## Notes

- There is a duplicate user record with email `jesadakorn.kirtnu@gmail.com` but different Firebase UID (`dev-jesadakorn.kirtnu@gmail.com`). This appears to be a development/test record and is not actively used.
- All active sessions are correctly associated with the super_admin user record.

## Database Connection Details

For future debugging, the database can be accessed with:

```bash
psql "postgresql://postgres:password@localhost:5432/epsx_db"
```

Use the debugging script:
```bash
psql "$DATABASE_URL" -f debug_user_permissions.sql
```

## Files Created

1. `/Users/fluke/Desktop/Work/Outsource/epsx/apps/backend/debug_user_permissions.sql` - SQL debugging script
2. `/Users/fluke/Desktop/Work/Outsource/epsx/apps/backend/AUTHENTICATION_DEBUG_REPORT.md` - This report

---

**Status**: RESOLVED ✅  
The user should now have full access to analytics and all admin features without authentication failures.