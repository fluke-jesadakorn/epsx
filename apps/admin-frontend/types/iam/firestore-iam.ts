// Core IAM types for Firestore storage
// All IAM data stored in Firestore, Firebase Auth only for credentials

export interface RoleDocument {
  id: string;
  name: string;
  description: string;
  color: string;
  icon: string;
  permissions: string[]; // Array of permission IDs
  isSystem: boolean;
  isActive: boolean;
  createdBy: string; // Admin UID
  createdAt: Date;
  updatedAt: Date;
  usageCount: number;
  metadata: {
    category: string;
    tags: string[];
    version: number;
  };
}

export interface PermissionDocument {
  id: string;
  name: string;
  description: string;
  category: PermissionCategory;
  action: string;
  resource: string;
  scope: PermissionScope;
  conditions?: PermissionCondition[];
  isSystem: boolean;
  tags: string[];
  deprecated: boolean;
  metadata: {
    createdAt: Date;
    updatedAt: Date;
    version: number;
  };
}

export interface UserProfileDocument {
  uid: string; // Links to Firebase Auth UID
  email: string; // Cache for quick lookup
  displayName?: string;
  photoURL?: string;
  
  // IAM Data (stored in Firestore only)
  iam: {
    roles: string[]; // References to role IDs
    explicitPermissions: string[]; // Direct permission IDs
    packageTier: PackageTier;
    subscriptionStatus: SubscriptionStatus;
    effectivePermissions: string[]; // Calculated permissions
    permissionVersion: number;
    lastUpdate: Date;
  };
  
  // Profile Data
  profile: {
    company?: string;
    jobTitle?: string;
    country?: string;
    timezone?: string;
    preferences: UserPreferences;
  };
  
  // Usage Tracking
  usage: {
    apiCalls: {
      daily: number;
      monthly: number;
      lastReset: Date;
    };
    exports: {
      daily: number;
      monthly: number;
      lastReset: Date;
    };
    storage: {
      used: number; // MB
      limit: number; // MB
    };
  };
  
  // Audit Trail
  audit: {
    createdAt: Date;
    createdBy?: string;
    lastModifiedAt: Date;
    lastModifiedBy?: string;
    permissionHistory: PermissionHistory[];
  };
}

export interface UserPermissionDocument {
  uid: string;
  permissionId: string;
  grantedBy: string; // Admin UID
  grantedAt: Date;
  expiresAt?: Date;
  reason?: string;
  isActive: boolean;
  source: PermissionSource;
  conditions?: PermissionCondition[];
}

export interface PermissionHistory {
  action: 'grant' | 'revoke' | 'modify';
  permissionId: string;
  performedBy: string;
  timestamp: Date;
  reason?: string;
  metadata?: any;
}

export interface PermissionCondition {
  type: 'usage_limit' | 'time_range' | 'ip_restriction' | 'subscription_status';
  operator: 'equals' | 'not_equals' | 'greater_than' | 'less_than';
  value: any;
}

export enum PermissionCategory {
  DASHBOARD = 'dashboard',
  API = 'api',
  DATA = 'data',
  ADMIN = 'admin',
  ANALYTICS = 'analytics',
  INTEGRATION = 'integration',
  BILLING = 'billing',
  SUPPORT = 'support'
}

export enum PermissionScope {
  OWN = 'own',
  COMPANY = 'company',
  PARTNER = 'partner',
  GLOBAL = 'global'
}

export enum PackageTier {
  FREE = 'free',
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum',
  ENTERPRISE = 'enterprise'
}

export enum SubscriptionStatus {
  ACTIVE = 'active',
  EXPIRED = 'expired',
  CANCELLED = 'cancelled',
  PENDING = 'pending',
  TRIAL = 'trial'
}

export enum PermissionSource {
  ROLE = 'role',
  EXPLICIT = 'explicit',
  PACKAGE = 'package'
}

export interface UserPreferences {
  theme: 'light' | 'dark' | 'auto';
  language: string;
  timezone: string;
  notifications: {
    email: boolean;
    push: boolean;
    sms: boolean;
  };
}
