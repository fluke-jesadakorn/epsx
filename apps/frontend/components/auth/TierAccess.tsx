import { ReactNode } from 'react';
import { getCurrentUser } from '@/lib/server-actions';
import { Role } from '@/lib/auth/roles';

interface SimpleRoleAccessProps {
  feature?: string;
  role?: Role;
  children: ReactNode;
  fallback?: ReactNode;
}

/**
 * Simple server-side role access component
 * Uses unified simple role system (admin/user/guest)
 */
export async function SimpleRoleAccess({
  feature,
  role,
  children,
  fallback,
}: SimpleRoleAccessProps) {
  let user = null;
  try {
    const result = await getCurrentUser({});
    user = result?.success ? result.data : null;
  } catch (error) {
    console.error('SimpleRoleAccess: Failed to get user:', error);
  }
  
  if (!user) {
    return fallback || <div>Authentication required</div>;
  }
  
  // Map user role to our simple system
  const userRole = (user as any).role || Role.Guest;
  
  // Import role checking logic
  const { checkFeatureAccess, checkRoleAccess } = await import('@/lib/auth/roles');
  
  // Check access
  let hasAccess = true;
  if (feature) {
    hasAccess = checkFeatureAccess(userRole, feature);
  }
  if (role && hasAccess) {
    hasAccess = checkRoleAccess(userRole, role);
  }
  
  if (!hasAccess) {
    if (fallback) {
      return <>{fallback}</>;
    }
    
    return (
      <div className="bg-gradient-to-r from-red-50 to-orange-50 dark:from-red-950/20 dark:to-orange-950/20 border border-red-200 dark:border-red-800 rounded-lg p-6 text-center">
        <div className="text-red-800 dark:text-red-200">
          <h3 className="font-semibold mb-2">Access Restricted</h3>
          <p className="text-sm mb-2">
            {feature ? `Feature "${feature}" requires higher permissions.` : `Role "${role}" required.`}
          </p>
          <p className="text-xs text-red-600 dark:text-red-400">
            Your current role: <span className="font-medium capitalize">{userRole}</span>
          </p>
        </div>
      </div>
    );
  }
  
  return <>{children}</>;
}

/**
 * Simple premium access (User role or higher)
 */
export async function PremiumAccess({ children }: { children: ReactNode }) {
  return (
    <SimpleRoleAccess role={Role.User}>
      {children}
    </SimpleRoleAccess>
  );
}

/**
 * Simple admin access (Admin role only)
 */
export async function AdminAccess({ children }: { children: ReactNode }) {
  return (
    <SimpleRoleAccess 
      role={Role.Admin}
      fallback={
        <div className="bg-slate-100 dark:bg-slate-900/50 border border-slate-300 dark:border-slate-700 rounded-lg p-8 text-center">
          <div className="text-slate-700 dark:text-slate-300">
            <h3 className="text-xl font-bold mb-3">Admin Only</h3>
            <p className="mb-4">This feature is available to administrators only.</p>
            <div className="space-y-2 text-sm mb-6">
              <div>• Full system access</div>
              <div>• User management</div>
              <div>• System configuration</div>
              <div>• Advanced analytics</div>
            </div>
          </div>
        </div>
      }
    >
      {children}
    </SimpleRoleAccess>
  );
}