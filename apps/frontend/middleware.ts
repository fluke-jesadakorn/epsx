import type { NextRequest } from 'next/server';
import { createFrontendMiddleware } from '@epsx/auth-shared/middleware';
import type { RoutePermissionConfig } from '@epsx/auth-shared';

// Enhanced route definitions with permission profile integration
const routePermissions: Record<string, RoutePermissionConfig> = {
  '/dashboard': { 
    permission: 'route:/dashboard', 
    fallbackRole: 'user',
    description: 'User dashboard access'
  },
  '/analytics': { 
    permission: 'route:/analytics/*', 
    profile: 'Silver User', 
    fallbackRole: 'premium',
    description: 'Analytics and reports access'
  },
  '/my-data': { 
    permission: 'route:/profile/*', 
    fallbackRole: 'user',
    description: 'User profile and data access'
  },
  '/admin': { 
    permission: 'route:/admin/*', 
    fallbackRole: 'admin',
    description: 'Admin interface access'
  },
  '/users': { 
    permission: 'admin.users.view', 
    fallbackRole: 'admin',
    description: 'User management access'
  },
  '/settings': { 
    permission: 'route:/settings', 
    fallbackRole: 'user',
    description: 'User settings access'
  },
  '/reports': { 
    permission: 'route:/reports/*', 
    profile: 'Gold User', 
    fallbackRole: 'premium',
    description: 'Advanced reports access'
  },
  '/trading': { 
    permission: 'route:/trading/*', 
    profile: 'Silver User', 
    fallbackRole: 'premium',
    description: 'Trading features access'
  },
  '/payment': { 
    permission: 'route:/payment/*', 
    fallbackRole: 'user',
    description: 'Payment and billing access'
  },
  '/premium': { 
    permission: 'route:/premium/*', 
    profile: 'Silver User', 
    fallbackRole: 'premium',
    description: 'Premium features access'
  },
  '/rankings': { 
    permission: 'api:rankings:read', 
    profile: 'Bronze User', 
    fallbackRole: 'user',
    description: 'Stock rankings access'
  },
  '/stock-rankings': { 
    permission: 'api:stock-rankings:read', 
    profile: 'Bronze User', 
    fallbackRole: 'user',
    description: 'Stock rankings access'
  },
};

// Create the unified middleware with frontend configuration
const middleware = createFrontendMiddleware(routePermissions);

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