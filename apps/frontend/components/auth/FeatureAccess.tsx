import { ReactNode } from 'react';
import { checkFeatureAccess } from '@epsx/server-actions';
import { AccessDenied } from './AccessDenied';

interface FeatureAccessProps {
  feature: string;
  action?: string;
  children: ReactNode;
  fallback?: ReactNode;
  showLimits?: boolean;
}

/**
 * Server-side feature access component
 * Replaces client-side FeatureGuard with server-only validation
 */
export async function FeatureAccess({
  feature,
  action = 'read',
  children,
  fallback,
  showLimits = false,
}: FeatureAccessProps) {
  let access = { allowed: false, reason: 'Access check failed' };
  try {
    const result = await checkFeatureAccess(feature);
    access = result?.success ? result.data : access;
  } catch (error) {
    console.error('FeatureAccess: Failed to check feature access:', error);
  }
  
  if (!access.allowed) {
    if (fallback) {
      return <>{fallback}</>;
    }
    
    return (
      <AccessDenied
        reason={access.reason || `Access denied for feature: ${feature}`}
        requiredPermissions={[`${feature}:${action}`]}
      />
    );
  }
  
  // Wrap children with limits data if available
  if (showLimits && access.limits) {
    return (
      <div data-feature-limits={JSON.stringify(access.limits)}>
        {children}
      </div>
    );
  }
  
  return <>{children}</>;
}

/**
 * Simple analytics feature wrapper
 */
export async function AnalyticsAccess({ children }: { children: ReactNode }) {
  return (
    <FeatureAccess feature="analytics" showLimits>
      {children}
    </FeatureAccess>
  );
}

/**
 * Trading feature wrapper with quota display
 */
export async function TradingAccess({ children }: { children: ReactNode }) {
  return (
    <FeatureAccess feature="trading" showLimits>
      {children}
    </FeatureAccess>
  );
}

/**
 * Premium features wrapper
 */
export async function PremiumAccess({ 
  children, 
  feature 
}: { 
  children: ReactNode;
  feature: string;
}) {
  return (
    <FeatureAccess 
      feature={feature} 
      fallback={
        <div className="bg-amber-50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-800 rounded-lg p-6 text-center">
          <div className="text-amber-800 dark:text-amber-200">
            <h3 className="font-semibold mb-2">Premium Feature</h3>
            <p className="text-sm mb-4">This feature requires a premium subscription.</p>
            <button className="bg-amber-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-amber-700">
              Upgrade Now
            </button>
          </div>
        </div>
      }
    >
      {children}
    </FeatureAccess>
  );
}

/**
 * API access wrapper with rate limiting info
 */
export async function ApiAccess({ children }: { children: ReactNode }) {
  let access = { allowed: false, reason: 'API access check failed' };
  try {
    const result = await checkFeatureAccess('api');
    access = result?.success ? result.data : access;
  } catch (error) {
    console.error('ApiAccess: Failed to check API access:', error);
  }
  
  if (!access.allowed) {
    return (
      <div className="bg-red-50 dark:bg-red-950/20 border border-red-200 dark:border-red-800 rounded-lg p-6 text-center">
        <div className="text-red-800 dark:text-red-200">
          <h3 className="font-semibold mb-2">API Access Required</h3>
          <p className="text-sm mb-4">{access.reason || 'You need API access to use this feature.'}</p>
          <button className="bg-red-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-red-700">
            Request API Access
          </button>
        </div>
      </div>
    );
  }
  
  return (
    <div className="relative">
      {access.limits?.rate_limit && (
        <div className="absolute top-0 right-0 bg-blue-100 dark:bg-blue-900/20 text-blue-800 dark:text-blue-200 text-xs px-2 py-1 rounded">
          Rate Limit: {access.limits.rate_limit}/min
        </div>
      )}
      {children}
    </div>
  );
}