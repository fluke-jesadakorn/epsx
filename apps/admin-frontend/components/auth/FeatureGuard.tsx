import React from 'react';
import { useFeatureAccess } from '../../hooks/useFeatureAccess';

interface FeatureGuardProps {
  featureId: string;
  userId?: string;
  children: React.ReactNode;
  fallback?: React.ReactNode;
  loadingComponent?: React.ReactNode;
}

export const FeatureGuard: React.FC<FeatureGuardProps> = ({
  featureId,
  userId,
  children,
  fallback = null,
  loadingComponent = <div>Checking permissions...</div>,
}) => {
  const { hasAccess, loading, error } = useFeatureAccess(featureId, userId);
  
  if (loading) return <>{loadingComponent}</>;
  
  if (error) {
    console.error('Feature access check failed:', error);
    return <>{fallback}</>;
  }
  
  return hasAccess ? <>{children}</> : <>{fallback}</>;
};

interface TierGuardProps {
  requiredTier: string;
  userTier?: string;
  children: React.ReactNode;
  fallback?: React.ReactNode;
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

// Usage examples:
/*
<FeatureGuard featureId="dashboard_advanced" userId={currentUser.id}>
  <AdvancedDashboard />
</FeatureGuard>

<TierGuard requiredTier="gold" userTier={user.packageTier}>
  <PremiumFeature />
</TierGuard>
*/
