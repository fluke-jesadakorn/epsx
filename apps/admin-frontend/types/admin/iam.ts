// IAM Types for AWS-style permission system
import { PermissionSource } from '@/shared/types/domain/Permission';
import { Group } from '@/shared/types/domain/User';

export interface UserWithPermissions extends User {
  group: Group;
  permissionGroup: Group; // Keep as alias
  // @deprecated Use group instead
  packageTier?: PackageTier;
  customPermissions: CustomPermission[];
  effectivePermissions: EffectivePermission[];
  packagePermissions: PackagePermission[];
  subscriptionStatus: SubscriptionStatus;
  lastPaymentDate?: Date;
}

export interface CustomPermission {
  id: string;
  userId: string;
  featureId: string;
  permission: Permission;
  grantedBy: string; // Admin who granted this
  grantedAt: Date;
  expiresAt?: Date;
  reason?: string;
  isActive: boolean;
}

export interface PermissionGroupPermission {
  id: string;
  group: Group;
  permissionGroup: Group; // Keep as alias
  featureId: string;
  permission: Permission;
  isDefault: boolean;
}

/** @deprecated Use PermissionGroupPermission instead */
export interface PackagePermission {
  id: string;
  packageTier: PackageTier;
  featureId: string;
  permission: Permission;
  isDefault: boolean;
  autoGranted: boolean;
}

export interface EffectivePermission {
  featureId: string;
  permission: Permission;
  source: PermissionSource;
  grantedAt: Date;
  expiresAt?: Date;
  grantedBy?: string;
}

// PermissionSource imported from shared system

/** @deprecated Use PermissionGroup from shared types instead */
export enum PackageTier {
  FREE = 'free',
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum',
  ENTERPRISE = 'enterprise',
}

export enum SubscriptionStatus {
  ACTIVE = 'active',
  EXPIRED = 'expired',
  CANCELLED = 'cancelled',
  PENDING = 'pending',
  TRIAL = 'trial',
}

export interface Permission {
  action: string;
  resource: string;
  conditions?: PermissionCondition[];
}

export interface PermissionCondition {
  type: 'usage_limit' | 'time_range' | 'ip_restriction' | 'resource_owner';
  value:
  | string
  | number
  | string[]
  | { start: string; end: string }
  | { min: number; max: number };
  operator: 'eq' | 'gt' | 'lt' | 'in' | 'between';
}

export interface PermissionProfile {
  id: string;
  name: string;
  description: string;
  permissions: Permission[];
  category: string;
  isSystem: boolean;
}

// Feature definitions for the EPSX system
export interface Feature {
  id: string;
  name: string;
  description: string;
  category: FeatureCategory;
  permissions: Permission[];
  requiredGroup?: Group;
  requiredPermissionGroup?: Group; // Keep as alias
  // @deprecated Use requiredGroup instead
  requiredTier?: PackageTier;
  isAdmin?: boolean;
}

export enum FeatureCategory {
  DASHBOARD = 'dashboard',
  PACKAGES = 'packages',
  API_ACCESS = 'api_access',
  PROFILE = 'profile',
  SUPPORT = 'support',
  ADMIN_USER = 'admin_user',
  ADMIN_IAM = 'admin_iam',
  ADMIN_PACKAGE = 'admin_package',
  ADMIN_SYSTEM = 'admin_system',
  ADMIN_SUPPORT = 'admin_support',
}

export interface User {
  id: string;
  email: string;
  name?: string;
  displayName?: string;
  emailVerified: boolean;
  disabled: boolean;
  roles: string[];
  groups: string[];
  attachedPolicies: string[];
  status: 'active' | 'inactive' | 'disabled';
  lastActivity?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Role {
  id: string;
  name: string;
  description: string;
  attachedPolicies: string[];
  trustPolicy?: PolicyDocument;
  maxSessionDuration?: number;
  path?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Policy {
  id: string;
  name: string;
  description: string;
  policyDocument: PolicyDocument;
  arn?: string;
  path?: string;
  isAttachable: boolean;
  attachmentCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface IamGroup {
  id: string;
  name: string;
  description: string;
  memberCount: number;
  attachedPolicies: string[];
  path?: string;
  createdAt: string;
  updatedAt: string;
}

export interface PolicyDocument {
  Version: '2012-10-17';
  Statement: PolicyStatement[];
}

export interface PolicyStatement {
  Sid?: string;
  Effect: 'Allow' | 'Deny';
  Action: string | string[];
  Resource?: string | string[];
  Condition?: Record<string, any>;
  Principal?: string | string[] | { [key: string]: string | string[] };
}

export interface Permission {
  id: string;
  service: string;
  action: string;
  resource: string;
  effect: 'Allow' | 'Deny';
  conditions?: PermissionCondition[];
}

export interface UserPermission {
  userId: string;
  permissionId: string;
  grantedBy: string;
  grantedAt: string;
  expiresAt?: string;
  source: 'direct' | 'role' | 'group' | 'policy';
  sourceId: string;
}

export interface AuditLog {
  id: string;
  userId: string;
  action: string;
  resource: string;
  effect: 'Allow' | 'Deny';
  timestamp: string;
  ipAddress?: string;
  userAgent?: string;
  details?: Record<string, any>;
}

export interface IAMStats {
  totalUsers: number;
  activeUsers: number;
  totalRoles: number;
  totalPolicies: number;
  totalGroups: number;
  totalPermissions: number;
}

// Form types for modals
export interface UserFormData {
  email: string;
  name?: string;
  displayName?: string;
  roles: string[];
  groups: string[];
  attachedPolicies: string[];
  status: 'active' | 'inactive' | 'disabled';
}

export interface RoleFormData {
  name: string;
  description: string;
  attachedPolicies: string[];
  trustPolicy?: PolicyDocument;
  maxSessionDuration?: number;
  path?: string;
}

export interface PolicyFormData {
  name: string;
  description: string;
  policyDocument: PolicyDocument;
  path?: string;
}

export interface GroupFormData {
  name: string;
  description: string;
  attachedPolicies: string[];
  path?: string;
}

// Policy permission profiles
export const POLICY_PERMISSION_PROFILES = {
  Bronze: {
    Version: '2012-10-17' as const,
    Statement: [
      {
        Effect: 'Allow' as const,
        Action: ['dashboard:read', 'profile:read', 'profile:update'],
        Resource: ['user:self'],
      },
    ],
  },
  Silver: {
    Version: '2012-10-17' as const,
    Statement: [
      {
        Effect: 'Allow' as const,
        Action: [
          'dashboard:read',
          'profile:read',
          'profile:update',
          'analytics:read',
        ],
        Resource: ['user:self', 'analytics:basic'],
      },
    ],
  },
  Gold: {
    Version: '2012-10-17' as const,
    Statement: [
      {
        Effect: 'Allow' as const,
        Action: [
          'dashboard:read',
          'profile:read',
          'profile:update',
          'analytics:read',
          'trading:read',
        ],
        Resource: ['user:self', 'analytics:*', 'trading:*'],
      },
    ],
  },
  Platinum: {
    Version: '2012-10-17' as const,
    Statement: [
      {
        Effect: 'Allow' as const,
        Action: [
          'dashboard:*',
          'profile:*',
          'analytics:*',
          'trading:*',
          'api:read',
        ],
        Resource: ['user:self', 'analytics:*', 'trading:*', 'api:*'],
      },
    ],
  },
  Admin: {
    Version: '2012-10-17' as const,
    Statement: [
      {
        Effect: 'Allow' as const,
        Action: ['*'],
        Resource: ['*'],
      },
    ],
  },
};
