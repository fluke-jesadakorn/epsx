/**
 * API Types
 * Consolidates all API request/response types
 * Replaces scattered API types across multiple files
 */

import type { User, Permission, UserStats, PermissionAnalytics, PaginatedResponse } from './core';

// ============================================================================
// Generic API Response Types
// ============================================================================

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  status: number;
}

export interface ApiError {
  message: string;
  status: number;
  code?: string;
}

export interface ActionResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

// ============================================================================
// Authentication API Types
// ============================================================================

export interface LoginRequest {
  email: string;
  password: string;
  rememberMe?: boolean;
}

export interface LoginResponse {
  user: User;
  tokens: {
    access_token: string;
    id_token: string;
    refresh_token: string;
    expires_in: number;
  };
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface AuthCallbackRequest {
  code: string;
  state: string;
}

export interface AuthenticationResult {
  success: boolean;
  user?: User;
  tokens?: any;
  error?: string;
  redirectUrl?: string;
}

// ============================================================================
// User Management API Types
// ============================================================================

export interface CreateUserRequest {
  email: string;
  permissions: string[];
  displayName?: string;
  firstName?: string;
  lastName?: string;
  role?: string;
  packageTier?: string;
}

export interface CreateUserResponse {
  userId: string;
  message: string;
}

export interface UpdateUserRequest {
  email?: string;
  displayName?: string;
  firstName?: string;
  lastName?: string;
  role?: string;
  packageTier?: string;
  isActive?: boolean;
  permissions?: string[];
}

export interface BulkUserOperationRequest {
  userIds: string[];
  operation: 'activate' | 'deactivate' | 'delete' | 'update_role' | 'assign_permissions';
  data?: {
    role?: string;
    permissions?: string[];
    packageTier?: string;
  };
}

export interface BulkOperationResponse {
  totalProcessed: number;
  successful: number;
  failed: number;
  errors: Array<{
    userId: string;
    error: string;
  }>;
}

export interface UserSearchRequest {
  query: string;
  filters?: {
    role?: string;
    status?: string;
    tier?: string;
  };
  limit?: number;
  offset?: number;
}

// ============================================================================
// Permission Management API Types
// ============================================================================

export interface GrantPermissionRequest {
  userId: string;
  permission: string;
  expiresAt?: string;
  reason?: string;
}

export interface RevokePermissionRequest {
  userId: string;
  permission: string;
  reason?: string;
}

export interface BulkPermissionRequest {
  userIds: string[];
  permissions: string[];
  expiresAt?: string;
  reason?: string;
}

export interface BulkPermissionResponse {
  totalUsers: number;
  successful: number;
  failed: number;
  errors: Array<{
    userId: string;
    error: string;
  }>;
  assignmentIds: string[];
}

export interface PermissionExpiryStatusRequest {
  userId: string;
}

export interface PermissionExpiryStatusResponse {
  expiringPermissions: Array<{
    permission: string;
    expiresAt: string;
    daysRemaining: number;
  }>;
  expiredPermissions: string[];
  healthScore: number;
}

export interface PermissionTemplate {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  targetRole?: string;
  isActive: boolean;
}

export interface CreatePermissionTemplateRequest {
  name: string;
  description: string;
  permissions: string[];
  targetRole?: string;
}

// ============================================================================
// Notification API Types
// ============================================================================

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

export interface BroadcastNotificationRequest {
  title: string;
  message: string;
  type: string;
  priority: string;
  userIds?: string[];
  allUsers?: boolean;
  metadata?: Record<string, any>;
}

export interface BroadcastNotificationResponse {
  notificationIds: string[];
  userCount: number;
  message: string;
}

export interface NotificationStats {
  total: number;
  unread: number;
  byType: Record<string, number>;
  byPriority: Record<string, number>;
  last24Hours: number;
  lastWeek: number;
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

// ============================================================================
// Analytics API Types
// ============================================================================

export interface EPSRanking {
  symbol: string;
  country: string;
  sector: string;
  epsGrowth: number;
  marketCap?: number;
  revenue?: number;
  netIncome?: number;
  rank?: number;
}

export interface EPSRankingsResponse {
  rankings: EPSRanking[];
  totalRequests: number;
  lastUpdated: string;
  pagination?: {
    page: number;
    limit: number;
    total: number;
  };
}

export interface EPSHealthResponse {
  status: 'healthy' | 'degraded' | 'down';
  uptime: number;
  responseTime: string;
  lastCheck: string;
}

export interface PerformanceMetrics {
  apiResponseTime: number;
  databaseQueryTime: number;
  memoryUsage: number;
  activeUsers: number;
  peakUsersToday: number;
  newSignups: number;
  errorRate: number;
  throughput: number;
}

export interface SystemRecommendation {
  id: string;
  priority: 'low' | 'medium' | 'high' | 'critical';
  type: 'security' | 'performance' | 'maintenance' | 'optimization';
  title: string;
  description: string;
  impact: 'low' | 'medium' | 'high';
  effort: 'low' | 'medium' | 'high';
  category?: string;
  estimatedTimeToFix?: string;
  resources?: string[];
}

export interface RecommendationsResponse {
  insights: SystemRecommendation[];
  confidence: number;
  generatedAt: string;
}

export interface AnalyticsDashboardData {
  userStats: UserStats;
  permissionAnalytics: PermissionAnalytics;
  performanceMetrics: PerformanceMetrics;
  recentActivity: number;
  systemHealth: {
    status: 'healthy' | 'warning' | 'critical';
    uptime: number;
    issues: number;
  };
}

// ============================================================================
// Stock Ranking API Types
// ============================================================================

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

export interface StockRankingAssignment {
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

export interface StockRankingAssignmentRequest {
  userId: string;
  packageTierId: string;
  stockRankingTypeId: string;
  expiresAt?: string;
  metadata?: Record<string, any>;
}

export interface BulkStockRankingAssignmentRequest {
  targetUsers: string[];
  packageTierId: string;
  stockRankingTypeId: string;
  expiresAt?: string;
  metadata?: Record<string, any>;
}

export interface AssignmentExtensionRequest {
  assignmentId: string;
  newExpiresAt: string;
  reason?: string;
}

// ============================================================================
// System Configuration API Types
// ============================================================================

export interface SystemConfigResponse {
  jwtSecretConfigured: boolean;
  apiBaseUrl: string;
  smtpConfigured: boolean;
  oauthConfigured: boolean;
  databaseConnected: boolean;
  redisConnected: boolean;
  version: string;
  environment: string;
}

export interface UpdateSystemConfigRequest {
  smtpSettings?: {
    host: string;
    port: number;
    username: string;
    password: string;
    fromEmail: string;
  };
  oauthSettings?: {
    clientId: string;
    clientSecret: string;
    redirectUrl: string;
  };
  features?: Record<string, boolean>;
}

export interface FeatureFlagsResponse {
  [key: string]: boolean;
}

export interface UpdateFeatureFlagsRequest {
  flags: Record<string, boolean>;
}

// ============================================================================
// File Upload API Types
// ============================================================================

export interface FileUploadResponse {
  fileId: string;
  fileName: string;
  fileSize: number;
  mimeType: string;
  uploadedAt: string;
  url: string;
}

export interface BulkUploadResponse {
  successful: FileUploadResponse[];
  failed: Array<{
    fileName: string;
    error: string;
  }>;
}

// ============================================================================
// Export/Import API Types
// ============================================================================

export interface ExportRequest {
  type: 'users' | 'permissions' | 'analytics' | 'notifications';
  format: 'csv' | 'json' | 'xlsx';
  filters?: Record<string, any>;
  includeDeleted?: boolean;
}

export interface ExportResponse {
  exportId: string;
  status: 'processing' | 'completed' | 'failed';
  downloadUrl?: string;
  expiresAt: string;
  message?: string;
}

export interface ImportRequest {
  type: 'users' | 'permissions';
  format: 'csv' | 'json';
  fileId: string;
  options?: {
    skipDuplicates?: boolean;
    updateExisting?: boolean;
    dryRun?: boolean;
  };
}

export interface ImportResponse {
  importId: string;
  status: 'processing' | 'completed' | 'failed';
  results?: {
    processed: number;
    successful: number;
    failed: number;
    skipped: number;
  };
  errors?: Array<{
    row: number;
    error: string;
  }>;
}

// ============================================================================
// WebSocket API Types
// ============================================================================

export interface WebSocketMessage {
  type: 'notification' | 'user_update' | 'system_alert' | 'permission_change';
  data: any;
  timestamp: string;
}

export interface NotificationWSMessage extends WebSocketMessage {
  type: 'notification';
  data: Notification;
}

export interface UserUpdateWSMessage extends WebSocketMessage {
  type: 'user_update';
  data: {
    userId: string;
    changes: Partial<User>;
  };
}

// ============================================================================
// Health Check Types
// ============================================================================

export interface HealthCheckResponse {
  status: 'healthy' | 'degraded' | 'unhealthy';
  timestamp: string;
  uptime: number;
  services: {
    database: 'up' | 'down';
    redis: 'up' | 'down';
    backend: 'up' | 'down';
    notifications: 'up' | 'down';
  };
  metrics: {
    responseTime: number;
    memoryUsage: number;
    activeConnections: number;
  };
}

// ============================================================================
// Type Guards for API Responses
// ============================================================================

export function isApiSuccess<T>(response: ApiResponse<T>): response is ApiResponse<T> & { success: true; data: T } {
  return response.success && response.data !== undefined;
}

export function isApiError(response: ApiResponse<any>): response is ApiResponse<any> & { success: false; error: string } {
  return !response.success && !!response.error;
}

export function isPaginatedResponse<T>(data: any): data is PaginatedResponse<T> {
  return data && typeof data === 'object' && Array.isArray(data.data) && data.pagination;
}