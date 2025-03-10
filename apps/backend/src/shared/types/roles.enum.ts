export enum UserRole {
  GUEST = "guest",
  REGISTERED_USER = "registered_user",
  PREMIUM_USER = "premium_user",
  TOKEN_HOLDER = "token_holder",
  ADMINISTRATOR = "administrator"
}

export enum TokenFeature {
  REAL_TIME_ANALYSIS = "real_time_analysis",
  PORTFOLIO_MANAGEMENT = "portfolio_management",
  PORTFOLIO_ASSISTANCE = "portfolio_assistance",
  TRADING_BOT = "trading_bot",
  AI_ANALYSIS = "ai_analysis",
  ADVANCED_TOOLS = "advanced_tools",
  GOVERNANCE = "governance"
}

// Access level mapping for roles (used for hierarchical access)
export const ROLE_ACCESS_LEVELS: Record<UserRole, number> = {
  [UserRole.ADMINISTRATOR]: 5,
  [UserRole.TOKEN_HOLDER]: 4,
  [UserRole.PREMIUM_USER]: 3,
  [UserRole.REGISTERED_USER]: 2,
  [UserRole.GUEST]: 1
};

// Helper to check if a role has sufficient access level
export function hasRequiredAccessLevel(userRole: UserRole, requiredRole: UserRole): boolean {
  return ROLE_ACCESS_LEVELS[userRole] >= ROLE_ACCESS_LEVELS[requiredRole];
}
