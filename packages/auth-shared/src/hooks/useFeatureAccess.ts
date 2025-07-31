import { useMemo, useCallback } from 'react';
import type { AuthenticatedUser, FrontendUser } from '@epsx/types';

// Feature tiers and definitions
export enum FeatureTier {
  FREE = 'free',
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum',
  ADMIN = 'admin',
}

export enum TokenFeature {
  BASIC_RANKINGS = 'basic_rankings',
  ADVANCED_ANALYTICS = 'advanced_analytics',
  CUSTOM_REPORTS = 'custom_reports',
  API_ACCESS = 'api_access',
  PRIORITY_SUPPORT = 'priority_support',
  BULK_OPERATIONS = 'bulk_operations',
}

export interface FeatureConfig {
  id: string;
  requiredTier?: FeatureTier;
  requiredPermissions?: string[];
  requiredRoles?: string[];
  tokenCost?: number;
  description: string;
}

export interface FeatureAccessResult {
  hasAccess: boolean;
  reason: string;
  requiredTier?: FeatureTier;
  currentTier?: FeatureTier;
  tokenCost?: number;
  currentTokens?: number;
}

export interface UseFeatureAccessOptions {
  /** Enable batch feature checking */
  enableBatch?: boolean;
  /** Auto-deduct tokens on access */
  autoDeductTokens?: boolean;
}

export interface FeatureAccessContext {
  user: AuthenticatedUser | null;
  permissions: string[];
  roles: string[];
  hasPermission: (permission: string) => boolean;
  hasRole: (role: string) => boolean;
}

/**
 * Unified feature access hook that consolidates tier-based and permission-based access
 * Handles both admin and user feature access patterns
 */
export function useFeatureAccess(
  context: FeatureAccessContext,
  options: UseFeatureAccessOptions = {}
) {
  const { enableBatch = true } = options;
  const { user, roles, hasPermission, hasRole } = context;

  // Feature tier hierarchy (lower index = higher tier)
  const tierHierarchy = useMemo(() => [
    FeatureTier.ADMIN,
    FeatureTier.PLATINUM,
    FeatureTier.GOLD,
    FeatureTier.SILVER,
    FeatureTier.BRONZE,
    FeatureTier.FREE,
  ], []);

  // Get user's current tier
  const currentTier = useMemo((): FeatureTier => {
    if (!user) return FeatureTier.FREE;
    
    // Admin users get admin tier
    if (roles.includes('admin') || roles.includes('super_admin')) {
      return FeatureTier.ADMIN;
    }
    
    // Get tier from user subscription (for frontend users)
    const frontendUser = user as FrontendUser;
    if (frontendUser.subscription_tier) {
      return frontendUser.subscription_tier as FeatureTier;
    }
    
    if (frontendUser.package_tier) {
      return frontendUser.package_tier as FeatureTier;
    }
    
    // Default to free
    return FeatureTier.FREE;
  }, [user, roles]);

  // Get user's token balance
  const currentTokens = useMemo(() => {
    const frontendUser = user as FrontendUser;
    return frontendUser?.token_balance || 0;
  }, [user]);

  // Check if current tier meets required tier
  const hasRequiredTier = useCallback((requiredTier: FeatureTier) => {
    const currentIndex = tierHierarchy.indexOf(currentTier);
    const requiredIndex = tierHierarchy.indexOf(requiredTier);
    return currentIndex <= requiredIndex; // Lower index = higher tier
  }, [currentTier, tierHierarchy]);

  // Core feature access checking
  const checkFeatureAccess = useCallback((
    featureConfig: FeatureConfig
  ): FeatureAccessResult => {
    const {
      requiredTier,
      requiredPermissions = [],
      requiredRoles = [],
      tokenCost = 0,
      description,
    } = featureConfig;

    // Check authentication
    if (!user) {
      return {
        hasAccess: false,
        reason: 'Authentication required',
        currentTier,
      };
    }

    // Check tier requirement
    if (requiredTier && !hasRequiredTier(requiredTier)) {
      return {
        hasAccess: false,
        reason: `${description} requires ${requiredTier} tier or higher`,
        requiredTier,
        currentTier,
      };
    }

    // Check permission requirements
    if (requiredPermissions.length > 0) {
      const hasAllPermissions = requiredPermissions.every(permission => 
        hasPermission(permission));
      
      if (!hasAllPermissions) {
        return {
          hasAccess: false,
          reason: `Missing required permissions: ${requiredPermissions.join(', ')}`,
          currentTier,
        };
      }
    }

    // Check role requirements
    if (requiredRoles.length > 0) {
      const hasRequiredRole = requiredRoles.some(role => hasRole(role));
      
      if (!hasRequiredRole) {
        return {
          hasAccess: false,
          reason: `Missing required roles: ${requiredRoles.join(', ')}`,
          currentTier,
        };
      }
    }

    // Check token requirements
    if (tokenCost > 0 && currentTokens < tokenCost) {
      return {
        hasAccess: false,
        reason: `Insufficient tokens. Required: ${tokenCost}, Available: ${currentTokens}`,
        tokenCost,
        currentTokens,
        currentTier,
      };
    }

    return {
      hasAccess: true,
      reason: 'Feature access granted',
      currentTier,
      tokenCost,
      currentTokens,
    };
  }, [user, hasRequiredTier, hasPermission, hasRole, currentTier, currentTokens]);

  // Batch feature access checking
  const checkMultipleFeatureAccess = useCallback((
    features: FeatureConfig[]
  ): { [featureId: string]: FeatureAccessResult } => {
    const results: { [featureId: string]: FeatureAccessResult } = {};
    
    for (const feature of features) {
      results[feature.id] = checkFeatureAccess(feature);
    }
    
    return results;
  }, [checkFeatureAccess]);

  // Pre-defined feature configurations
  const predefinedFeatures = useMemo((): { [key: string]: FeatureConfig } => ({
    // User features
    [TokenFeature.BASIC_RANKINGS]: {
      id: TokenFeature.BASIC_RANKINGS,
      requiredTier: FeatureTier.FREE,
      description: 'Basic rankings view',
    },
    [TokenFeature.ADVANCED_ANALYTICS]: {
      id: TokenFeature.ADVANCED_ANALYTICS,
      requiredTier: FeatureTier.SILVER,
      tokenCost: 10,
      description: 'Advanced analytics dashboard',
    },
    [TokenFeature.CUSTOM_REPORTS]: {
      id: TokenFeature.CUSTOM_REPORTS,
      requiredTier: FeatureTier.GOLD,
      tokenCost: 25,
      description: 'Custom report generation',
    },
    [TokenFeature.API_ACCESS]: {
      id: TokenFeature.API_ACCESS,
      requiredTier: FeatureTier.PLATINUM,
      requiredPermissions: ['api:access'],
      description: 'API access privileges',
    },
    [TokenFeature.PRIORITY_SUPPORT]: {
      id: TokenFeature.PRIORITY_SUPPORT,
      requiredTier: FeatureTier.GOLD,
      description: 'Priority customer support',
    },
    [TokenFeature.BULK_OPERATIONS]: {
      id: TokenFeature.BULK_OPERATIONS,
      requiredTier: FeatureTier.PLATINUM,
      tokenCost: 50,
      description: 'Bulk data operations',
    },
    
    // Admin features
    'admin_user_management': {
      id: 'admin_user_management',
      requiredRoles: ['admin', 'super_admin'],
      requiredPermissions: ['admin.users.*'],
      description: 'User management interface',
    },
    'admin_payment_management': {
      id: 'admin_payment_management',
      requiredRoles: ['admin', 'super_admin'],
      requiredPermissions: ['admin.payments.*'],
      description: 'Payment management interface',
    },
    'admin_system_settings': {
      id: 'admin_system_settings',
      requiredRoles: ['super_admin'],
      requiredPermissions: ['admin.system.*'],
      description: 'System configuration',
    },
    'admin_analytics_dashboard': {
      id: 'admin_analytics_dashboard',
      requiredRoles: ['admin', 'super_admin'],
      requiredPermissions: ['admin.analytics.*'],
      description: 'Analytics dashboard',
    },
  }), []);

  // Convenience methods for common features
  const canAccessRankings = useCallback(() => {
    return checkFeatureAccess(predefinedFeatures[TokenFeature.BASIC_RANKINGS]).hasAccess;
  }, [checkFeatureAccess, predefinedFeatures]);

  const canAccessAdvancedAnalytics = useCallback(() => {
    return checkFeatureAccess(predefinedFeatures[TokenFeature.ADVANCED_ANALYTICS]).hasAccess;
  }, [checkFeatureAccess, predefinedFeatures]);

  // Admin convenience methods
  const canManageUsers = useCallback(() => {
    return checkFeatureAccess(predefinedFeatures['admin_user_management']).hasAccess;
  }, [checkFeatureAccess, predefinedFeatures]);

  const canViewPayments = useCallback(() => {
    return checkFeatureAccess(predefinedFeatures['admin_payment_management']).hasAccess;
  }, [checkFeatureAccess, predefinedFeatures]);

  const canManageSystem = useCallback(() => {
    return checkFeatureAccess(predefinedFeatures['admin_system_settings']).hasAccess;
  }, [checkFeatureAccess, predefinedFeatures]);

  const canViewAnalytics = useCallback(() => {
    return checkFeatureAccess(predefinedFeatures['admin_analytics_dashboard']).hasAccess;
  }, [checkFeatureAccess, predefinedFeatures]);

  // Pagination helpers (for tier-based limits)
  const getMaxAllowedLimit = useCallback(() => {
    switch (currentTier) {
      case FeatureTier.ADMIN:
      case FeatureTier.PLATINUM:
        return 1000;
      case FeatureTier.GOLD:
        return 500;
      case FeatureTier.SILVER:
        return 100;
      case FeatureTier.BRONZE:
        return 50;
      case FeatureTier.FREE:
      default:
        return 20;
    }
  }, [currentTier]);

  const canAccessPage = useCallback((pageNumber: number, itemsPerPage: number) => {
    const maxItems = getMaxAllowedLimit();
    const maxPages = Math.ceil(maxItems / itemsPerPage);
    return pageNumber <= maxPages;
  }, [getMaxAllowedLimit]);

  const getAvailablePageSizes = useCallback((): number[] => {
    const maxLimit = getMaxAllowedLimit();
    const baseSizes = [10, 25, 50, 100, 250, 500, 1000];
    return baseSizes.filter(size => size <= maxLimit);
  }, [getMaxAllowedLimit]);

  return {
    // Core methods
    checkFeatureAccess,
    checkMultipleFeatureAccess: enableBatch ? checkMultipleFeatureAccess : undefined,
    
    // Convenience methods
    canAccessRankings,
    canAccessAdvancedAnalytics,
    
    // Admin methods
    canManageUsers,
    canViewPayments,
    canManageSystem,
    canViewAnalytics,
    
    // Pagination helpers
    getMaxAllowedLimit,
    canAccessPage,
    getAvailablePageSizes,
    
    // State
    currentTier,
    currentTokens,
    isAuthenticated: !!user,
    predefinedFeatures,
  };
}