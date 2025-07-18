import React, { type ReactNode } from 'react';
import { usePermission, usePermissions } from '../hooks/usePermissions';
import type { PermissionEvaluationResult } from '../types';

interface PermissionGateProps {
  resource: string;
  action: string;
  children: ReactNode;
  fallback?: ReactNode;
  loadingComponent?: ReactNode;
  onPermissionDenied?: (result: PermissionEvaluationResult) => void;
  showReason?: boolean;
}

/**
 * Permission gate component that conditionally renders children based on permissions
 */
export const PermissionGate: React.FC<PermissionGateProps> = ({
  resource,
  action,
  children,
  fallback,
  loadingComponent,
  onPermissionDenied,
  showReason = false,
}) => {
  const { allowed, loading, result } = usePermission(resource, action);

  if (loading) {
    return loadingComponent || <div>Loading permissions...</div>;
  }

  if (!allowed) {
    if (onPermissionDenied && result) {
      onPermissionDenied(result);
    }
    
    if (fallback) {
      return <>{fallback}</>;
    }

    if (showReason && result) {
      return (
        <div className="permission-denied">
          <h3>Access Denied</h3>
          <p>{result.reason}</p>
        </div>
      );
    }

    return null;
  }

  return <>{children}</>;
};

interface MultiPermissionGateProps {
  permissions: Array<{ resource: string; action: string }>;
  children: ReactNode;
  fallback?: ReactNode;
  loadingComponent?: ReactNode;
  requireAll?: boolean; // true = AND logic, false = OR logic
  showReason?: boolean;
}

/**
 * Multi-permission gate that checks multiple permissions
 */
export const MultiPermissionGate: React.FC<MultiPermissionGateProps> = ({
  permissions,
  children,
  fallback,
  loadingComponent,
  requireAll = true,
  showReason = false,
}) => {
  const { allAllowed, anyAllowed, loading, permissions: results } = usePermissions(permissions);

  if (loading) {
    return loadingComponent || <div>Loading permissions...</div>;
  }

  const hasAccess = requireAll ? allAllowed : anyAllowed;

  if (!hasAccess) {
    if (fallback) {
      return <>{fallback}</>;
    }

    if (showReason) {
      const deniedPermissions = Object.entries(results)
        .filter(([_, result]) => !result.allowed)
        .map(([key, result]) => ({ key, reason: result.result?.reason || 'Access denied' }));

      return (
        <div className="permission-denied">
          <h3>Access Denied</h3>
          <ul>
            {deniedPermissions.map(({ key, reason }) => (
              <li key={key}>{key}: {reason}</li>
            ))}
          </ul>
        </div>
      );
    }

    return null;
  }

  return <>{children}</>;
};

interface TierGateProps {
  requiredTier: 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM';
  children: ReactNode;
  fallback?: ReactNode;
  upgradePrompt?: ReactNode;
}

/**
 * Tier-based access gate
 */
export const TierGate: React.FC<TierGateProps> = ({
  requiredTier,
  children,
  fallback,
  upgradePrompt,
}) => {
  // This would integrate with your existing tier system
  const userTier = 'BRONZE'; // Replace with actual user tier logic
  
  const tierHierarchy = {
    BRONZE: 1,
    SILVER: 2,
    GOLD: 3,
    PLATINUM: 4,
  };

  const hasAccess = tierHierarchy[userTier] >= tierHierarchy[requiredTier];

  if (!hasAccess) {
    if (upgradePrompt) {
      return <>{upgradePrompt}</>;
    }
    
    if (fallback) {
      return <>{fallback}</>;
    }

    return (
      <div className="tier-gate-denied">
        <h3>Upgrade Required</h3>
        <p>This feature requires {requiredTier} tier or higher.</p>
        <p>Your current tier: {userTier}</p>
      </div>
    );
  }

  return <>{children}</>;
};

interface AdminGateProps {
  children: ReactNode;
  fallback?: ReactNode;
  requiredRole?: 'ADMIN' | 'SUPER_ADMIN';
}

/**
 * Admin-only access gate
 */
export const AdminGate: React.FC<AdminGateProps> = ({
  children,
  fallback,
  requiredRole: _requiredRole = 'ADMIN',
}) => {
  const { allowed, loading } = usePermission(
    'epsx:admin:::admin_users:*',
    'manage'
  );

  if (loading) {
    return <div>Loading admin permissions...</div>;
  }

  if (!allowed) {
    return fallback || <div>Admin access required</div>;
  }

  return <>{children}</>;
};

interface ConditionalPermissionProps {
  condition: boolean;
  resource: string;
  action: string;
  children: ReactNode;
  fallback?: ReactNode;
}

/**
 * Conditional permission gate - only checks permission if condition is true
 */
export const ConditionalPermission: React.FC<ConditionalPermissionProps> = ({
  condition,
  resource,
  action,
  children,
  fallback,
}) => {
  const { allowed, loading } = usePermission(resource, action, {
    enabled: condition,
  });

  if (!condition) {
    return <>{children}</>;
  }

  if (loading) {
    return <div>Loading permissions...</div>;
  }

  if (!allowed) {
    return fallback || null;
  }

  return <>{children}</>;
};

// Higher-order component for permission checking
export function withPermission<P extends object>(
  Component: React.ComponentType<P>,
  resource: string,
  action: string,
  fallback?: ReactNode
) {
  return function PermissionWrappedComponent(props: P) {
    return (
      <PermissionGate
        resource={resource}
        action={action}
        fallback={fallback}
      >
        <Component {...props} />
      </PermissionGate>
    );
  };
}

// Higher-order component for tier checking
export function withTierAccess<P extends object>(
  Component: React.ComponentType<P>,
  requiredTier: 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM',
  fallback?: ReactNode
) {
  return function TierWrappedComponent(props: P) {
    return (
      <TierGate
        requiredTier={requiredTier}
        fallback={fallback}
      >
        <Component {...props} />
      </TierGate>
    );
  };
}

// Export all components
export default {
  PermissionGate,
  MultiPermissionGate,
  TierGate,
  AdminGate,
  ConditionalPermission,
  withPermission,
  withTierAccess,
};
