// Core API Response Types
export interface ApiResponse<T = any> {
  data?: T;
  error?: string;
  details?: string;
  message?: string;
}

export interface ApiError {
  error: string;
  details?: string;
  status?: number;
}

// Authentication Types
export interface AuthCookies {
  session?: string;
  csrfToken?: string;
  refreshToken?: string;
}

export interface LoginRequest {
  type: 'credentials';
  email: string;
  password: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
  name?: string;
  package_tier?: string;
}

export interface EnhancedRegisterRequest {
  email: string;
  password: string;
  package_tier: string;
  referral_code?: string;
  source: string;
  region?: string;
  utm_source?: string;
  utm_campaign?: string;
}

export interface FeatureAssignmentResult {
  feature_id: string;
  profile_name: string;
  success: boolean;
  reason: string;
  expires_at?: string; // ISO string format
}

export interface RegistrationResponse {
  user_id: string;
  access_token: string;
  expires_in: number;
  features_unlocked: string[];
  total_features_assigned: number;
  assignment_results: FeatureAssignmentResult[];
}

export interface UserProfile {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  package_tier: string;
  expires_at: string;
  session_type: string;
}

export interface PasswordResetRequest {
  email: string;
}

export interface ProfileUpdateRequest {
  displayName?: string;
  photoURL?: string;
}

export interface PasswordChangeRequest {
  currentPassword: string;
  newPassword: string;
}

// Admin Types
export interface AdminUser {
  uid: string;
  email: string;
  emailVerified: boolean;
  displayName?: string;
  disabled: boolean;
  role?: string;
  permissions?: string[];
}

export interface UserListOptions {
  maxResults?: number;
  pageToken?: string;
  limit?: number;
  offset?: string;
}

export interface UserListResult {
  users: AdminUser[];
  pageToken?: string;
  total?: number;
}

// Payment Types
export interface CreatePaymentRequest {
  currency: string;
  amount: string;
  payment_method: 'on_line' | 'on_chain';
  product_name: string;
  notify_url?: string;
}

export interface CreatePaymentResponse {
  payment_id: string;
  amount: number;
  currency: string;
  status: string;
  payment_url?: string;
  expires_at: string;
}

export interface PaymentStatusResponse {
  payment_id: string;
  status: string;
  amount: number;
  currency: string;
  created_at: string;
  updated_at: string;
}

// Stock/Market Data Types
export interface StockFinancialData {
  symbol: string;
  company_name: string;
  price: number;
  eps: number;
  market_cap: number;
  pe_ratio: number;
  dividend_yield: number;
  updated_at: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
    startIndex: number;
    endIndex: number;
    currentPageSize: number;
  };
}

export interface CountResponse {
  count: number;
  timestamp: string;
  filters: {
    country: string;
    quarters: number;
  };
}

// Permission Profile Types
export interface PermissionProfile {
  id: string;
  name: string;
  description: string;
  category: string;
  target_tier: string;
  is_active: boolean;
  permissions_count: number;
  tags: string[];
  created_at: string;
  version: string;
  metadata: {
    requires_approval: boolean;
    max_assignments?: number;
    use_cases: string[];
    warnings: string[];
  };
}

// Permission Management Types
export interface Permission {
  id: string;
  name: string;
  description: string;
  resource: string;
  action: string;
  risk: 'low' | 'medium' | 'high';
}

export interface Role {
  id: string;
  name: string;
  permissions: string[];
  userCount: number;
  isSystem: boolean;
}

export interface UserPermissionStatus {
  userId: string;
  permissions: string[];
  profiles: string[];
  role: string;
  effectivePermissions: string[];
  hasWildcardAccess: boolean;
}

export interface PermissionCheckRequest {
  permission: string;
}

export interface PermissionCheckResponse {
  allowed: boolean;
}

export interface PermissionProfileAssignmentRequest {
  permission_profile_id: string;
  user_ids: string[];
  reason?: string;
  merge_permissions?: boolean;
  expires_at?: string;
  notify_users?: boolean;
}

export interface AssignmentResult {
  permission_profile_id: string;
  successful_assignments: Array<{
    user_id: string;
    features_unlocked: string[];
    permissions_added: string[];
    assignment_type: string;
  }>;
  failed_assignments: Array<{
    user_id: string;
    error: string;
    error_code: string;
  }>;
  total_assigned: number;
  total_failed: number;
  applied_at: string;
}

// HTTP Method Types
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

// Request Configuration
export interface RequestConfig {
  method?: HttpMethod;
  headers?: Record<string, string>;
  body?: string;
  credentials?: RequestCredentials;
}

// Utility Types
export type ApiSuccessResponse<T> = { data: T };
export type ApiErrorResponse = { error: string; details?: string };

// Stock Ranking Types
export interface StockRankingAssignment {
  id: string;
  user_id: string;
  package_tier: string;
  status: string;
  assigned_at: string;
  expires_at?: string;
  revoked_at?: string;
  assigned_by: string;
  revoked_by?: string;
  user?: {
    id: string;
    email: string;
    display_name?: string;
  };
}

export interface StockRankingAssignmentRequest {
  user_ids: string[];
  package_tier: string;
  expires_at?: string;
}

export interface StockRankingAssignmentExtendRequest {
  new_expires_at: string;
}

export interface StockRankingAssignmentUpdateRequest {
  status?: string;
  expires_at?: string;
  package_tier?: string;
}

// Analytics Types
export interface AnalyticsStatistics {
  users: {
    total: number;
    active: number;
    inactive: number;
    by_role: Record<string, number>;
  };
  stock_ranking: {
    total_assignments: number;
    active_assignments: number;
    expired_assignments: number;
    by_tier: Record<string, number>;
  };
  permission_profiles: {
    total_profiles: number;
    active_profiles: number;
    total_assignments: number;
  };
}

export interface StockRankingAnalytics {
  assignments_over_time: Array<{
    date: string;
    count: number;
  }>;
  tier_distribution: Record<string, number>;
  assignment_status: Record<string, number>;
  revenue_impact: {
    total_value: number;
    currency: string;
  };
}

// User Management Types
export interface UserSoftDeleteRequest {
  reason: string;
}

// Admin Profile Types
export interface AdminProfile {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  display_name?: string;
  last_login?: string;
  created_at: string;
}

// Action Result Types
export interface ActionResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

// Trading Types
export interface StockItem {
  symbol: string;
  company_name: string;
  price: number;
  change: number;
  change_percent: number;
  volume?: number;
  market_cap?: number;
  pe_ratio?: number;
  dividend_yield?: number;
  eps?: number;
  updated_at: string;
}

export interface PortfolioItem {
  symbol: string;
  company_name: string;
  shares: number;
  avgCost: number;
  currentPrice: number;
  totalValue: number;
  gainLoss: number;
  gainLossPercent: number;
  updated_at: string;
}

export interface StockRanking {
  symbol: string;
  company_name: string;
  rank: number;
  score: number;
  category: string;
  recommendation: 'BUY' | 'SELL' | 'HOLD';
  price: number;
  target_price: number;
  updated_at: string;
}

export interface PriceAlert {
  id: string;
  symbol: string;
  type: 'above' | 'below';
  targetPrice: number;
  currentPrice: number;
  isActive: boolean;
  triggered: boolean;
  createdAt: string;
  triggeredAt?: string;
}

export interface WatchlistAddRequest {
  symbol: string;
}

export interface PriceAlertCreateRequest {
  symbol: string;
  type: 'above' | 'below';
  targetPrice: number;
}

// Notification Types
export interface Notification {
  id: string;
  title: string;
  message: string;
  type: 'trading' | 'system' | 'payment' | 'security' | 'feature';
  priority: 'low' | 'medium' | 'high' | 'urgent';
  read: boolean;
  actionUrl?: string;
  metadata?: Record<string, any>;
  createdAt: string;
  updatedAt?: string;
}

export interface NotificationPreferences {
  email: boolean;
  push: boolean;
  inApp: boolean;
  tradingAlerts: boolean;
  systemUpdates: boolean;
  paymentNotifications?: boolean;
  securityAlerts?: boolean;
}

export interface NotificationListResponse {
  notifications: Notification[];
  unreadCount: number;
  total?: number;
}

export interface PushSubscriptionRequest {
  endpoint: string;
  keys: {
    p256dh: string;
    auth: string;
  };
}

// Type Guards
export const isApiError = (response: ApiResponse): response is ApiErrorResponse => {
  return 'error' in response && !!response.error;
};

export const isApiSuccess = <T>(response: ApiResponse<T>): response is ApiSuccessResponse<T> => {
  return 'data' in response && !response.error;
};