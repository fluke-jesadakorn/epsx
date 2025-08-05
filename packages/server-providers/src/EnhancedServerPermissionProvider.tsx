import { ErrorHandler, type Result } from '@epsx/shared-core';
import type { 
  PaymentPlan
} from '@epsx/types';
import { 
  getUserPermissions, 
  getPaymentStatus, 
  getPlanDetails, 
  checkFeatureAccess, 
  checkRankingAccess,
  type UserPermission,
  type PaymentStatus as ServerPaymentStatus
} from '@epsx/server-actions';

// Enhanced server data structure
export interface EnhancedServerData {
  permissions: UserPermission[] | null;
  paymentStatus: ServerPaymentStatus | null;
  plans: PaymentPlan[] | null;
  featureAccess: Record<string, boolean>;
  rankingAccess: { allowed: boolean; tier: string; expiresAt?: string };
  error: string | null;
  timestamp: Date;
}

// Data fetchers
async function getPermissionsData(): Promise<Result<UserPermission[]>> {
  console.debug('Fetching user permissions', { component: 'EnhancedServerPermissionProvider' });

  return ErrorHandler.withErrorHandling(async () => {
    const result = await getUserPermissions();
    return result;
  });
}

async function getPaymentData(): Promise<Result<ServerPaymentStatus | null>> {
  console.debug('Fetching payment status', { component: 'EnhancedServerPermissionProvider' });

  return ErrorHandler.withErrorHandling(async () => {
    const result = await getPaymentStatus('');
    if (result && typeof result === 'object' && 'success' in result) {
      return result.success ? result.data : null;
    }
    return result as ServerPaymentStatus | null;
  });
}

async function getPlansData(): Promise<Result<PaymentPlan[]>> {
  console.debug('Fetching payment plans', { component: 'EnhancedServerPermissionProvider' });

  return ErrorHandler.withErrorHandling(async () => {
    const result = await getPlanDetails('');
    if (result && typeof result === 'object' && 'success' in result) {
      return result.success ? result.data : [];
    }
    return result || [];
  });
}

async function getFeatureAccessData(features: string[]): Promise<Result<Record<string, boolean>>> {
  console.debug('Fetching feature access', { features, component: 'EnhancedServerPermissionProvider' });

  return ErrorHandler.withErrorHandling(async () => {
    const featureAccess: Record<string, boolean> = {};
    
    // Check each feature in parallel
    const results = await Promise.allSettled(
      features.map(async (feature) => {
        const result = await checkFeatureAccess(feature);
        return { feature, result };
      })
    );

    results.forEach((result, index) => {
      const feature = features[index];
      if (feature) {
        if (result.status === 'fulfilled') {
          const accessResult = result.value.result;
          featureAccess[feature] = accessResult && typeof accessResult === 'object' && 'allowed' in accessResult 
            ? Boolean(accessResult.allowed)
            : Boolean(accessResult);
        } else {
          featureAccess[feature] = false;
          console.warn(`Failed to check feature access for ${feature}`, {
            error: result.reason
          });
        }
      }
    });

    return featureAccess;
  });
}

async function getRankingAccessData(): Promise<Result<{ allowed: boolean; tier: string; expiresAt?: string }>> {
  console.debug('Fetching ranking access', { component: 'EnhancedServerPermissionProvider' });

  return ErrorHandler.withErrorHandling(async () => {
    const result = await checkRankingAccess();
    return result;
  });
}

/**
 * Enhanced server-side data fetcher with comprehensive error handling
 */
export async function getEnhancedPermissionData(options: {
  includePermissions?: boolean;
  includePayment?: boolean;
  includePlans?: boolean;
  includeFeatures?: string[];
  includeRanking?: boolean;
} = {}): Promise<EnhancedServerData> {
  const {
    includePermissions = true,
    includePayment = true,
    includePlans = false,
    includeFeatures = [],
    includeRanking = true
  } = options;

  console.info('Fetching enhanced permission data', { options, component: 'EnhancedServerPermissionProvider' });

  const data: EnhancedServerData = {
    permissions: null,
    paymentStatus: null,
    plans: null,
    featureAccess: {},
    rankingAccess: { allowed: false, tier: 'BRONZE' },
    error: null,
    timestamp: new Date()
  };

  const errors: string[] = [];

  try {
    // Fetch data in parallel
    interface PromiseResult {
      type: string;
      result: any;
    }
    const promises: Promise<PromiseResult>[] = [];
    
    if (includePermissions) {
      promises.push(getPermissionsData().then((result: any) => ({ type: 'permissions', result })));
    }
    
    if (includePayment) {
      promises.push(getPaymentData().then((result: any) => ({ type: 'payment', result })));
    }
    
    if (includePlans) {
      promises.push(getPlansData().then((result: any) => ({ type: 'plans', result })));
    }
    
    if (includeFeatures.length > 0) {
      promises.push(getFeatureAccessData(includeFeatures).then((result: any) => ({ type: 'features', result })));
    }
    
    if (includeRanking) {
      promises.push(getRankingAccessData().then((result: any) => ({ type: 'ranking', result })));
    }

    const results = await Promise.allSettled(promises);

    // Process results
    results.forEach((promiseResult, index) => {
      if (promiseResult.status === 'fulfilled') {
        const { type, result } = promiseResult.value;
        
        if (result.success) {
          switch (type) {
            case 'permissions':
              data.permissions = result.data;
              break;
            case 'payment':
              data.paymentStatus = result.data;
              break;
            case 'plans':
              data.plans = result.data;
              break;
            case 'features':
              data.featureAccess = result.data;
              break;
            case 'ranking':
              data.rankingAccess = result.data;
              break;
          }
        } else {
          const errorMsg = `Failed to fetch ${type}: ${result.error?.message || 'Unknown error'}`;
          errors.push(errorMsg);
          console.warn(errorMsg, { error: result.error });
        }
      } else {
        const errorMsg = `Promise rejected: ${promiseResult.reason}`;
        errors.push(errorMsg);
        console.error(errorMsg, { reason: promiseResult.reason });
      }
    });

  } catch (error) {
    const errorMsg = `Unexpected error in getEnhancedPermissionData: ${error instanceof Error ? error.message : error}`;
    errors.push(errorMsg);
    console.error(errorMsg, { error });
  }

  // Set combined error if any occurred
  if (errors.length > 0) {
    data.error = errors.join('; ');
  }

  console.info('Enhanced permission data fetched', {
    hasPermissions: !!data.permissions,
    hasPaymentStatus: !!data.paymentStatus,
    hasPlans: !!data.plans,
    featureCount: Object.keys(data.featureAccess).length,
    hasRankingAccess: data.rankingAccess,
    errorCount: errors.length,
    component: 'EnhancedServerPermissionProvider'
  });

  return data;
}

/**
 * Convenience function for getting basic permission data
 */
export async function getBasicPermissionData(): Promise<EnhancedServerData> {
  return getEnhancedPermissionData({
    includePermissions: true,
    includePayment: true,
    includePlans: false,
    includeFeatures: [],
    includeRanking: true
  });
}

/**
 * Convenience function for getting comprehensive data
 */
export async function getComprehensivePermissionData(features: string[] = []): Promise<EnhancedServerData> {
  return getEnhancedPermissionData({
    includePermissions: true,
    includePayment: true,
    includePlans: true,
    includeFeatures: features,
    includeRanking: true
  });
}

/**
 * Type guard to check if server data is valid
 */
export function isValidServerData(data: EnhancedServerData): boolean {
  return !data.error && (
    data.permissions !== null ||
    data.paymentStatus !== null ||
    data.plans !== null ||
    Object.keys(data.featureAccess).length > 0
  );
}

/**
 * Get cache info for debugging
 */
export function getCacheStats() {
  return {
    timestamp: new Date(),
    message: 'Cache stats not available in this React version'
  };
}