import type { User } from './iam';

export interface UserWithPermissions extends User {
  packageTier: PackageTier;
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

export enum PermissionSource {
  PACKAGE = 'package',
  CUSTOM = 'custom',
  ROLE = 'role',
  GROUP = 'group',
}

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
  value: any;
  operator: 'eq' | 'gt' | 'lt' | 'in' | 'between';
}

export interface PermissionTemplate {
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

// Payment integration types
export interface PaymentEvent {
  userId: string;
  packageTier: PackageTier;
  transactionId: string;
  amount: number;
  currency: string;
  paymentDate: Date;
  eventType: 'upgrade' | 'renewal' | 'downgrade' | 'cancellation';
}

// Audit logging
export interface PermissionAuditLog {
  id: string;
  userId: string;
  action: string;
  resource: string;
  performedBy: string;
  reason?: string;
  timestamp: Date;
  metadata?: Record<string, any>;
}

// Bulk operations
export interface BulkPermissionOperation {
  userIds: string[];
  templateId?: string;
  customPermissions?: CustomPermission[];
  operation: 'grant' | 'revoke' | 'template';
  performedBy: string;
  reason?: string;
}

// Permission preview for upgrades
export interface PermissionPreview {
  currentPermissions: EffectivePermission[];
  newPermissions: PackagePermission[];
  addedPermissions: PackagePermission[];
  removedPermissions: PackagePermission[];
}
