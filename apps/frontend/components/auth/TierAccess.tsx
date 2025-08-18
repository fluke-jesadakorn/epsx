import { ReactNode } from 'react';
import { getCurrentUser } from '@/lib/server-actions';

interface TierAccessProps {
  requiredTier: string;
  children: ReactNode;
  fallback?: ReactNode;
  showTierInfo?: boolean;
}

/**
 * Server-side tier access component
 * Replaces client-side TierGuard with server-only validation
 */
export async function TierAccess({
  requiredTier,
  children,
  fallback,
  showTierInfo = false,
}: TierAccessProps) {
  let user = null;
  try {
    const result = await getCurrentUser({});
    user = result?.success ? result.data : null;
  } catch (error) {
    console.error('TierAccess: Failed to get user:', error);
  }
  
  if (!user) {
    return fallback || <div>Authentication required</div>;
  }
  
  const tierHierarchy = {
    'free': 0,
    'bronze': 1,
    'silver': 2,
    'gold': 3,
    'platinum': 4,
    'enterprise': 5,
  };
  
  const userLevel = tierHierarchy[user.package_tier.toLowerCase() as keyof typeof tierHierarchy] || 0;
  const requiredLevel = tierHierarchy[requiredTier.toLowerCase() as keyof typeof tierHierarchy] || 0;
  
  const hasAccess = userLevel >= requiredLevel;
  
  if (!hasAccess) {
    if (fallback) {
      return <>{fallback}</>;
    }
    
    return (
      <div className="bg-gradient-to-r from-purple-50 to-blue-50 dark:from-purple-950/20 dark:to-blue-950/20 border border-purple-200 dark:border-purple-800 rounded-lg p-6 text-center">
        <div className="text-purple-800 dark:text-purple-200">
          <h3 className="font-semibold mb-2">Subscription Required</h3>
          <p className="text-sm mb-2">
            This feature requires <span className="font-medium capitalize">{requiredTier}</span> tier or higher.
          </p>
          <p className="text-xs text-purple-600 dark:text-purple-400 mb-4">
            Your current tier: <span className="font-medium capitalize">{user.package_tier}</span>
          </p>
          <button className="bg-purple-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-purple-700 transition-colors">
            Upgrade to {requiredTier.charAt(0).toUpperCase() + requiredTier.slice(1)}
          </button>
        </div>
      </div>
    );
  }
  
  if (showTierInfo) {
    return (
      <div className="relative">
        <div className="absolute top-0 right-0 bg-green-100 dark:bg-green-900/20 text-green-800 dark:text-green-200 text-xs px-2 py-1 rounded">
          {user.package_tier.charAt(0).toUpperCase() + user.package_tier.slice(1)} Tier
        </div>
        {children}
      </div>
    );
  }
  
  return <>{children}</>;
}

/**
 * Premium tier wrapper (Gold or higher)
 */
export async function PremiumTierAccess({ children }: { children: ReactNode }) {
  return (
    <TierAccess requiredTier="gold">
      {children}
    </TierAccess>
  );
}

/**
 * Enterprise tier wrapper
 */
export async function EnterpriseTierAccess({ children }: { children: ReactNode }) {
  return (
    <TierAccess 
      requiredTier="enterprise"
      fallback={
        <div className="bg-slate-100 dark:bg-slate-900/50 border border-slate-300 dark:border-slate-700 rounded-lg p-8 text-center">
          <div className="text-slate-700 dark:text-slate-300">
            <h3 className="text-xl font-bold mb-3">Enterprise Feature</h3>
            <p className="mb-4">This advanced feature is available to Enterprise customers only.</p>
            <div className="space-y-2 text-sm mb-6">
              <div>• Advanced analytics and reporting</div>
              <div>• Priority support</div>
              <div>• Custom integrations</div>
              <div>• Dedicated account manager</div>
            </div>
            <button className="bg-slate-800 dark:bg-slate-200 text-white dark:text-slate-800 px-6 py-3 rounded-md font-medium hover:bg-slate-700 dark:hover:bg-slate-100 transition-colors">
              Contact Sales
            </button>
          </div>
        </div>
      }
    >
      {children}
    </TierAccess>
  );
}