/**
 * Local type definitions for EPSX Admin Frontend
 * Replaces @epsx/types dependency with self-contained types
 */

// Re-export existing types
export * from './userLevel';

// Export separated authentication types
export * from './auth-separation';

// Payment and Subscription Types
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

// Package Tier Enum (for assignments and pricing)
export enum PackageTier {
  FREE = 'free',
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum',
}

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

// Notification Types
export interface Notification {
  id: string;
  title: string;
  message: string;
  type: 'trading' | 'system' | 'account' | 'price_alert' | 'security' | 'compliance';
  priority: 'low' | 'medium' | 'high' | 'critical';
  read: boolean;
  createdAt: string;
  updatedAt: string;
  userId?: string;
  actionUrl?: string;
  metadata?: Record<string, any>;
}

export interface NotificationListParams {
  page?: number;
  limit?: number;
  type?: string;
  priority?: string;
  read?: boolean;
  userId?: string;
  startDate?: string;
  endDate?: string;
}

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
  type: Notification['type'];
  priority: Notification['priority'];
  userId?: string;
  userIds?: string[];
  actionUrl?: string;
  metadata?: Record<string, any>;
}

export interface NotificationUpdateRequest {
  title?: string;
  message?: string;
  type?: Notification['type'];
  priority?: Notification['priority'];
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

export interface NotificationStats {
  total: number;
  unread: number;
  byType: Record<string, number>;
  byPriority: Record<string, number>;
  last24Hours: number;
  lastWeek: number;
}
