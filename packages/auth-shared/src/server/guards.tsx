import { redirect } from 'next/navigation';
import { getServerAuth, requireAuth, requirePermission, requireRole } from './auth';
import type { AuthServerConfig } from './auth';

interface SSRAuthGuardProps {
  children: React.ReactNode;
  requiredPermission?: string;
  requiredRole?: string;
  redirectTo?: string;
  fallback?: React.ReactNode;
  config?: AuthServerConfig;
}

/**
 * Server-side authentication guard that handles auth checks during SSR
 * This component runs on the server and redirects before the page loads
 */
export async function SSRAuthGuard({ 
  children, 
  requiredPermission,
  requiredRole,
  redirectTo = '/login',
  fallback,
  config = {}
}: SSRAuthGuardProps): Promise<React.ReactElement> {
  try {
    if (requiredPermission) {
      // Require specific permission (will redirect if not found)
      await requirePermission(requiredPermission, undefined, config);
    } else if (requiredRole) {
      // Require specific role (will redirect if not found)
      await requireRole(requiredRole, undefined, config);
    } else {
      // Just require authentication (will redirect if not authenticated)
      await requireAuth(undefined, config);
    }

    // If we get here, the user is authenticated and has required permissions
    return <>{children}</> as React.ReactElement;
  } catch (error) {
    // Handle any errors gracefully
    console.error('SSR Auth Guard error:', error);
    
    if (fallback) {
      return <>{fallback}</> as React.ReactElement;
    }
    
    // Redirect to login as fallback
    redirect(redirectTo);
  }
}

/**
 * Server-side role-based content renderer
 */
interface SSRRoleContentProps {
  children: React.ReactNode;
  requiredPermission?: string;
  requiredRole?: string;
  fallback?: React.ReactNode;
  config?: AuthServerConfig;
}

export async function SSRRoleContent({ 
  children, 
  requiredPermission,
  requiredRole,
  fallback,
  config = {}
}: SSRRoleContentProps): Promise<React.ReactElement | null> {
  const authResult = await getServerAuth(config);
  
  if (!authResult.isAuthenticated) {
    return (fallback as React.ReactElement) || null;
  }
  
  // Check permission if required
  if (requiredPermission) {
    const hasPermission = authResult.user?.permissions?.includes(requiredPermission) ||
      authResult.user?.permissions?.some(permission => {
        if (permission.endsWith('.*') || permission.endsWith(':*')) {
          const prefix = permission.slice(0, -2);
          return requiredPermission.startsWith(prefix + '.') || requiredPermission.startsWith(prefix + ':');
        }
        if (permission === '*') {
          return true;
        }
        return false;
      });
    
    if (!hasPermission) {
      return (fallback as React.ReactElement) || null;
    }
  }
  
  // Check role if required
  if (requiredRole) {
    const ROLE_HIERARCHY: Record<string, number> = {
      'user': 1,
      'premium': 2,
      'moderator': 3,
      'admin': 4,
      'super_admin': 5
    };

    const userLevel = ROLE_HIERARCHY[authResult.user?.role?.toLowerCase() || ''] || 0;
    const requiredLevel = ROLE_HIERARCHY[requiredRole.toLowerCase()] || 1;

    if (userLevel < requiredLevel) {
      return (fallback as React.ReactElement) || null;
    }
  }
  
  return <>{children}</>;
}

/**
 * Server-side user info display
 */
interface SSRUserInfoProps {
  showPermissions?: boolean;
  showRole?: boolean;
  showPackageTier?: boolean;
  config?: AuthServerConfig;
}

export async function SSRUserInfo({ 
  showPermissions = false,
  showRole = true,
  showPackageTier = true,
  config = {}
}: SSRUserInfoProps): Promise<React.ReactElement> {
  const authResult = await getServerAuth(config);
  
  if (!authResult.isAuthenticated || !authResult.user) {
    return (
      <div className="text-sm text-gray-500">
        Not authenticated
      </div>
    );
  }
  
  const { user } = authResult;
  
  return (
    <div className="text-sm">
      <div className="font-medium">{user.displayName || user.email}</div>
      {showRole && user.role && (
        <div className="text-gray-500 capitalize">
          {user.role.replace('_', ' ')}
        </div>
      )}
      {showPermissions && (
        <div className="text-gray-500">
          {user.permissions?.length || 0} permissions
        </div>
      )}
      {showPackageTier && user.package_tier && (
        <div className="text-xs text-blue-600 capitalize">
          {user.package_tier} tier
        </div>
      )}
    </div>
  );
}

/**
 * Admin-specific auth guard
 */
interface SSRAdminGuardProps {
  children: React.ReactNode;
  minimumRole?: 'admin' | 'super_admin';
  redirectTo?: string;
  fallback?: React.ReactNode;
}

export function SSRAdminGuard({
  children,
  minimumRole = 'admin',
  redirectTo = '/login',
  fallback
}: SSRAdminGuardProps) {
  return SSRAuthGuard({
    children,
    requiredRole: minimumRole,
    redirectTo,
    fallback,
    config: { adminSessionCookieName: 'admin_sess_id' }
  });
}