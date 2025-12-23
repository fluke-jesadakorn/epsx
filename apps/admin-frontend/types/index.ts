/**
 * Local type definitions for EPSX Admin Frontend
 * Replaces @epsx/types dependency with self-contained types
 */

// Import permission group types
import { PermissionGroup } from '../../../shared/types/domain/User';

// Re-export existing types
export * from './permission-templates';

// Export separated authentication types
export * from './auth-separation';

// Payment and Subscription Types (deprecated - use PermissionGroup)
/** @deprecated Use PermissionGroup instead */
export enum PaymentTier {
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum'
}

export interface PaymentTierConfig {
  id: string;
  name: string;
  description: string;
  monthlyPrice: number;
  yearlyPrice: number;
  features: string[];
  maxUsers?: number;
  maxApiCalls?: number;
  priority: 'low' | 'medium' | 'high';
  isActive: boolean;
}

export interface UserSubscription {
  id: string;
  userId: string;
  tierName: string;
  paymentTier: PaymentTierConfig;
  startDate: string;
  endDate: string;
  status: 'active' | 'cancelled' | 'expired' | 'pending';
  autoRenew: boolean;
  paymentMethod: 'credit_card' | 'paypal' | 'bank_transfer';
  lastPaymentDate?: string;
  nextPaymentDate?: string;
  cancelledAt?: string;
  cancelReason?: string;
}

// Stock Ranking Types
export interface StockRankingType {
  id: string;
  name: string;
  description: string;
  algorithm: string;
  category: 'growth' | 'value' | 'dividend' | 'momentum' | 'quality';
  riskLevel: 'low' | 'medium' | 'high';
  timeHorizon: 'short' | 'medium' | 'long';
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

// Package Tier Enum (deprecated - use PermissionGroup)
/** @deprecated Use PermissionGroup instead */
export enum PackageTier {
  FREE = 'free',
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum',
}

// Permission Group Config (replaces PackageTierConfig)
export interface PermissionGroupConfig {
  id: string;
  name: PermissionGroup;
  description: string;
  level: number;
  maxStocks: number;
  refreshRate: 'realtime' | 'hourly' | 'daily';
  features: string[];
}

/** @deprecated Use PermissionGroupConfig instead */
export interface PackageTierConfig {
  id: string;
  name: string;
  description: string;
  level: number;
  maxStocks: number;
  refreshRate: 'realtime' | 'hourly' | 'daily';
  features: string[];
  monthlyPrice: number;
  isActive: boolean;
}

export interface StockRankingPermissionAssignment {
  id: string;
  userId: string;
  permissionGroupId: string;
  stockRankingTypeId: string;
  permissionGroup: PermissionGroupConfig;
  stockRankingType: StockRankingType;
  assignedAt: string;
  expiresAt?: string;
  status: 'active' | 'expired' | 'revoked';
  assignedBy: string;
  metadata?: Record<string, any>;
}

/** @deprecated Use StockRankingPermissionAssignment instead */
export interface StockRankingPackageAssignment {
  id: string;
  userId: string;
  packageTierId: string;
  stockRankingTypeId: string;
  packageTier: PackageTierConfig;
  stockRankingType: StockRankingType;
  assignedAt: string;
  expiresAt?: string;
  status: 'active' | 'expired' | 'revoked';
  assignedBy: string;
  metadata?: Record<string, any>;
}

export interface BulkStockRankingPermissionAssignment {
  targetUsers: string[];
  permissionGroupId: string;
  stockRankingTypeId: string;
  expiresAt?: string;
  metadata?: Record<string, any>;
}

/** @deprecated Use BulkStockRankingPermissionAssignment instead */
export interface BulkStockRankingAssignment {
  targetUsers: string[];
  packageTierId: string;
  stockRankingTypeId: string;
  expiresAt?: string;
  metadata?: Record<string, any>;
}

export interface BulkStockRankingAssignmentResult {
  totalTargeted: number;
  successful: number;
  failed: number;
  errors: Array<{
    userId: string;
    error: string;
  }>;
  assignmentIds: string[];
  createdAt: string;
}

// Permission Types (for PermissionProfileModal)
export interface Permission {
  resource: string;
  action: string;
}

export interface PermissionProfile {
  id: string;
  name: string;
  description: string;
  category: string;
  targetTier: string;
  isActive: boolean;
  permissions: Permission[];
  createdAt: string;
  updatedAt: string;
  assignedUserCount?: number;
}

// Notification Types - Re-exported from shared
export type {
  Notification,
  NotificationListParams, NotificationPriority, NotificationStats,
  NotificationType
} from '@/shared/types/notifications';

export interface NotificationListResponse {
  notifications: Notification[];
  pagination: {
    currentPage: number;
    totalPages: number;
    totalItems: number;
    itemsPerPage: number;
  };
}

export interface NotificationCreateRequest {
  title: string;
  message: string;
  type: any; // Use any or specific type if needed for compatibility
  priority: any;
  userId?: string;
  userIds?: string[];
  actionUrl?: string;
  metadata?: Record<string, any>;
}

export interface NotificationUpdateRequest {
  title?: string;
  message?: string;
  type?: any;
  priority?: any;
  read?: boolean;
  actionUrl?: string;
  metadata?: Record<string, any>;
}

export interface NotificationPreferences {
  inApp: boolean;
  email: boolean;
  push: boolean;
  tradingAlerts: boolean;
  systemUpdates: boolean;
  securityAlerts: boolean;
  complianceNotifications: boolean;
}
