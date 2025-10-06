/**
 * ADMIN FRONTEND API TYPES - MIGRATED TO SHARED
 * All API types moved to shared/types/api with compatibility layer
 * This file now re-exports shared types for backward compatibility
 */

// Re-export everything from shared API types
export * from '../../../shared/types/api';

// Import for local re-export with legacy names (maintaining compatibility)
import type { 
  ApiResponse as SharedApiResponse,
  ApiError as SharedApiError,
  ActionResult as SharedActionResult,
  LoginRequest as SharedLoginRequest,
  LoginResponse as SharedLoginResponse,
  CreateUserRequest as SharedCreateUserRequest,
  CreateUserResponse as SharedCreateUserResponse,
  UpdateUserRequest as SharedUpdateUserRequest,
  BulkUserOperationRequest as SharedBulkUserOperationRequest,
  BulkOperationResponse as SharedBulkOperationResponse,
  UserSearchRequest as SharedUserSearchRequest,
  GrantPermissionRequest as SharedGrantPermissionRequest,
  RevokePermissionRequest as SharedRevokePermissionRequest,
  BulkPermissionRequest as SharedBulkPermissionRequest,
  BulkPermissionResponse as SharedBulkPermissionResponse,
  PermissionTemplate as SharedPermissionTemplate,
  Notification as SharedNotification,
  NotificationCreateRequest as SharedNotificationCreateRequest,
  BroadcastNotificationRequest as SharedBroadcastNotificationRequest,
  EPSRanking as SharedEPSRanking,
  EPSRankingsResponse as SharedEPSRankingsResponse,
  PerformanceMetrics as SharedPerformanceMetrics,
  SystemRecommendation as SharedSystemRecommendation
} from '../../../shared/types/api';

// Re-export with exact same names for backward compatibility
export type ApiResponse<T = any> = SharedApiResponse<T>;
export type ApiError = SharedApiError;
export type ActionResult<T = any> = SharedActionResult<T>;

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

// Legacy compatibility (admin-frontend was importing from core types)
import type { User, Permission, UserStats, PermissionAnalytics } from './core';
import type { PaginatedResponse } from '../../../shared/types/api';
export type { User, Permission, UserStats, PermissionAnalytics } from './core';
export type { PaginatedResponse };