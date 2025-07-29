import { 
  getPaymentStatus, 
  getUserPermissions, 
  checkFeatureAccess,
  checkRankingAccess 
} from '@epsx/server-actions';

interface ServerPermissionProviderProps {
  children: React.ReactNode;
}

interface PermissionServerData {
  paymentStatus: any;
  permissions: any;
  featureAccess: any;
  rankingAccess: any;
  error: string | null;
}

export async function getPermissionData(): Promise<PermissionServerData> {
  let paymentStatus = null;
  let permissions = null;
  let featureAccess = null;
  let rankingAccess = null;
  let error: string | null = null;

  try {
    // Fetch all permission-related data server-side
    const [
      paymentResult,
      permissionsResult,
      tradingFeatureResult,
      analyticsFeatureResult,
      rankingResult
    ] = await Promise.allSettled([
      getPaymentStatus(),
      getUserPermissions(),
      checkFeatureAccess('trading'),
      checkFeatureAccess('analytics'),
      checkRankingAccess()
    ]);

    if (paymentResult.status === 'fulfilled') {
      paymentStatus = paymentResult.value;
    }
    if (permissionsResult.status === 'fulfilled') {
      permissions = permissionsResult.value;
    }
    if (tradingFeatureResult.status === 'fulfilled' && analyticsFeatureResult.status === 'fulfilled') {
      featureAccess = {
        trading: tradingFeatureResult.value,
        analytics: analyticsFeatureResult.value
      };
    }
    if (rankingResult.status === 'fulfilled') {
      rankingAccess = rankingResult.value;
    }
  } catch (err) {
    console.error('Failed to fetch permissions server-side:', err);
    error = 'Failed to load permissions';
  }

  return {
    paymentStatus,
    permissions,
    featureAccess,
    rankingAccess,
    error
  };
}