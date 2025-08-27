'use client';

import React from 'react';
import { useRouter } from 'next/navigation';
import { Role, checkFeatureAccess } from '@/types/permissions';
import { FeatureErrorBoundary } from './FeatureErrorBoundary';

interface SimpleFeatureGateProps {
  feature: string;
  children: React.ReactNode;
  fallback?: React.ReactNode;
  userRole?: Role;
}

interface UpgradeCardProps {
  feature: string;
  currentRole: Role;
  missingFeatures: string[];
}

const UpgradeCard: React.FC<UpgradeCardProps> = ({
  feature,
  currentRole,
  missingFeatures
}) => {
  const router = useRouter();
  
  const getRequiredRole = (feature: string): Role => {
    // All features except view_eps require user role
    return feature === 'view_eps' ? Role.Guest : Role.User;
  };

  const requiredRole = getRequiredRole(feature);

  return (
    <div className="p-4 border rounded-lg bg-gray-50 dark:bg-gray-800">
      <h3 className="text-lg font-semibold mb-2">Feature Locked</h3>
      <p className="text-sm text-gray-600 dark:text-gray-300 mb-4">
        You need to upgrade your account to access this feature.
      </p>
      <div className="space-y-2">
        <div className="flex justify-between items-center">
          <span>Current Role:</span>
          <span className="font-medium capitalize">{currentRole}</span>
        </div>
        <div className="flex justify-between items-center">
          <span>Required Role:</span>
          <span className="font-medium capitalize">{requiredRole}</span>
        </div>
        {missingFeatures.length > 0 && (
          <div>
            <span className="text-sm font-medium">Missing Features:</span>
            <ul className="text-sm text-gray-600 dark:text-gray-300 ml-4">
              {missingFeatures.map((feat, index) => (
                <li key={index}>• {feat.replace('_', ' ')}</li>
              ))}
            </ul>
          </div>
        )}
        <div className="mt-4">
          <button
            className="w-full bg-blue-600 text-white rounded-lg px-4 py-2 hover:bg-blue-700 transition-colors"
            onClick={() => router.push('/payment')}
          >
            Upgrade to {requiredRole === Role.User ? 'Premium' : 'Admin'}
          </button>
        </div>
      </div>
    </div>
  );
};

// Mock user data - in real app, this would come from auth context
const getMockUserRole = (): Role => {
  // For demo purposes, return guest. In real app, get from auth context
  return Role.Guest;
};

export const TokenGatedFeature: React.FC<SimpleFeatureGateProps> = ({
  feature,
  children,
  fallback,
  userRole
}) => {
  return (
    <FeatureErrorBoundary feature={feature} fallback={fallback}>
      <SimpleFeatureGateContent 
        feature={feature} 
        fallback={fallback}
        userRole={userRole}
      >
        {children}
      </SimpleFeatureGateContent>
    </FeatureErrorBoundary>
  );
};

const SimpleFeatureGateContent: React.FC<SimpleFeatureGateProps> = ({
  feature,
  children,
  fallback,
  userRole
}) => {
  // Get user role from props or mock for demo
  const currentRole = userRole || getMockUserRole();
  
  // Check if user has access to this feature
  const hasAccess = checkFeatureAccess(currentRole, feature);
  
  if (hasAccess) {
    return <>{children}</>;
  }

  if (fallback) {
    return <>{fallback}</>;
  }

  // Determine what features are missing
  const missingFeatures = [feature];

  return (
    <UpgradeCard
      feature={feature}
      currentRole={currentRole}
      missingFeatures={missingFeatures}
    />
  );
};

// Higher-order component with simple role-based access
export function withFeatureGate<P extends object>(
  WrappedComponent: React.ComponentType<P>,
  feature: string,
  fallback?: React.ReactNode,
) {
  return function WithFeatureGate(props: P) {
    return (
      <TokenGatedFeature feature={feature} fallback={fallback}>
        <WrappedComponent {...props} />
      </TokenGatedFeature>
    );
  };
}

// Convenience components for common features
export const ViewEPSGate: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ children, fallback }) => (
  <TokenGatedFeature feature="view_eps" fallback={fallback}>
    {children}
  </TokenGatedFeature>
);

export const ExportDataGate: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ children, fallback }) => (
  <TokenGatedFeature feature="export_data" fallback={fallback}>
    {children}
  </TokenGatedFeature>
);

export const RealtimeGate: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ children, fallback }) => (
  <TokenGatedFeature feature="realtime" fallback={fallback}>
    {children}
  </TokenGatedFeature>
);

export const AdvancedFiltersGate: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ children, fallback }) => (
  <TokenGatedFeature feature="advanced_filters" fallback={fallback}>
    {children}
  </TokenGatedFeature>
);

export const AdminGate: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ children, fallback }) => {
  const currentRole = getMockUserRole();
  const hasAdminAccess = currentRole === Role.Admin;
  
  if (hasAdminAccess) {
    return <>{children}</>;
  }
  
  if (fallback) {
    return <>{fallback}</>;
  }
  
  return (
    <div className="p-4 border rounded-lg bg-red-50 dark:bg-red-900/20">
      <h3 className="text-lg font-semibold mb-2 text-red-800 dark:text-red-200">Admin Access Required</h3>
      <p className="text-sm text-red-600 dark:text-red-300">
        You need administrator privileges to access this feature.
      </p>
    </div>
  );
};

// Example usage:
// <ViewEPSGate fallback={<div>Please upgrade to view EPS data</div>}>
//   <EPSAnalyticsComponent />
// </ViewEPSGate>
//
// <ExportDataGate>
//   <ExportButton />
// </ExportDataGate>
//
// <AdminGate fallback={<div>Admin only</div>}>
//   <AdminPanel />
// </AdminGate>