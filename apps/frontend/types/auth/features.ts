import { UserRole } from './roles';
import type { User as FirebaseUser } from 'firebase/auth';
import type { UserSubscription } from '@/types/payment/plans';

export interface User extends FirebaseUser {
  role: UserRole;
  token_balance: number;
  features: TokenFeature[];
  permissions: Permission[];
  subscription?: UserSubscription; // New payment system
}

export enum TokenFeature {
  TRADING = 'TRADING',
  ADMIN_ACCESS = 'ADMIN_ACCESS',
  REAL_TIME_ANALYSIS = 'REAL_TIME_ANALYSIS',
  TRADING_BOT = 'TRADING_BOT',
  AI_ANALYSIS = 'AI_ANALYSIS',
  PORTFOLIO_MANAGEMENT = 'PORTFOLIO_MANAGEMENT',
  ADVANCED_TOOLS = 'ADVANCED_TOOLS',
}

export enum Permission {
  READ = 'READ',
  WRITE = 'WRITE',
  DELETE = 'DELETE',
  ADMIN = 'ADMIN',
  MANAGE_USERS = 'MANAGE_USERS',
  MANAGE_ROLES = 'MANAGE_ROLES',
  VIEW_ANALYTICS = 'VIEW_ANALYTICS',
  EXECUTE_TRADES = 'EXECUTE_TRADES',
  ACCESS_API = 'ACCESS_API',
}

export interface GoogleAuthParams {
  redirectUrl?: string;
}

export interface UpgradeRequirement {
  type: 'ROLE_UPGRADE' | 'TOKEN_PURCHASE';
  currentValue: number | UserRole;
  requiredValue: number | UserRole;
  description: string;
}

export interface FeatureAccess {
  hasAccess: boolean;
  currentTokens: number;
  requiredTokens?: number;
  currentRole: UserRole;
  requiredRole?: UserRole;
  missingPermissions: Permission[];
}

export interface AuthResponse {
  success: boolean;
  message?: string;
  user?: {
    email: string;
    user_id: string;
    role: UserRole;
    token_balance: number;
    features: TokenFeature[];
    permissions: Permission[];
  };
  oauth_url?: string;
}
