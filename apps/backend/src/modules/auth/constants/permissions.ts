import { UserRole } from '../../../shared/types/roles.enum';

export const PERMISSIONS = {
  VIEW_RISK_ANALYSIS: 'view_risk_analysis',
  PORTFOLIO_MANAGEMENT: 'portfolio_management',
  TRADING_BOT: 'trading_bot',
  AI_ANALYSIS: 'ai_analysis',
  PORTFOLIO_AUTOMATION: 'portfolio_automation'
} as const;

export type PermissionKey = keyof typeof PERMISSIONS;

export interface Permission {
  key: string;
  name: string;
  description: string;
  roles: UserRole[];
}

export const hasPermission = (userRole: UserRole, permission: Permission): boolean => {
  return permission.roles.includes(userRole);
};
