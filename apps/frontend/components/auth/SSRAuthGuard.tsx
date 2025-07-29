import { redirect } from 'next/navigation';
import { getServerAuth, requireAuth, requirePermission } from '@/lib/auth-server';

interface SSRAuthGuardProps {
  children: React.ReactNode;
  requiredPermission?: string;
  redirectTo?: string;
  fallback?: React.ReactNode;
}

/**
 * Server-side authentication guard that handles auth checks during SSR
 * This component runs on the server and redirects before the page loads
 */
export async function SSRAuthGuard({ 
  children, 
  requiredPermission, 
  redirectTo = '/login',
  fallback 
}: SSRAuthGuardProps) {
  try {
    if (requiredPermission) {
      // Require specific permission (will redirect if not found)
      await requirePermission(requiredPermission);
    } else {
      // Just require authentication (will redirect if not authenticated)
      await requireAuth();
    }

    // If we get here, the user is authenticated and has required permissions
    return <>{children}</>;
  } catch (error) {
    // Handle any errors gracefully
    console.error('SSR Auth Guard error:', error);
    
    if (fallback) {
      return <>{fallback}</>;
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
  requiredPermission: string;
  fallback?: React.ReactNode;
}

export async function SSRRoleContent({ 
  children, 
  requiredPermission, 
  fallback 
}: SSRRoleContentProps) {
  const authResult = await getServerAuth();
  
  if (!authResult.isAuthenticated) {
    return fallback || null;
  }
  
  const hasPermission = authResult.user?.permissions?.includes(requiredPermission) ||
    authResult.user?.permissions?.some(permission => {
      if (permission.endsWith('.*')) {
        const prefix = permission.slice(0, -2);
        return requiredPermission.startsWith(prefix + '.');
      }
      return false;
    });
  
  if (!hasPermission) {
    return fallback || null;
  }
  
  return <>{children}</>;
}

/**
 * Server-side user info display
 */
export async function SSRUserInfo() {
  const authResult = await getServerAuth();
  
  if (!authResult.isAuthenticated || !authResult.user) {
    return (
      <div className="text-sm text-gray-500">
        Not authenticated
      </div>
    );
  }
  
  return (
    <div className="text-sm">
      <div className="font-medium">{authResult.user.email}</div>
      <div className="text-gray-500">
        {authResult.user.permissions?.length || 0} permissions
      </div>
      {authResult.user.package_tier && (
        <div className="text-xs text-blue-600">
          {authResult.user.package_tier} tier
        </div>
      )}
    </div>
  );
}