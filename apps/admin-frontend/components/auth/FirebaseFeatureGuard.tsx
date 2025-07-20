import React from 'react';
import type { ReactNode } from 'react';
import { useFeatureAccess } from '../../hooks/useFirebaseFeatureAccess';
import { useAdminAuth } from '../../hooks/useAdminAuth';

interface FeatureGuardProps {
  featureId: string;
  children: ReactNode;
  fallback?: ReactNode;
  requireAll?: boolean; // For multiple features
  features?: string[]; // Alternative to single featureId
}

/**
 * Component that conditionally renders children based on user's feature access
 * Uses Firebase Firestore to check permissions in real-time
 */
export const FirebaseFeatureGuard: React.FC<FeatureGuardProps> = ({
  featureId,
  children,
  fallback = null,
  requireAll = false,
  features = []
}) => {
  const { adminUser } = useAdminAuth();
  const userId = adminUser?.id || null;
  
  // Handle multiple features
  const featuresToCheck = features.length > 0 ? features : [featureId];
  
  // Check access for all features
  const accessChecks = featuresToCheck.map(feature => 
    useFeatureAccess(userId, feature)
  );
  
  // Calculate if user has access
  const hasAccess = requireAll 
    ? accessChecks.every(check => check.hasAccess)
    : accessChecks.some(check => check.hasAccess);
  
  const isLoading = accessChecks.some(check => check.loading);
  const hasError = accessChecks.some(check => check.error);
  
  // Show loading state
  if (isLoading) {
    return (
      <div className="animate-pulse">
        <div className="h-4 bg-gray-200 rounded w-3/4"></div>
      </div>
    );
  }
  
  // Show error state (optional - you might want to show children instead)
  if (hasError) {
    console.warn('Feature access check failed, defaulting to no access');
    return <>{fallback}</>;
  }
  
  // Render based on access
  return hasAccess ? <>{children}</> : <>{fallback}</>;
};

// Convenience component for common use cases
export const FirebaseAdminFeatureGuard: React.FC<{
  feature: string;
  children: ReactNode;
  fallback?: ReactNode;
}> = ({ feature, children, fallback }) => (
  <FirebaseFeatureGuard 
    featureId={feature} 
    fallback={fallback || (
      <div className="p-4 bg-gray-100 rounded-lg text-center text-gray-500">
        You don't have access to this feature.
      </div>
    )}
  >
    {children}
  </FirebaseFeatureGuard>
);

// Component for checking package tier access
export const FirebasePackageTierGuard: React.FC<{
  requiredTier: string;
  children: ReactNode;
  upgradeMessage?: string;
}> = ({ requiredTier, children, upgradeMessage }) => (
  <FirebaseFeatureGuard 
    featureId={`package:${requiredTier}`}
    fallback={
      <div className="p-6 border-2 border-dashed border-gray-300 rounded-lg text-center">
        <div className="text-gray-600 mb-2">
          {upgradeMessage || `This feature requires ${requiredTier} package or higher.`}
        </div>
        <button 
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          onClick={() => window.open('/upgrade', '_blank')}
        >
          Upgrade Package
        </button>
      </div>
    }
  >
    {children}
  </FirebaseFeatureGuard>
);
