// Mirror the backend TokenFeature enum
export enum TokenFeature {
  REAL_TIME_ANALYSIS = "real_time_analysis",
  PORTFOLIO_MANAGEMENT = "portfolio_management",
  PORTFOLIO_ASSISTANCE = "portfolio_assistance",
  TRADING_BOT = "trading_bot",
  AI_ANALYSIS = "ai_analysis",
  ADVANCED_TOOLS = "advanced_tools",
  GOVERNANCE = "governance"
}

export interface FeatureRequirement {
  minTokens: number;
  requiredRole: UserRole;
  description: string;
}

export interface UserFeatureAccess {
  availableFeatures: TokenFeature[];
  tokenBalance: number;
  requirements: {
    [key in TokenFeature]: FeatureRequirement;
  };
}

// Mirror backend permission constants
export const PERMISSIONS = {
  VIEW_PUBLIC_RANKS: 'view:public_ranks',
  VIEW_BASIC_ANALYSIS: 'view:basic_analysis',
  VIEW_PREMIUM_ANALYSIS: 'view:premium_analysis',
  MANAGE_BASIC_PORTFOLIO: 'manage:basic_portfolio',
  MANAGE_PREMIUM_PORTFOLIO: 'manage:premium_portfolio',
  USE_PORTFOLIO_ASSISTANCE: 'use:portfolio_assistance',
  USE_TRADING_BOT: 'use:trading_bot',
  ACCESS_AI_ANALYSIS: 'use:ai_analysis',
  USE_ADVANCED_TOOLS: 'use:advanced_tools',
  VIEW_RISK_ANALYSIS: 'view:risk_analysis',
  VIEW_TREND_DETECTION: 'view:trend_detection',
  VOTE_SYSTEM_UPGRADES: 'governance:vote_upgrades',
  VOTE_INVESTMENT_STRATEGIES: 'governance:vote_strategies',
  RECEIVE_PLATFORM_DIVIDENDS: 'token:receive_dividends',
} as const;

export type Permission = typeof PERMISSIONS[keyof typeof PERMISSIONS];

// Helper types for the feature system
export interface FeatureAccess {
  hasAccess: boolean;
  requiredTokens?: number;
  currentTokens: number;
  requiredRole?: UserRole;
  currentRole: UserRole;
  missingPermissions: Permission[];
}

export interface UpgradeRequirement {
  type: 'ROLE_UPGRADE' | 'TOKEN_PURCHASE';
  currentValue: number | string;
  requiredValue: number | string;
  description: string;
}

// Import at top of file
import { UserRole } from './roles';
