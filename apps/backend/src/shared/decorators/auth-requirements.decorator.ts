import { applyDecorators, SetMetadata, UseGuards } from '@nestjs/common';
import { UserRole, TokenFeature } from '../types/roles.enum';
import { Permission, PERMISSIONS } from '../permissions';
import { TokenFeatureGuard } from '../guards/token-feature.guard';

export const AUTH_REQUIREMENTS_KEY = 'auth_requirements';

export interface AuthRequirements {
  roles?: UserRole[];
  features?: TokenFeature[];
  permissions?: Permission[];
}

// Main decorator that combines all requirements
export function Auth(requirements: AuthRequirements = {}) {
  return applyDecorators(
    SetMetadata(AUTH_REQUIREMENTS_KEY, requirements),
    UseGuards(TokenFeatureGuard)
  );
}

// Helper decorator for public routes (no auth required)
export function Public() {
  return Auth({
    roles: [
      UserRole.GUEST,
      UserRole.REGISTERED_USER,
      UserRole.PREMIUM_USER,
      UserRole.TOKEN_HOLDER,
      UserRole.ADMINISTRATOR
    ]
  });
}

// Helper decorator for registered users
export function RequireRegistered() {
  return Auth({
    roles: [
      UserRole.REGISTERED_USER,
      UserRole.PREMIUM_USER,
      UserRole.TOKEN_HOLDER,
      UserRole.ADMINISTRATOR
    ]
  });
}

// Helper decorator for premium users
export function RequirePremium() {
  return Auth({
    roles: [
      UserRole.PREMIUM_USER,
      UserRole.TOKEN_HOLDER,
      UserRole.ADMINISTRATOR
    ]
  });
}

// Helper decorator for token holders
export function RequireTokenHolder() {
  return Auth({
    roles: [
      UserRole.TOKEN_HOLDER,
      UserRole.ADMINISTRATOR
    ]
  });
}

// Helper decorator for administrators
export function RequireAdmin() {
  return Auth({
    roles: [UserRole.ADMINISTRATOR]
  });
}

// Feature-specific decorators

export function RequireRealTimeAnalysis() {
  return Auth({
    roles: [UserRole.REGISTERED_USER, UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    features: [TokenFeature.REAL_TIME_ANALYSIS],
    permissions: [PERMISSIONS.VIEW_BASIC_ANALYSIS, PERMISSIONS.VIEW_PREMIUM_ANALYSIS]
  });
}

export function RequirePortfolioManagement() {
  return Auth({
    roles: [UserRole.REGISTERED_USER, UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    features: [TokenFeature.PORTFOLIO_MANAGEMENT],
    permissions: [PERMISSIONS.MANAGE_BASIC_PORTFOLIO, PERMISSIONS.MANAGE_PREMIUM_PORTFOLIO]
  });
}

export function RequirePortfolioAssistance() {
  return Auth({
    roles: [UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    features: [TokenFeature.PORTFOLIO_ASSISTANCE],
    permissions: [PERMISSIONS.USE_PORTFOLIO_ASSISTANCE]
  });
}

export function RequireTradingBot() {
  return Auth({
    roles: [UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    features: [TokenFeature.TRADING_BOT],
    permissions: [PERMISSIONS.USE_TRADING_BOT]
  });
}

export function RequireAIAnalysis() {
  return Auth({
    roles: [UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    features: [TokenFeature.AI_ANALYSIS],
    permissions: [PERMISSIONS.ACCESS_AI_ANALYSIS]
  });
}

export function RequireAdvancedTools() {
  return Auth({
    roles: [UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    features: [TokenFeature.ADVANCED_TOOLS],
    permissions: [PERMISSIONS.USE_ADVANCED_TOOLS, PERMISSIONS.VIEW_RISK_ANALYSIS, PERMISSIONS.VIEW_TREND_DETECTION]
  });
}

export function RequireGovernance() {
  return Auth({
    roles: [UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    features: [TokenFeature.GOVERNANCE],
    permissions: [PERMISSIONS.VOTE_SYSTEM_UPGRADES, PERMISSIONS.VOTE_INVESTMENT_STRATEGIES]
  });
}

// Combined requirement decorators

export function RequirePortfolioAutomation() {
  return Auth({
    roles: [UserRole.PREMIUM_USER, UserRole.TOKEN_HOLDER, UserRole.ADMINISTRATOR],
    features: [
      TokenFeature.PORTFOLIO_MANAGEMENT,
      TokenFeature.TRADING_BOT,
      TokenFeature.AI_ANALYSIS
    ],
    permissions: [
      PERMISSIONS.MANAGE_PREMIUM_PORTFOLIO,
      PERMISSIONS.USE_TRADING_BOT,
      PERMISSIONS.ACCESS_AI_ANALYSIS
    ]
  });
}

// Custom requirements helper
export function RequireFeatures(features: TokenFeature[], permissions?: Permission[]) {
  return Auth({
    features,
    permissions
  });
}
