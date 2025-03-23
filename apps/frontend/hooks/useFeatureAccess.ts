"use client";

import { useAuth } from '@/context/auth-context';
import { TokenFeature } from '@/types/auth/features';
import type { FeatureAccess, Permission, UpgradeRequirement } from '@/types/auth/features';
import { UserRole } from '@/types/auth/roles';

export function useFeatureAccess() {
  const { role, tokenBalance, features, permissions } = useAuth();

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
    hasFeature: (feature: TokenFeature) => features.includes(feature),
    checkFeatureAccess,
    getUpgradeRequirements,
    canAccessFeature: (feature: TokenFeature) => checkFeatureAccess(feature).hasAccess,
    availableFeatures: features,
    tokenBalance,
    role
  };
}

// Helper functions
function hasRequiredRole(currentRole: UserRole, requiredRole: UserRole): boolean {
  const roleValues = {
    [UserRole.GUEST]: 0,
    [UserRole.REGISTERED_USER]: 1,
    [UserRole.PREMIUM_USER]: 2,
    [UserRole.TOKEN_HOLDER]: 3,
    [UserRole.ADMINISTRATOR]: 4
  };

  return roleValues[currentRole] >= roleValues[requiredRole];
}

function getFeatureRequirements(feature: TokenFeature): {
  minTokens: number;
  requiredRole: UserRole;
} {
  // This would come from a shared config
  const requirements = {
    [TokenFeature.REAL_TIME_ANALYSIS]: {
      minTokens: 100,
      requiredRole: UserRole.REGISTERED_USER
    },
    [TokenFeature.PORTFOLIO_MANAGEMENT]: {
      minTokens: 500,
      requiredRole: UserRole.REGISTERED_USER
    },
    [TokenFeature.PORTFOLIO_ASSISTANCE]: {
      minTokens: 1000,
      requiredRole: UserRole.PREMIUM_USER
    },
    [TokenFeature.TRADING_BOT]: {
      minTokens: 2500,
      requiredRole: UserRole.PREMIUM_USER
    },
    [TokenFeature.AI_ANALYSIS]: {
      minTokens: 5000,
      requiredRole: UserRole.PREMIUM_USER
    },
    [TokenFeature.ADVANCED_TOOLS]: {
      minTokens: 7500,
      requiredRole: UserRole.PREMIUM_USER
    },
    [TokenFeature.GOVERNANCE]: {
      minTokens: 10000,
      requiredRole: UserRole.TOKEN_HOLDER
    }
  };

  return requirements[feature];
}

function getFeaturePermissions(feature: TokenFeature): Permission[] {
  // This would come from a shared config
  const featurePermissions: Record<TokenFeature, Permission[]> = {
    [TokenFeature.REAL_TIME_ANALYSIS]: ['view:basic_analysis', 'view:premium_analysis'],
    [TokenFeature.PORTFOLIO_MANAGEMENT]: ['manage:basic_portfolio', 'manage:premium_portfolio'],
    [TokenFeature.PORTFOLIO_ASSISTANCE]: ['use:portfolio_assistance'],
    [TokenFeature.TRADING_BOT]: ['use:trading_bot'],
    [TokenFeature.AI_ANALYSIS]: ['use:ai_analysis'],
    [TokenFeature.ADVANCED_TOOLS]: ['use:advanced_tools', 'view:risk_analysis', 'view:trend_detection'],
    [TokenFeature.GOVERNANCE]: ['governance:vote_upgrades', 'governance:vote_strategies']
  } as const;

  return featurePermissions[feature];
}
