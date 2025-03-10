import { UserRole, TokenFeature } from './roles.enum';

export interface FeatureRequirement {
  minTokens: number;
  requiredRoles: UserRole[];
  description: string;
}

export type FeatureRequirements = {
  [key in TokenFeature]: FeatureRequirement;
};

export const TOKEN_FEATURE_REQUIREMENTS: FeatureRequirements = {
  [TokenFeature.REAL_TIME_ANALYSIS]: {
    minTokens: 100,
    requiredRoles: [UserRole.REGISTERED_USER, UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    description: "Access to real-time stock analysis dashboard"
  },
  [TokenFeature.PORTFOLIO_MANAGEMENT]: {
    minTokens: 500,
    requiredRoles: [UserRole.REGISTERED_USER, UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    description: "Access to portfolio management system"
  },
  [TokenFeature.PORTFOLIO_ASSISTANCE]: {
    minTokens: 1000,
    requiredRoles: [UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    description: "Access to portfolio assistance system with alerts and recommendations"
  },
  [TokenFeature.TRADING_BOT]: {
    minTokens: 2500,
    requiredRoles: [UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    description: "Access to automated trading bot"
  },
  [TokenFeature.AI_ANALYSIS]: {
    minTokens: 5000,
    requiredRoles: [UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    description: "Access to AI-driven stock analysis"
  },
  [TokenFeature.ADVANCED_TOOLS]: {
    minTokens: 7500,
    requiredRoles: [UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    description: "Access to advanced tools like risk analysis and trend detection"
  },
  [TokenFeature.GOVERNANCE]: {
    minTokens: 10000,
    requiredRoles: [UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    description: "Voting rights in system upgrades and investment strategies"
  }
};

// Helper functions for feature access checks
export function hasRequiredTokens(balance: number, feature: TokenFeature): boolean {
  const requirement = TOKEN_FEATURE_REQUIREMENTS[feature];
  return balance >= requirement.minTokens;
}

export function hasRequiredRole(userRole: UserRole, feature: TokenFeature): boolean {
  const requirement = TOKEN_FEATURE_REQUIREMENTS[feature];
  return requirement.requiredRoles.includes(userRole);
}

export function canAccessFeature(userRole: UserRole, tokenBalance: number, feature: TokenFeature): boolean {
  // Administrators always have access to all features
  if (userRole === UserRole.ADMINISTRATOR) return true;
  
  return hasRequiredRole(userRole, feature) && hasRequiredTokens(tokenBalance, feature);
}

export function getAvailableFeatures(userRole: UserRole, tokenBalance: number): TokenFeature[] {
  return Object.values(TokenFeature).filter(feature => 
    canAccessFeature(userRole, tokenBalance, feature)
  );
}

export function getFeatureRequirements(feature: TokenFeature): {
  requiredTokens: number;
  requiredRole: UserRole;
  description: string;
} {
  const requirement = TOKEN_FEATURE_REQUIREMENTS[feature];
  return {
    requiredTokens: requirement.minTokens,
    // Get the minimum required role (first one in the array)
    requiredRole: requirement.requiredRoles[0],
    description: requirement.description
  };
}
