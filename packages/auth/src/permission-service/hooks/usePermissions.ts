import { useEffect, useState, useMemo } from 'react';
// import { useAuth } from '@/context/auth-context-improved';
import type { 
  PermissionContext, 
  PermissionEvaluationResult,
  EvaluationOptions 
} from '../types';
import { PermissionService } from '../PermissionService';
import { ResourceType, ActionType } from '../types';

// Mock auth hook for now - replace with actual implementation
const useAuth = () => ({
  user: {
    uid: 'user-123',
    email: 'user@example.com',
  },
});

interface UsePermissionOptions extends EvaluationOptions {
  enabled?: boolean;
  cacheTime?: number;
  additionalContext?: Record<string, any>;
}

interface PermissionHookResult {
  allowed: boolean;
  loading: boolean;
  error: string | null;
  result: PermissionEvaluationResult | null;
  refetch: () => void;
}

/**
 * Hook for checking a single permission
 */
export function usePermission(
  resource: string,
  action: string,
  options: UsePermissionOptions = {}
): PermissionHookResult {
  const { user } = useAuth();
  const [result, setResult] = useState<PermissionEvaluationResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const permissionService = useMemo(() => PermissionService.getInstance(), []);
  const { enabled = true, ...evaluationOptions } = options;

  const checkPermission = async () => {
    if (!user || !enabled) return;

    setLoading(true);
    setError(null);

    try {
      const context: PermissionContext = {
        userId: user.uid,
        requestId: `${Date.now()}-${Math.random()}`,
        timestamp: new Date(),
        resource,
        action,
        additionalContext: {
          userEmail: user.email,
          // Add any additional context here
        },
      };

      const evaluationResult = await permissionService.evaluatePermission(
        context,
        evaluationOptions
      );

      setResult(evaluationResult);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Permission check failed');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    checkPermission();
  }, [user, resource, action, enabled]);

  return {
    allowed: result?.allowed ?? false,
    loading,
    error,
    result,
    refetch: checkPermission,
  };
}

/**
 * Hook for checking multiple permissions
 */
export function usePermissions(
  permissions: Array<{ resource: string; action: string }>,
  options: UsePermissionOptions = {}
): {
  permissions: Record<string, PermissionHookResult>;
  allAllowed: boolean;
  anyAllowed: boolean;
  loading: boolean;
} {
  const results: Record<string, PermissionHookResult> = {};
  
  permissions.forEach(({ resource, action }) => {
    const key = `${resource}:${action}`;
    // eslint-disable-next-line react-hooks/rules-of-hooks
    results[key] = usePermission(resource, action, options);
  });

  const allAllowed = Object.values(results).every(r => r.allowed);
  const anyAllowed = Object.values(results).some(r => r.allowed);
  const loading = Object.values(results).some(r => r.loading);

  return {
    permissions: results,
    allAllowed,
    anyAllowed,
    loading,
  };
}

/**
 * Hook for stock analytics permissions
 */
export function useStockAnalyticsPermissions(limit?: number) {
  const rankingsPermission = usePermission(
    `epsx:stock:::${ResourceType.STOCK_RANKINGS}:*`,
    ActionType.READ,
    {
      additionalContext: {
        requestedLimit: limit,
      },
    }
  );

  const analyticsPermission = usePermission(
    `epsx:stock:::${ResourceType.STOCK_ANALYTICS}:*`,
    ActionType.ANALYZE
  );

  const screenerPermission = usePermission(
    `epsx:stock:::${ResourceType.STOCK_SCREENER}:*`,
    ActionType.SCREEN
  );

  const exportPermission = usePermission(
    `epsx:stock:::${ResourceType.STOCK_RANKINGS}:*`,
    ActionType.EXPORT
  );

  return {
    canViewRankings: rankingsPermission.allowed,
    canAnalyze: analyticsPermission.allowed,
    canScreen: screenerPermission.allowed,
    canExport: exportPermission.allowed,
    loading: rankingsPermission.loading || analyticsPermission.loading || 
             screenerPermission.loading || exportPermission.loading,
    results: {
      rankings: rankingsPermission.result,
      analytics: analyticsPermission.result,
      screener: screenerPermission.result,
      export: exportPermission.result,
    },
  };
}

/**
 * Hook for admin permissions
 */
export function useAdminPermissions() {
  const userManagementPermission = usePermission(
    `epsx:admin:::${ResourceType.ADMIN_USERS}:*`,
    ActionType.MANAGE
  );

  const systemManagementPermission = usePermission(
    `epsx:admin:::${ResourceType.ADMIN_SYSTEM}:*`,
    ActionType.CONFIGURE
  );

  const analyticsPermission = usePermission(
    `epsx:admin:::${ResourceType.ADMIN_ANALYTICS}:*`,
    ActionType.VIEW
  );

  return {
    canManageUsers: userManagementPermission.allowed,
    canConfigureSystem: systemManagementPermission.allowed,
    canViewAnalytics: analyticsPermission.allowed,
    isAdmin: userManagementPermission.allowed || systemManagementPermission.allowed,
    loading: userManagementPermission.loading || systemManagementPermission.loading || 
             analyticsPermission.loading,
  };
}

/**
 * Hook for billing permissions
 */
export function useBillingPermissions() {
  const { user } = useAuth();
  
  const paymentHistoryPermission = usePermission(
    `epsx:billing:::${ResourceType.PAYMENT_HISTORY}:${user?.uid}`,
    ActionType.READ
  );

  const paymentMethodsPermission = usePermission(
    `epsx:billing:::${ResourceType.PAYMENT_METHODS}:${user?.uid}`,
    ActionType.UPDATE
  );

  const subscriptionPermission = usePermission(
    `epsx:user:::${ResourceType.USER_SUBSCRIPTION}:${user?.uid}`,
    ActionType.UPDATE
  );

  return {
    canViewPaymentHistory: paymentHistoryPermission.allowed,
    canManagePaymentMethods: paymentMethodsPermission.allowed,
    canManageSubscription: subscriptionPermission.allowed,
    loading: paymentHistoryPermission.loading || paymentMethodsPermission.loading || 
             subscriptionPermission.loading,
  };
}

/**
 * Hook for resource-specific permissions
 */
export function useResourcePermissions(resourceArn: string) {
  const readPermission = usePermission(resourceArn, ActionType.READ);
  const updatePermission = usePermission(resourceArn, ActionType.UPDATE);
  const deletePermission = usePermission(resourceArn, ActionType.DELETE);
  const createPermission = usePermission(resourceArn, ActionType.CREATE);

  return {
    canRead: readPermission.allowed,
    canUpdate: updatePermission.allowed,
    canDelete: deletePermission.allowed,
    canCreate: createPermission.allowed,
    loading: readPermission.loading || updatePermission.loading || 
             deletePermission.loading || createPermission.loading,
    results: {
      read: readPermission.result,
      update: updatePermission.result,
      delete: deletePermission.result,
      create: createPermission.result,
    },
  };
}

/**
 * Hook for tier-based access
 */
export function useTierAccess(tier: 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM') {
  const limits = useMemo(() => {
    switch (tier) {
      case 'BRONZE':
        return { stockRankings: 5, apiCalls: 100 };
      case 'SILVER':
        return { stockRankings: 25, apiCalls: 500 };
      case 'GOLD':
        return { stockRankings: 50, apiCalls: 1000 };
      case 'PLATINUM':
        return { stockRankings: 100, apiCalls: 5000 };
      default:
        return { stockRankings: 5, apiCalls: 100 };
    }
  }, [tier]);

  const stockAccess = useStockAnalyticsPermissions(limits.stockRankings);

  return {
    tier,
    limits,
    stockAccess,
    canAccessRankings: (requestedLimit: number) => requestedLimit <= limits.stockRankings,
    canMakeApiCalls: (requestedCalls: number) => requestedCalls <= limits.apiCalls,
  };
}

export default {
  usePermission,
  usePermissions,
  useStockAnalyticsPermissions,
  useAdminPermissions,
  useBillingPermissions,
  useResourcePermissions,
  useTierAccess,
};
