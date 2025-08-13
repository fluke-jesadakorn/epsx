'use client';

import React, { ReactNode, useEffect, useState } from 'react';
import { useAuth, usePermission } from '@/components/providers/AuthContext';
import { ErrorDisplay } from '@/components/ui/ErrorDisplay';

interface FeatureGuardProps {
  featureId?: string; // Keep for backward compatibility
  feature?: string;   // New prop name
  children: ReactNode;
  fallback?: ReactNode;
  loadingComponent?: ReactNode;
  requireAll?: boolean;
  features?: string[];
}

export const FeatureGuard: React.FC<FeatureGuardProps> = ({
  featureId,
  feature,
  children,
  fallback = null,
  loadingComponent = <div>Checking permissions...</div>,
  requireAll = true,
  features,
}) => {
  const { isAuthenticated, isLoading, permissions } = useAuth();
  const [hasAccess, setHasAccess] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  // Use featureId for backward compatibility, otherwise use feature
  const targetFeature = feature || featureId;

  const retryCheck = () => {
    setError(null);
    setHasAccess(null);
    setLoading(true);
  };

  // Helper function to check permissions using NextAuth session
  const hasPermission = (permission: string): boolean => {
    return permissions.includes(permission);
  };

  useEffect(() => {
    const checkAccess = async () => {
      try {
        setLoading(true);
        setError(null);
        
        if (!isAuthenticated) {
          setHasAccess(false);
          return;
        }
        
        let result: boolean;
        
        if (features && features.length > 0) {
          // Check multiple features
          const results = features.map(f => hasPermission(f));
          
          result = requireAll 
            ? results.every(r => r) // All must be true
            : results.some(r => r);  // At least one must be true
        } else if (targetFeature) {
          // Check single feature
          result = hasPermission(targetFeature);
        } else {
          // No feature specified, allow access
          result = true;
        }
        
        setHasAccess(result);
      } catch (err) {
        console.error('Feature access check failed:', err);
        const error = err instanceof Error ? err : new Error('Feature access check failed');
        setError(error);
        setHasAccess(false);
      } finally {
        setLoading(false);
      }
    };

    if (!isLoading) {
      checkAccess();
    }
  }, [targetFeature, features, requireAll, permissions, isAuthenticated, isLoading]);

  if (isLoading || loading) return <>{loadingComponent}</>;
  
  // Show error with retry option
  if (error) {
    return (
      <ErrorDisplay
        error={error}
        context="permission"
        title="Permission Check Failed"
        onRetry={retryCheck}
        className="my-4"
      />
    );
  }
  
  // Show permission denied with better UX
  if (hasAccess === false) {
    if (!isAuthenticated) {
      return (
        <ErrorDisplay
          error="Authentication required"
          context="auth"
          title="Sign In Required"
          className="my-4"
        />
      );
    }
    
    return (
      <ErrorDisplay
        error={`Access denied for feature: ${targetFeature || features?.join(', ')}`}
        context="permission"
        title="Access Denied"
        className="my-4"
      />
    );
  }
  
  return hasAccess ? <>{children}</> : <>{fallback}</>;
};

// Utility component for route-based protection
interface RouteGuardProps {
  route: string;
  children: ReactNode;
  fallback?: ReactNode;
  loadingComponent?: ReactNode;
}

export const RouteGuard: React.FC<RouteGuardProps> = ({ 
  route, 
  children, 
  fallback = null,
  loadingComponent = <div>Checking route access...</div>
}) => {
  const { isAuthenticated, isLoading, adminModules } = useAuth();
  const [canAccess, setCanAccess] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const retryRouteCheck = () => {
    setError(null);
    setCanAccess(null);
    setLoading(true);
  };

  // Admin module route mapping
  const routeModuleMap: Record<string, string> = {
    '/users': 'user_operations',
    '/analytics': 'analytics_specialist',
    '/billing': 'billing_admin',
    '/settings': 'system_admin',
    '/permissions': 'permission_admin',
    '/modules': 'module_coordinator'
  };

  const canAccessRoute = (route: string): boolean => {
    const requiredModule = routeModuleMap[route];
    if (!requiredModule) return true; // No specific module required
    return adminModules.includes(requiredModule);
  };

  useEffect(() => {
    const checkRouteAccess = async () => {
      try {
        setLoading(true);
        setError(null);
        
        if (!isAuthenticated) {
          setCanAccess(false);
          return;
        }
        
        const result = canAccessRoute(route);
        setCanAccess(result);
      } catch (err) {
        console.error('Route access check failed:', err);
        const error = err instanceof Error ? err : new Error('Route access check failed');
        setError(error);
        setCanAccess(false);
      } finally {
        setLoading(false);
      }
    };

    if (!isLoading) {
      checkRouteAccess();
    }
  }, [route, adminModules, isAuthenticated, isLoading]);

  if (isLoading || loading) return <>{loadingComponent}</>;
  
  // Show error with retry option
  if (error) {
    return (
      <ErrorDisplay
        error={error}
        context="permission"
        title="Route Access Check Failed"
        onRetry={retryRouteCheck}
        className="my-4"
      />
    );
  }
  
  // Show route access denied with better UX
  if (canAccess === false) {
    if (!isAuthenticated) {
      return (
        <ErrorDisplay
          error="Authentication required"
          context="auth"
          title="Sign In Required"
          className="my-4"
        />
      );
    }
    
    return (
      <ErrorDisplay
        error={`Access denied for route: ${route}`}
        context="permission"
        title="Route Access Denied"
        className="my-4"
      />
    );
  }
  
  return canAccess ? <>{children}</> : <>{fallback}</>;
};

// Keep TierGuard for backward compatibility but simplify it
interface TierGuardProps {
  requiredTier: string;
  userTier?: string;
  children: ReactNode;
  fallback?: ReactNode;
}

export const TierGuard: React.FC<TierGuardProps> = ({
  requiredTier,
  userTier,
  children,
  fallback = null,
}) => {
  const tierHierarchy = {
    'free': 0,
    'bronze': 1,
    'silver': 2,
    'gold': 3,
    'platinum': 4,
    'enterprise': 5,
  };

  const userLevel = tierHierarchy[userTier?.toLowerCase() as keyof typeof tierHierarchy] || 0;
  const requiredLevel = tierHierarchy[requiredTier.toLowerCase() as keyof typeof tierHierarchy] || 0;

  const hasAccess = userLevel >= requiredLevel;

  return hasAccess ? <>{children}</> : <>{fallback}</>;
};

// Higher-order component for feature protection
export function withFeatureGuard<P extends object>(
  Component: React.ComponentType<P>, 
  feature: string,
  fallback?: ReactNode
) {
  return function FeatureProtectedComponent(props: P) {
    return (
      <FeatureGuard feature={feature} fallback={fallback}>
        <Component {...props} />
      </FeatureGuard>
    );
  };
}

// Usage examples:
/*
<FeatureGuard feature="ADMIN_ACCESS">
  <DashboardServer />
</FeatureGuard>

<RouteGuard route="/users">
  <UserManagement />
</RouteGuard>

<TierGuard requiredTier="gold" userTier={user.packageTier}>
  <PremiumFeature />
</TierGuard>
*/
