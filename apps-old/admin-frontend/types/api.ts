/**
 * ADMIN FRONTEND API TYPES - MIGRATED TO SHARED
 * All API types moved to shared/types/api with compatibility layer
 * This file now re-exports shared types for backward compatibility
 */

// Re-export everything from shared API types
// Import for local re-export with legacy names (maintaining compatibility)
import type {
  EPSRanking as SharedEPSRanking,
  AnalyticsRankingsResponse as SharedEPSRankingsResponse,
} from '@/shared/types/analytics';
import type {
  PaginatedResponse,
  ActionResult as SharedActionResult,
  ApiError as SharedApiError,
  ApiResponse as SharedApiResponse,
  BulkOperationResponse as SharedBulkOperationResponse,
  BulkPermissionRequest as SharedBulkPermissionRequest,
  BulkPermissionResponse as SharedBulkPermissionResponse,
  BulkUserOperationRequest as SharedBulkUserOperationRequest,
  CreateUserRequest as SharedCreateUserRequest,
  CreateUserResponse as SharedCreateUserResponse,
  GrantPermissionRequest as SharedGrantPermissionRequest,
  LoginRequest as SharedLoginRequest,
  LoginResponse as SharedLoginResponse,
  PerformanceMetrics as SharedPerformanceMetrics,
  PermissionTemplate as SharedPermissionTemplate,
  RevokePermissionRequest as SharedRevokePermissionRequest,
  SystemRecommendation as SharedSystemRecommendation,
  UpdateUserRequest as SharedUpdateUserRequest,
  UserSearchRequest as SharedUserSearchRequest
} from '@/shared/types/api';
import type {
  BroadcastNotificationRequest as SharedBroadcastNotificationRequest,
  Notification as SharedNotification,
  NotificationCreateRequest as SharedNotificationCreateRequest,
} from '@/shared/types/notifications';

// Legacy compatibility (admin-frontend was importing from core types)

export * from '@/shared/types/api';
// export type { Permission, PermissionAnalytics, User, UserStats } from './core';
export type { PaginatedResponse };

// Re-export with exact same names for backward compatibility
export type ApiResponse<T = unknown> = SharedApiResponse<T>;
export type ApiError = SharedApiError;
export type ActionResult<T = unknown> = SharedActionResult<T>;

// Authentication API Types
export type LoginRequest = SharedLoginRequest;
export type LoginResponse = SharedLoginResponse;

// User Management API Types
export type CreateUserRequest = SharedCreateUserRequest;
export type CreateUserResponse = SharedCreateUserResponse;
export type UpdateUserRequest = SharedUpdateUserRequest;
export type BulkUserOperationRequest = SharedBulkUserOperationRequest;
export type BulkOperationResponse = SharedBulkOperationResponse;
export type UserSearchRequest = SharedUserSearchRequest;

// Permission Management API Types  
export type GrantPermissionRequest = SharedGrantPermissionRequest;
export type RevokePermissionRequest = SharedRevokePermissionRequest;
export type BulkPermissionRequest = SharedBulkPermissionRequest;
export type BulkPermissionResponse = SharedBulkPermissionResponse;
export type PermissionTemplate = SharedPermissionTemplate;

// Notification API Types
export type Notification = SharedNotification;
export type NotificationCreateRequest = SharedNotificationCreateRequest;
export type BroadcastNotificationRequest = SharedBroadcastNotificationRequest;

// Analytics API Types
export type EPSRanking = SharedEPSRanking;
export type EPSRankingsResponse = SharedEPSRankingsResponse;
export type PerformanceMetrics = SharedPerformanceMetrics;
export type SystemRecommendation = SharedSystemRecommendation;
