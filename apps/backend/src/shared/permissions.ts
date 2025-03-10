import { UserRole, TokenFeature } from './types/roles.enum';
import { TOKEN_FEATURE_REQUIREMENTS, canAccessFeature } from './types/features';

export const PERMISSIONS_LIST = {
  // Basic Access
  VIEW_PUBLIC_RANKS: 'view:public_ranks',
  VIEW_BASIC_ANALYSIS: 'view:basic_analysis',
  VIEW_PREMIUM_ANALYSIS: 'view:premium_analysis',
  
  // Portfolio Management
  MANAGE_BASIC_PORTFOLIO: 'manage:basic_portfolio',
  MANAGE_PREMIUM_PORTFOLIO: 'manage:premium_portfolio',
  USE_PORTFOLIO_ASSISTANCE: 'use:portfolio_assistance',
  
  // Trading & Analysis
  USE_TRADING_BOT: 'use:trading_bot',
  ACCESS_AI_ANALYSIS: 'use:ai_analysis',
  USE_ADVANCED_TOOLS: 'use:advanced_tools',
  VIEW_RISK_ANALYSIS: 'view:risk_analysis',
  VIEW_TREND_DETECTION: 'view:trend_detection',
  
  // Token Holder Features
  VOTE_SYSTEM_UPGRADES: 'governance:vote_upgrades',
  VOTE_INVESTMENT_STRATEGIES: 'governance:vote_strategies',
  RECEIVE_PLATFORM_DIVIDENDS: 'token:receive_dividends',
  
  // Administrative
  MANAGE_USERS: 'admin:manage_users',
  MANAGE_ROLES: 'admin:manage_roles',
  MANAGE_EPS_DATA: 'admin:manage_eps_data',
  UPGRADE_INFRASTRUCTURE: 'admin:upgrade_infrastructure',
  FIX_SYSTEM_ISSUES: 'admin:fix_issues',
  MONITOR_SYSTEM: 'admin:monitor_system'
} as const;

export type Permission = typeof PERMISSIONS_LIST[keyof typeof PERMISSIONS_LIST];
export type PermissionKey = keyof typeof PERMISSIONS_LIST;

export const PERMISSIONS: Record<PermissionKey, Permission> = PERMISSIONS_LIST;

// Feature to Permission mapping
export const FEATURE_PERMISSIONS: Record<TokenFeature, Permission[]> = {
  [TokenFeature.REAL_TIME_ANALYSIS]: [
    PERMISSIONS.VIEW_BASIC_ANALYSIS,
    PERMISSIONS.VIEW_PREMIUM_ANALYSIS
  ],
  [TokenFeature.PORTFOLIO_MANAGEMENT]: [
    PERMISSIONS.MANAGE_BASIC_PORTFOLIO,
    PERMISSIONS.MANAGE_PREMIUM_PORTFOLIO
  ],
  [TokenFeature.PORTFOLIO_ASSISTANCE]: [
    PERMISSIONS.USE_PORTFOLIO_ASSISTANCE
  ],
  [TokenFeature.TRADING_BOT]: [
    PERMISSIONS.USE_TRADING_BOT
  ],
  [TokenFeature.AI_ANALYSIS]: [
    PERMISSIONS.ACCESS_AI_ANALYSIS
  ],
  [TokenFeature.ADVANCED_TOOLS]: [
    PERMISSIONS.USE_ADVANCED_TOOLS,
    PERMISSIONS.VIEW_RISK_ANALYSIS,
    PERMISSIONS.VIEW_TREND_DETECTION
  ],
  [TokenFeature.GOVERNANCE]: [
    PERMISSIONS.VOTE_SYSTEM_UPGRADES,
    PERMISSIONS.VOTE_INVESTMENT_STRATEGIES,
    PERMISSIONS.RECEIVE_PLATFORM_DIVIDENDS
  ]
};

// Role to Base Permissions mapping (without token-gated features)
export const ROLE_PERMISSIONS: Record<UserRole, Permission[]> = {
  [UserRole.ADMINISTRATOR]: [
    ...Object.values(PERMISSIONS)
  ],
  [UserRole.TOKEN_HOLDER]: [
    PERMISSIONS.VIEW_PUBLIC_RANKS,
    PERMISSIONS.VIEW_BASIC_ANALYSIS,
    PERMISSIONS.VIEW_PREMIUM_ANALYSIS,
    PERMISSIONS.MANAGE_BASIC_PORTFOLIO,
    PERMISSIONS.MANAGE_PREMIUM_PORTFOLIO
  ],
  [UserRole.PREMIUM_USER]: [
    PERMISSIONS.VIEW_PUBLIC_RANKS,
    PERMISSIONS.VIEW_BASIC_ANALYSIS,
    PERMISSIONS.VIEW_PREMIUM_ANALYSIS,
    PERMISSIONS.MANAGE_BASIC_PORTFOLIO
  ],
  [UserRole.REGISTERED_USER]: [
    PERMISSIONS.VIEW_PUBLIC_RANKS,
    PERMISSIONS.VIEW_BASIC_ANALYSIS,
    PERMISSIONS.MANAGE_BASIC_PORTFOLIO
  ],
  [UserRole.GUEST]: [
    PERMISSIONS.VIEW_PUBLIC_RANKS
  ]
};

export function hasPermission(role: UserRole, permission: Permission): boolean {
  if (role === UserRole.ADMINISTRATOR) return true;
  return ROLE_PERMISSIONS[role]?.includes(permission) || false;
}

export function getFeaturePermissions(feature: TokenFeature): Permission[] {
  return FEATURE_PERMISSIONS[feature] || [];
}

export function hasFeatureAccess(
  role: UserRole,
  tokenBalance: number,
  feature: TokenFeature
): boolean {
  // First check if user can access the feature based on role and token balance
  if (!canAccessFeature(role, tokenBalance, feature)) {
    return false;
  }

  // Then verify they have all required permissions for the feature
  const requiredPermissions = getFeaturePermissions(feature);
  return requiredPermissions.every(permission => hasPermission(role, permission));
}

export function getUserPermissions(role: UserRole, tokenBalance: number): Permission[] {
  // Start with base role permissions
  const permissions = new Set(ROLE_PERMISSIONS[role]);

  // Add permissions from accessible features
  const availableFeatures = Object.values(TokenFeature).filter(feature =>
    canAccessFeature(role, tokenBalance, feature)
  );

  availableFeatures.forEach(feature => {
    getFeaturePermissions(feature).forEach(permission => {
      permissions.add(permission);
    });
  });

  return Array.from(permissions);
}
