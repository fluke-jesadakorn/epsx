/**
 * CONSOLIDATED API TYPES
 * Unified API type definitions shared across admin-frontend and frontend
 * Combines comprehensive admin API types with user-focused API types
 */

// ============================================================================
// BASE API RESPONSE TYPES
// ============================================================================

// ============================================================================
// BASE API RESPONSE TYPES
// ============================================================================

/**
 * Standard API Response Wrapper
 * All API endpoints must return this structure.
 */
export interface ApiResponse<T = any> {
  success: boolean;       // Check this first
  data: T | null;         // Payload if success=true, null if error
  error: ApiError | null; // Error details if success=false, null if success
  meta?: ApiMeta;         // Optional metadata (pagination, trace_id, execution time)
}

/**
 * Standard API Error Structure
 */
export interface ApiError {
  code: string;           // Machine-parsable (e.g., "VALIDATION_ERROR", "UNAUTHORIZED")
  message: string;        // Human-readable message
  details?: Record<string, any>; // detailed validation errors or context
  requestId?: string;     // Optional trace ID for debugging
}

/**
 * Standard API Metadata
 */
export interface ApiMeta {
  page?: number;
  per_page?: number;
  total?: number;
  total_pages?: number;
  trace_id?: string;
  timestamp?: string;
  [key: string]: any;     // Allow extensibility
}

export interface ApiErrorDetail {
  field?: string;
  message: string;
  code?: string;
}

export interface ActionResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

// ============================================================================
// PAGINATION TYPES
// ============================================================================

export interface PaginationParams {
  page?: number;
  limit?: number;
  offset?: number;
  sort_by?: string;
  sort_order?: 'asc' | 'desc';
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
  };
  metadata?: Record<string, unknown>;
}

// ============================================================================
// AUTHENTICATION API TYPES
// ============================================================================

export interface LoginRequest {
  email: string;
  password: string;
  rememberMe?: boolean;
  remember_me?: boolean;
  device_info?: DeviceInfo;
}

export interface RegisterRequest {
  email: string;
  password: string;
  name?: string;
  terms_accepted: boolean;
  marketing_consent?: boolean;
}

export interface LoginResponse {
  user: UserProfile;
  tokens: AuthTokens;
  permissions?: string[];
  subscription?: UserSubscription;
}

export interface AuthTokens {
  access_token: string;
  refresh_token: string;
  id_token?: string;
  token_type: 'Bearer';
  expires_in: number;
  expires_at?: number;
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
  user?: UserProfile;
  tokens?: AuthTokens;
  error?: string;
  redirectUrl?: string;
}

export interface DeviceInfo {
  user_agent?: string;
  ip_address?: string;
  device_type?: 'desktop' | 'mobile' | 'tablet';
  os?: string;
  browser?: string;
}

// ============================================================================
// USER PROFILE TYPES
// ============================================================================

export interface UserProfile {
  id: string;
  email: string;
  name: string;
  display_name?: string;
  photo_url?: string;
  email_verified?: boolean;
  phone_number?: string;
  role?: string;
  created_at: string;
  updated_at: string;
  last_sign_in_at?: string;
  lastActivityAt?: string;
  metadata?: Record<string, unknown>;

  // Permission-related fields
  permissions?: string[];
  packageTier?: string;

  // Web3 fields
  walletAddress?: string;
  firebaseUid?: string;

  // Platform context
  platforms?: string[];
  primaryPlatform?: string;
  platformContext?: string;
}

export interface UserPreferences {
  language: string;
  timezone: string;
  currency: string;
  theme: 'light' | 'dark' | 'system';
  notifications: NotificationPreferences;
  analytics: AnalyticsPreferences;
  privacy?: PrivacyPreferences;
}

export interface NotificationPreferences {
  email: boolean;
  push: boolean;
  in_app: boolean;
  analytics_alerts: boolean;
  price_alerts?: boolean;
  system_updates: boolean;
  analyticsAlerts?: boolean;
  securityAlerts?: boolean;
  complianceNotifications?: boolean;
  marketing?: boolean;
}

export interface AnalyticsPreferences {
  default_view: 'grid' | 'list' | 'chart';
  auto_refresh: boolean;
  refresh_interval: number;
  default_country?: string;
  default_sector?: string;
  page_size: number;
}

export interface PrivacyPreferences {
  share_analytics: boolean;
  data_retention_days?: number;
  allow_cookies: boolean;
}

// ============================================================================
// USER MANAGEMENT API TYPES (ADMIN-FOCUSED)
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
// PERMISSION MANAGEMENT API TYPES (ADMIN-FOCUSED)
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
// SUBSCRIPTION TYPES
// ============================================================================

export interface UserSubscription {
  id: string;
  user_id?: string;
  plan_id?: string;
  plan_name?: string;
  tier?: string;
  status: 'active' | 'inactive' | 'expired' | 'cancelled';
  valid_until?: string;
  started_at?: string;
  expires_at?: string;
  created_at?: string;
  updated_at?: string;
  features?: string[];
  limits?: SubscriptionLimits;
  billing?: BillingInfo;
  auto_renew?: boolean;
  current_usage?: Record<string, any>;
  quota_limits?: Record<string, any>;
  metadata?: Record<string, any>;
  access_context?: string;
  api_key?: string;
  api_key_name?: string;
  last_billed_at?: string;
  next_billing_date?: string;
}

export interface SubscriptionLimits {
  api_requests_per_day?: number;
  data_exports_per_month?: number;
  watchlist_items?: number;
  price_alerts?: number;
  custom_filters?: number;
}

export interface BillingInfo {
  amount: number;
  currency: string;
  billing_cycle: 'monthly' | 'yearly' | 'pay_per_use'; // Platform uses pay_per_use
  next_billing_date?: string;
  payment_method?: string;
}

// Analytics types moved to analytics.ts

// Notification types moved to notifications.ts

// ============================================================================
// PAYMENT API TYPES
// ============================================================================

export interface PaymentRequest {
  amount: number;
  currency: string;
  payment_method: string;
  product_id?: string;
  product_name?: string;
  customer_info: CustomerInfo;
  metadata?: Record<string, unknown>;
}

export interface CustomerInfo {
  email: string;
  name?: string;
  phone?: string;
  address?: Address;
}

export interface Address {
  street: string;
  city: string;
  state?: string;
  postal_code: string;
  country: string;
}

export interface PaymentResponse {
  payment_id?: string;
  id?: string;
  order_id?: string;
  order_no?: string;
  status: PaymentStatus;
  amount: number;
  currency: string;
  payment_url?: string;
  checkout_url?: string;
  qr_code?: string;
  expires_at?: string;
  expiration_date?: string;
  receive_address?: string;
  created_at?: string;
  updated_at?: string;
}

export type PaymentStatus =
  | 'pending'
  | 'processing'
  | 'completed'
  | 'succeeded'
  | 'failed'
  | 'cancelled'
  | 'expired'
  | 'refunded'
  | 'RequiresAction';

export interface PaymentStatusUpdate {
  payment_id: string;
  status: PaymentStatus;
  transaction_id?: string;
  failure_reason?: string;
  updated_at: string;
}

export interface PaymentTransaction {
  id: string;
  amount: number;
  currency: string;
  status: string;
  created_at: string;
  description?: string;
  user_id?: string;
  tier?: string;
}

// ============================================================================
// PERFORMANCE METRICS TYPES
// ============================================================================

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

// ============================================================================
// WEBSOCKET API TYPES
// ============================================================================

export interface WebSocketMessage<T = unknown> {
  type: string;
  id?: string;
  timestamp: string;
  data: T;
  channel?: string;
  user_id?: string;
}

export interface WebSocketError {
  type: 'error';
  message: string;
  code?: string;
  timestamp: string;
}

// NotificationWSMessage moved to notifications.ts or consolidated

export interface UserUpdateWSMessage extends WebSocketMessage {
  type: 'user_update';
  data: {
    userId: string;
    changes: Partial<UserProfile>;
  };
}

// ============================================================================
// FILE UPLOAD TYPES
// ============================================================================

export interface FileUploadRequest {
  file: File;
  type: 'avatar' | 'document' | 'image' | 'data_export';
  metadata?: Record<string, unknown>;
}

export interface FileUploadResponse {
  fileId?: string;
  file_id?: string;
  fileName?: string;
  filename?: string;
  file_type?: string;
  fileSize?: number;
  file_size?: number;
  mimeType?: string;
  uploadedAt?: string;
  url: string;
  thumbnail_url?: string;
  expires_at?: string;
}

export interface BulkUploadResponse {
  successful: FileUploadResponse[];
  failed: Array<{
    fileName: string;
    error: string;
  }>;
}

// ============================================================================
// SEARCH TYPES
// ============================================================================

export interface SearchRequest {
  query: string;
  type?: 'stocks' | 'companies' | 'all';
  limit?: number;
  filters?: Record<string, unknown>;
}

export interface SearchResult {
  type: 'stock' | 'company';
  id: string;
  symbol?: string;
  name: string;
  description?: string;
  market?: string;
  sector?: string;
  relevance_score: number;
  metadata?: Record<string, unknown>;
}

export interface SearchResponse {
  query: string;
  total_results: number;
  results: SearchResult[];
  suggestions?: string[];
  facets?: Record<string, SearchFacet[]>;
}

export interface SearchFacet {
  value: string;
  count: number;
  selected?: boolean;
}

// ============================================================================
// SYSTEM CONFIGURATION TYPES
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
// HEALTH CHECK TYPES
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
// IMPORT/EXPORT TYPES
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
// UTILITY TYPES
// ============================================================================

export type JsonRequestBody = Record<string, unknown> | unknown[] | string | number | boolean | null;

export interface RequestConfig extends Omit<RequestInit, 'body'> {
  body?: JsonRequestBody;
  timeout?: number;
  retries?: number;
}

export type ApiResponseData<T extends (...args: unknown[]) => Promise<ApiResponse<unknown>>> =
  T extends (...args: unknown[]) => Promise<ApiResponse<infer R>> ? R : never;

export type ApiMethodParams<T extends (...args: unknown[]) => unknown> = Parameters<T>;

export type ApiStatus = 'idle' | 'loading' | 'success' | 'error';

export interface ApiState<T> {
  data: T | null;
  status: ApiStatus;
  error: ApiError | null;
  lastUpdated: number | null;
}

// ============================================================================
// TYPE GUARDS AND UTILITIES
// ============================================================================

export function isApiSuccess<T>(response: ApiResponse<T>): response is ApiResponse<T> & { success: true; data: T } {
  return response.success && response.data !== undefined;
}

export function isApiError(error: unknown): error is ApiError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    'code' in error &&
    typeof (error as any).message === 'string' &&
    typeof (error as any).code === 'string'
  );
}

export function isApiResponse<T>(response: unknown): response is ApiResponse<T> {
  return (
    typeof response === 'object' &&
    response !== null &&
    'data' in response &&
    'error' in response &&
    'success' in response &&
    typeof (response as ApiResponse).success === 'boolean'
  );
}

export function isPaginatedResponse<T>(data: any): data is PaginatedResponse<T> {
  return data && typeof data === 'object' && Array.isArray(data.data) && data.pagination;
}

// Validation helpers
export function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

export function isValidPassword(password: string): boolean {
  // At least 8 characters, 1 uppercase, 1 lowercase, 1 number
  const passwordRegex = /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[a-zA-Z\d@$!%*?&]{8,}$/;
  return passwordRegex.test(password);
}

export function isValidCurrency(currency: string): boolean {
  // ISO 4217 currency codes
  const validCurrencies = ['USD', 'EUR', 'GBP', 'JPY', 'CAD', 'AUD', 'CHF', 'CNY'];
  return validCurrencies.includes(currency.toUpperCase());
}