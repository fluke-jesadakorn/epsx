"use client";

import { useAuth } from '@/context/auth-context';
import { TokenFeature, Permission } from '@/types/auth/features';
import type { FeatureAccess, UpgradeRequirement, User } from '@/types/auth/features';
import { UserRole } from '@/types/auth/roles';
import { PaymentTier } from '@/types/payment/plans';
import { PaymentService } from '@/services/paymentService';

export function useFeatureAccess() {
  const { user: firebaseUser } = useAuth();
  const user = firebaseUser as User | null;
  const role = user?.role ?? UserRole.USER;
  const tokenBalance = user?.token_balance ?? 0;
  const features = user?.features ?? [];
  const permissions = user?.permissions ?? [];
  
  // Get user's payment tier (new payment system)
  const userTier = user?.subscription?.tier || PaymentTier.BASIC;

  const hasPaymentFeature = (feature: string): boolean => {
    return PaymentService.hasFeatureAccess(userTier, feature);
  };

  const getApiLimits = () => {
    return PaymentService.getApiLimitsByTier(userTier);
  };

  const canAccessRankings = (count: number): boolean => {
    return PaymentService.canAccessRankings(userTier, count);
  };

  const checkFeatureAccess = (feature: TokenFeature): FeatureAccess => {
    // Feature requirements would be imported from a shared config
    const requirements = getFeatureRequirements(feature);
    const hasRole = hasRequiredRole(role, requirements.requiredRole);
    const hasTokens = tokenBalance >= requirements.minTokens;
    const requiredPermissions = getFeaturePermissions(feature);
    const missingPermissions = requiredPermissions.filter(
      perm => !permissions.includes(perm)
    );

    return {
      hasAccess: hasRole && hasTokens && missingPermissions.length === 0,
      requiredTokens: requirements.minTokens,
      currentTokens: tokenBalance,
      requiredRole: requirements.requiredRole,
      currentRole: role,
      missingPermissions
    };
  };

  const getUpgradeRequirements = (feature: TokenFeature): UpgradeRequirement[] => {
    const access = checkFeatureAccess(feature);
    const requirements: UpgradeRequirement[] = [];

    if (!access.hasAccess) {
      if (access.requiredRole && !hasRequiredRole(access.currentRole, access.requiredRole)) {
        requirements.push({
          type: 'ROLE_UPGRADE',
          currentValue: access.currentRole,
          requiredValue: access.requiredRole,
          description: `Upgrade to ${access.requiredRole} required`
        });
      }

      if (access.requiredTokens && access.currentTokens < access.requiredTokens) {
        requirements.push({
          type: 'TOKEN_PURCHASE',
          currentValue: access.currentTokens,
          requiredValue: access.requiredTokens,
          description: `${access.requiredTokens - access.currentTokens} more tokens required`
        });
      }
    }

    return requirements;
  };

  return {
    // Legacy token-based features
    hasFeature: (feature: TokenFeature) => features.includes(feature),
    checkFeatureAccess,
    getUpgradeRequirements,
    canAccessFeature: (feature: TokenFeature) => checkFeatureAccess(feature).hasAccess,
    availableFeatures: features,
    tokenBalance,
    role,
    
    // New payment-based features
    userTier,
    hasPaymentFeature,
    getApiLimits,
    canAccessRankings
  };
}

// Helper functions
function hasRequiredRole(currentRole: UserRole, requiredRole: UserRole): boolean {
  const roleValues = {
    [UserRole.USER]: 0,
    [UserRole.ADMIN]: 1
  };

  return roleValues[currentRole] >= roleValues[requiredRole];
}

function getFeatureRequirements(feature: TokenFeature): {
  minTokens: number;
  requiredRole: UserRole;
} {
  // This would come from a shared config
  const requirements = {
    [TokenFeature.TRADING]: {
      minTokens: 100,
      requiredRole: UserRole.USER
    },
    [TokenFeature.REAL_TIME_ANALYSIS]: {
      minTokens: 500,
      requiredRole: UserRole.USER
    },
    [TokenFeature.TRADING_BOT]: {
      minTokens: 1000,
      requiredRole: UserRole.USER
    },
    [TokenFeature.AI_ANALYSIS]: {
      minTokens: 2000,
      requiredRole: UserRole.USER
    },
    [TokenFeature.PORTFOLIO_MANAGEMENT]: {
      minTokens: 5000,
      requiredRole: UserRole.USER
    },
    [TokenFeature.ADVANCED_TOOLS]: {
      minTokens: 10000,
      requiredRole: UserRole.USER
    },
    [TokenFeature.ADMIN_ACCESS]: {
      minTokens: 0,
      requiredRole: UserRole.ADMIN
    }
  };

  return requirements[feature];
}

function getFeaturePermissions(feature: TokenFeature): Permission[] {
  // This would come from a shared config
  const featurePermissions: Record<TokenFeature, Permission[]> = {
    [TokenFeature.TRADING]: [Permission.EXECUTE_TRADES],
    [TokenFeature.REAL_TIME_ANALYSIS]: [Permission.VIEW_ANALYTICS],
    [TokenFeature.TRADING_BOT]: [Permission.EXECUTE_TRADES, Permission.ACCESS_API],
    [TokenFeature.AI_ANALYSIS]: [Permission.VIEW_ANALYTICS, Permission.ACCESS_API],
    [TokenFeature.PORTFOLIO_MANAGEMENT]: [Permission.READ, Permission.WRITE],
    [TokenFeature.ADVANCED_TOOLS]: [Permission.READ, Permission.WRITE, Permission.ACCESS_API],
    [TokenFeature.ADMIN_ACCESS]: [Permission.ADMIN, Permission.MANAGE_USERS, Permission.MANAGE_ROLES]
  } as const;

  return featurePermissions[feature];
}
