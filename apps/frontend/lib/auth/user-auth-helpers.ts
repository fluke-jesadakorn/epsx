/**
 * User Authentication Helpers using Separated Types
 * Performance-optimized functions for user context
 */

'use client';

import {
  UserSessionData,
  UserProfile,
  UserJWTPayload,
  PackageTier,
  UserPermissionCheck,
  PermissionValidation,
  UserAnalyticsAccess,
  UserTradingAccess,
  hasValidSubscription,
  canAccessFeature,
  isPremiumTier,
  isTrialUser
} from '@/types/auth-separation';

// ============================================================================
// Permission Validation (User-Specific)
// ============================================================================

/**
 * Check if user has specific permission with subscription validation
 */
export function hasUserPermission(
  user: UserProfile | null | undefined,
  permission: string,
  requiresSubscription: boolean = false
): PermissionValidation {
  if (!user?.permissions) {
    return {
      hasPermission: false,
      reason: 'no_permission'
    };
  }
  
  const permissions = Array.isArray(user.permissions) ? user.permissions : [];
  
  // Check for user wildcard permission
  if (permissions.includes('epsx:*:*')) {
    return { hasPermission: true, reason: 'valid' };
  }
  
  // Check for exact permission match
  if (permissions.includes(permission)) {
    return { hasPermission: true, reason: 'valid' };
  }
  
  // Check for broader permissions
  if (permission.includes(':')) {
    const [platform, resource] = permission.split(':');
    const hasMatchingPattern = permissions.some(p => 
      p === `${platform}:${resource}:*` || 
      p === `${platform}:*:*`
    );
    
    if (hasMatchingPattern) {
      // Check subscription requirement
      if (requiresSubscription && !hasValidSubscription(user)) {
        return {
          hasPermission: false,
          reason: 'tier_insufficient',
          requiredTier: PackageTier.BRONZE,
          upgradeUrl: '/subscription/upgrade'
        };
      }
      
      return { hasPermission: true, reason: 'valid' };
    }
  }
  
  return {
    hasPermission: false,
    reason: 'no_permission'
  };
}

/**
 * Check if user has access to analytics features
 */
export function getUserAnalyticsAccess(user: UserProfile | null | undefined): UserAnalyticsAccess {
  if (!user) {
    return {
      canViewRankings: false,
      canExportData: false,
      maxStocksTracked: 0,
      realTimeAccess: false,
      advancedFilters: false,
      customIndicators: false
    };
  }
  
  const tier = user.packageTier as PackageTier;
  
  return {
    canViewRankings: true,
    canExportData: isPremiumTier(tier),
    maxStocksTracked: getMaxStocksForTier(tier),
    realTimeAccess: tier !== PackageTier.FREE,
    advancedFilters: isPremiumTier(tier),
    customIndicators: tier === PackageTier.PLATINUM || tier === PackageTier.ENTERPRISE
  };
}

/**
 * Check if user has access to trading features
 */
export function getUserTradingAccess(user: UserProfile | null | undefined): UserTradingAccess {
  if (!user) {
    return {
      paperTrading: false,
      liveTrading: false,
      advancedOrders: false,
      algorithmicTrading: false,
      portfolioAnalysis: false,
      riskManagement: false
    };
  }
  
  const tier = user.packageTier as PackageTier;
  
  return {
    paperTrading: tier !== PackageTier.FREE,
    liveTrading: isPremiumTier(tier),
    advancedOrders: tier === PackageTier.GOLD || tier === PackageTier.PLATINUM || tier === PackageTier.ENTERPRISE,
    algorithmicTrading: tier === PackageTier.PLATINUM || tier === PackageTier.ENTERPRISE,
    portfolioAnalysis: tier !== PackageTier.FREE,
    riskManagement: isPremiumTier(tier)
  };
}

// ============================================================================
// Package Tier Utilities
// ============================================================================

/**
 * Get maximum stocks tracked for package tier
 */
function getMaxStocksForTier(tier: PackageTier): number {
  const limits = {
    [PackageTier.FREE]: 10,
    [PackageTier.BRONZE]: 50,
    [PackageTier.SILVER]: 100,
    [PackageTier.GOLD]: 500,
    [PackageTier.PLATINUM]: 1000,
    [PackageTier.ENTERPRISE]: -1 // unlimited
  };
  
  return limits[tier] || 0;
}

/**
 * Check if user can upgrade to specific tier
 */
export function canUpgradeToTier(
  user: UserProfile | null | undefined,
  targetTier: PackageTier
): boolean {
  if (!user) return true;
  
  const tierHierarchy = {
    [PackageTier.FREE]: 0,
    [PackageTier.BRONZE]: 1,
    [PackageTier.SILVER]: 2,
    [PackageTier.GOLD]: 3,
    [PackageTier.PLATINUM]: 4,
    [PackageTier.ENTERPRISE]: 5
  };
  
  const currentLevel = tierHierarchy[user.packageTier as PackageTier] || 0;
  const targetLevel = tierHierarchy[targetTier] || 0;
  
  return targetLevel > currentLevel;
}

/**
 * Get upgrade URL for feature
 */
export function getUpgradeUrlForFeature(feature: string): string {
  const featureMapping: Record<string, string> = {
    'export_data': '/subscription/upgrade?feature=export',
    'real_time': '/subscription/upgrade?feature=realtime',
    'advanced_analytics': '/subscription/upgrade?feature=analytics',
    'live_trading': '/subscription/upgrade?feature=trading',
    'algorithmic_trading': '/subscription/upgrade?feature=algo'
  };
  
  return featureMapping[feature] || '/subscription/upgrade';
}

// ============================================================================
// Feature Access Validation
// ============================================================================

/**
 * Validate feature access with detailed response
 */
export function validateFeatureAccess(
  user: UserProfile | null | undefined,
  feature: string
): PermissionValidation {
  if (!user) {
    return {
      hasPermission: false,
      reason: 'no_permission',
      upgradeUrl: '/auth/login'
    };
  }
  
  // Check permission
  const permissionResult = hasUserPermission(user, `epsx:${feature}:access`);
  if (permissionResult.hasPermission) {
    return permissionResult;
  }
  
  // Check if feature requires specific tier
  const requiredTier = getRequiredTierForFeature(feature);
  const currentTier = user.packageTier as PackageTier;
  
  if (requiredTier && !meetsMinimumTier(currentTier, requiredTier)) {
    return {
      hasPermission: false,
      reason: 'tier_insufficient',
      requiredTier,
      upgradeUrl: getUpgradeUrlForFeature(feature)
    };
  }
  
  return {
    hasPermission: false,
    reason: 'no_permission'
  };
}

/**
 * Get required tier for specific feature
 */
function getRequiredTierForFeature(feature: string): PackageTier | null {
  const featureTiers: Record<string, PackageTier> = {
    'export_data': PackageTier.BRONZE,
    'real_time': PackageTier.BRONZE,
    'advanced_analytics': PackageTier.SILVER,
    'live_trading': PackageTier.GOLD,
    'algorithmic_trading': PackageTier.PLATINUM,
    'priority_support': PackageTier.GOLD
  };
  
  return featureTiers[feature] || null;
}

/**
 * Check if current tier meets minimum requirement
 */
function meetsMinimumTier(currentTier: PackageTier, requiredTier: PackageTier): boolean {
  const tierOrder = [
    PackageTier.FREE,
    PackageTier.BRONZE,
    PackageTier.SILVER,
    PackageTier.GOLD,
    PackageTier.PLATINUM,
    PackageTier.ENTERPRISE
  ];
  
  const currentIndex = tierOrder.indexOf(currentTier);
  const requiredIndex = tierOrder.indexOf(requiredTier);
  
  return currentIndex >= requiredIndex;
}

// ============================================================================
// Session Validation
// ============================================================================

/**
 * Validate user session
 */
export function validateUserSession(
  session: UserSessionData | null | undefined
): boolean {
  if (!session) return false;
  
  // Check session expiry
  if (Date.now() > session.expiresAt) return false;
  
  // Validate user profile
  if (!session.user || !session.user.id || !session.user.email) return false;
  
  return true;
}

/**
 * Check if session is close to expiry
 */
export function isSessionCloseToExpiry(
  session: UserSessionData,
  warningMinutes: number = 15
): boolean {
  const warningTime = Date.now() + (warningMinutes * 60 * 1000);
  return session.expiresAt < warningTime;
}

// ============================================================================
// UI Helper Functions
// ============================================================================

/**
 * Get user display name
 */
export function getUserDisplayName(user: UserProfile | null | undefined): string {
  if (!user) return 'Guest User';
  return user.name || user.email.split('@')[0] || 'User';
}

/**
 * Get package tier display name
 */
export function getTierDisplayName(tier: PackageTier): string {
  const displayNames = {
    [PackageTier.FREE]: 'Free',
    [PackageTier.BRONZE]: 'Bronze',
    [PackageTier.SILVER]: 'Silver',
    [PackageTier.GOLD]: 'Gold',
    [PackageTier.PLATINUM]: 'Platinum',
    [PackageTier.ENTERPRISE]: 'Enterprise'
  };
  
  return displayNames[tier] || 'Unknown';
}

/**
 * Get tier color for UI components
 */
export function getTierColor(tier: PackageTier): string {
  const colors = {
    [PackageTier.FREE]: 'bg-gray-100 text-gray-800',
    [PackageTier.BRONZE]: 'bg-orange-100 text-orange-800',
    [PackageTier.SILVER]: 'bg-gray-100 text-gray-700',
    [PackageTier.GOLD]: 'bg-yellow-100 text-yellow-800',
    [PackageTier.PLATINUM]: 'bg-purple-100 text-purple-800',
    [PackageTier.ENTERPRISE]: 'bg-blue-100 text-blue-800'
  };
  
  return colors[tier] || 'bg-gray-100 text-gray-800';
}

/**
 * Format user permissions for display (simplified)
 */
export function formatUserPermissions(permissions: string[]): string[] {
  return permissions.map(permission => {
    // Handle wildcard
    if (permission === 'epsx:*:*') return 'Full Access';
    
    // Format structured permissions
    const [platform, resource, action] = permission.split(':');
    if (platform === 'epsx' && resource && action) {
      const resourceLabel = resource.charAt(0).toUpperCase() + resource.slice(1);
      const actionLabel = action === '*' ? 'All' : 
        action.charAt(0).toUpperCase() + action.slice(1);
      return `${resourceLabel} ${actionLabel}`;
    }
    
    return permission;
  }).slice(0, 5); // Limit to 5 most relevant permissions for display
}