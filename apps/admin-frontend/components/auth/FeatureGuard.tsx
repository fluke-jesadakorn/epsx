'use client';

import React, { ReactNode, useEffect, useState } from 'react';
import { authUtils } from '@/lib/auth-api';

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
  const [hasAccess, setHasAccess] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);

  // Use featureId for backward compatibility, otherwise use feature
  const targetFeature = feature || featureId;

  useEffect(() => {
    const checkAccess = async () => {
      try {
        setLoading(true);
        
        let result: boolean;
        
        if (features && features.length > 0) {
          // Check multiple features
          const results = await Promise.all(
            features.map(f => authUtils.hasPermission(f))
          );
          
          result = requireAll 
            ? results.every(r => r) // All must be true
            : results.some(r => r);  // At least one must be true
        } else if (targetFeature) {
          // Check single feature
          result = await authUtils.hasPermission(targetFeature);
        } else {
          // No feature specified, allow access
          result = true;
        }
        
        setHasAccess(result);
      } catch (error) {
        console.error('Feature access check failed:', error);
        setHasAccess(false);
      } finally {
        setLoading(false);
      }
    };

    checkAccess();
  }, [targetFeature, features, requireAll]);

  if (loading) return <>{loadingComponent}</>;
  
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
  const [canAccess, setCanAccess] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const checkRouteAccess = async () => {
      try {
        setLoading(true);
        const result = await authUtils.canAccessRoute(route);
        setCanAccess(result);
      } catch (error) {
        console.error('Route access check failed:', error);
        setCanAccess(false);
      } finally {
        setLoading(false);
      }
    };

    checkRouteAccess();
  }, [route]);

  if (loading) return <>{loadingComponent}</>;
  
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
  <AdminDashboard />
</FeatureGuard>

<RouteGuard route="/admin/users">
  <UserManagement />
</RouteGuard>

<TierGuard requiredTier="gold" userTier={user.packageTier}>
  <PremiumFeature />
</TierGuard>
*/
