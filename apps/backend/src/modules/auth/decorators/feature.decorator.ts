import { SetMetadata } from '@nestjs/common';
import { PermissionKey } from '../constants/permissions';

export const FEATURES_KEY = 'features';
export const REQUIRED_PERMISSIONS_KEY = 'required_permissions';

export enum TokenFeature {
  ADVANCED_TOOLS = 'advanced_tools',
  PORTFOLIO_MANAGEMENT = 'portfolio_management',
  TRADING_BOT = 'trading_bot',
  AI_ANALYSIS = 'ai_analysis',
  PORTFOLIO_AUTOMATION = 'portfolio_automation'
}

export const RequireFeatures = (
  features: TokenFeature[],
  permissions: PermissionKey[] = []
) => {
  return (target: any, key: string | symbol, descriptor?: any) => {
    if (descriptor) {
      // Method decorator
      SetMetadata(FEATURES_KEY, features)(target, key, descriptor);
      SetMetadata(REQUIRED_PERMISSIONS_KEY, permissions)(target, key, descriptor);
      return descriptor;
    }
    // Class decorator
    SetMetadata(FEATURES_KEY, features)(target);
    SetMetadata(REQUIRED_PERMISSIONS_KEY, permissions)(target);
    return target;
  };
};

export const RequirePortfolioManagement = () => RequireFeatures(
  [TokenFeature.PORTFOLIO_MANAGEMENT]
);

export const RequireTradingBot = () => RequireFeatures(
  [TokenFeature.TRADING_BOT]
);

export const RequireAIAnalysis = () => RequireFeatures(
  [TokenFeature.AI_ANALYSIS]
);

export const RequirePortfolioAutomation = () => RequireFeatures(
  [TokenFeature.PORTFOLIO_AUTOMATION]
);
