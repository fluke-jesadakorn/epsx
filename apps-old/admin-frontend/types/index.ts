/**
 * Local type definitions for EPSX Admin Frontend
 * Replaces @epsx/types dependency with self-contained types
 */

// Import permission group types
import type { PermissionGroup } from '@/shared/types/domain/user';

// Export separated authentication types
export * from './auth-separation';

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

// Permission Group Config
export interface PermissionGroupConfig {
  id: string;
  name: PermissionGroup;
  description: string;
  level: number;
  maxStocks: number;
  refreshRate: 'realtime' | 'hourly' | 'daily';
  features: string[];
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
  metadata?: Record<string, unknown>;
}

export interface BulkStockRankingPermissionAssignment {
  targetUsers: string[];
  permissionGroupId: string;
  stockRankingTypeId: string;
  expiresAt?: string;
  metadata?: Record<string, unknown>;
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

// Notification Types - Local definition (avoids import resolution issues)
export interface Notification {
  id: string;
  title: string;
  message: string;
  type: string;
  priority: string;
  read: boolean;
  createdAt: string;
  updatedAt?: string;
  userId?: string;
  actionUrl?: string;
  metadata?: Record<string, unknown>;
}

export type NotificationType = 'info' | 'warning' | 'error' | 'success' | 'system';
export type NotificationPriority = 'low' | 'medium' | 'high' | 'critical';

export interface NotificationStats {
  total: number;
  unread: number;
  byType: Record<string, number>;
}

export interface NotificationListParams {
  page?: number;
  limit?: number;
  unreadOnly?: boolean;
  type?: string;
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
  type: NotificationType | string; // Use any or specific type if needed for compatibility
  priority: NotificationPriority | string;
  userId?: string;
  userIds?: string[];
  actionUrl?: string;
  metadata?: Record<string, unknown>;
}

export interface NotificationUpdateRequest {
  title?: string;
  message?: string;
  type?: NotificationType | string;
  priority?: NotificationPriority | string;
  read?: boolean;
  actionUrl?: string;
  metadata?: Record<string, unknown>;
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
