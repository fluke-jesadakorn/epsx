import type { NextRequest } from 'next/server';
import { createAdminMiddleware } from '@epsx/auth-shared/middleware';
import type { RoutePermissionConfig } from '@epsx/auth-shared';

// Admin route definitions with strict permission requirements
const adminRoutePermissions: Record<string, RoutePermissionConfig> = {
  '/': { 
    permission: 'admin.dashboard.view', 
    fallbackRole: 'admin',
    description: 'Admin dashboard access'
  },
  '/dashboard': { 
    permission: 'admin.dashboard.view', 
    fallbackRole: 'admin',
    description: 'Admin dashboard access'
  },
  '/users': { 
    permission: 'admin.users.manage', 
    profile: 'Admin Assistant',
    fallbackRole: 'admin',
    description: 'User management interface'
  },
  '/user-management': { 
    permission: 'admin.users.manage', 
    profile: 'Admin Assistant',
    fallbackRole: 'admin',
    description: 'Enhanced user management'
  },
  '/permission-profiles': {
    permission: 'admin.permission_profiles.manage',
    profile: 'System Administrator',
    fallbackRole: 'admin',
    description: 'Permission profile management'
  },
  '/stock-ranking-packages': {
    permission: 'admin.stock_rankings.manage',
    profile: 'Content Manager',
    fallbackRole: 'admin', 
    description: 'Stock ranking package management'
  },
  '/analytics': { 
    permission: 'admin.analytics.view', 
    profile: 'Admin Assistant',
    fallbackRole: 'admin',
    description: 'Admin analytics dashboard'
  },
  '/iam': {
    permission: 'admin.iam.manage',
    profile: 'System Administrator',
    fallbackRole: 'admin',
    description: 'Identity and Access Management'
  },
  '/settings': {
    permission: 'admin.settings.manage',
    profile: 'System Administrator', 
    fallbackRole: 'admin',
    description: 'Admin settings management'
  },
  '/database': {
    permission: 'admin.database.manage',
    profile: 'System Administrator',
    fallbackRole: 'admin',
    description: 'Database management interface'
  },
  '/system': { 
    permission: 'admin.system.configure', 
    profile: 'System Administrator',
    fallbackRole: 'super_admin',
    description: 'System configuration'
  },
  '/audit': { 
    permission: 'admin.audit.view', 
    profile: 'Admin Assistant',
    fallbackRole: 'admin',
    description: 'Audit log access'
  },
  '/api/admin': { 
    permission: 'api:admin:*', 
    fallbackRole: 'admin',
    description: 'Admin API access'
  },
  '/billing': {
    permission: 'admin.billing.manage',
    profile: 'Admin Assistant',
    fallbackRole: 'admin',
    description: 'Billing and subscription management'
  },
  '/developer-portal': {
    permission: 'admin.developer_portal.manage',
    profile: 'System Administrator',
    fallbackRole: 'admin',
    description: 'Developer portal management'
  },
  '/modules': {
    permission: 'admin.modules.manage',
    profile: 'System Administrator',
    fallbackRole: 'admin',
    description: 'Module system management'
  },
  '/permissions-demo': {
    permission: 'admin.permissions.demo',
    profile: 'Admin Assistant',
    fallbackRole: 'admin',
    description: 'Permission system demonstration'
  }
};

// Create the unified middleware with admin configuration
const middleware = createAdminMiddleware(adminRoutePermissions);

export default middleware;

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     * - public folder
     */
    '/((?!api|_next/static|_next/image|favicon.ico|public).*)',
  ],
};