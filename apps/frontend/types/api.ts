/**
 * Comprehensive API Type Definitions
 * Replaces all `any` types with proper TypeScript interfaces
 * Provides type safety for all API interactions
 */

// Base API response structure
export interface ApiResponse<T = unknown> {
  data: T;
  status: number;
  success: boolean;
  message?: string;
  timestamp?: string;
  requestId?: string;
}

// API Error types
export interface ApiError {
  message: string;
  status: number;
  code?: string;
  details?: ApiErrorDetail[];
  timestamp?: string;
  requestId?: string;
}

export interface ApiErrorDetail {
  field?: string;
  message: string;
  code?: string;
}

// HTTP request body types
export type JsonRequestBody = Record<string, unknown> | unknown[] | string | number | boolean | null;

export interface RequestConfig extends Omit<RequestInit, 'body'> {
  body?: JsonRequestBody;
  timeout?: number;
  retries?: number;
}

// Pagination types
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

// Analytics API types
export interface AnalyticsQueryParams extends PaginationParams {
  country?: string;
  sector?: string;
  min_eps?: number;
  max_eps?: number;
  min_growth?: number;
  max_growth?: number;
  min_market_cap?: number;
  max_market_cap?: number;
}

export interface AnalyticsRankingItem {
  symbol: string;
  name: string;
  country: string;
  sector: string;
  exchange: string;
  current_eps: number | null;
  growth_factor: number | null;
  price_current: number | null;
  market_cap: number | null;
  volume: number | null;
  pe_ratio?: number | null;
  dividend_yield?: number | null;
  price_change?: number | null;
  price_change_pct?: number | null;
  ranking_position: number;
  active_status: string;
  quarterly_data?: QuarterlyEPSData[];
}

export interface QuarterlyEPSData {
  quarter: string;
  year: number;
  eps: number | null;
  eps_growth: number | null;
  price_growth: number | null;
  revenue?: number | null;
  revenue_growth?: number | null;
}

export interface FilterOptions {
  countries: CountryOption[];
  sectors: string[];
  exchanges?: string[];
  stock_types?: string[];
}

export interface CountryOption {
  value: string;
  label: string;
  flag?: string;
  market_count?: number;
}

// User management types
export interface UserProfile {
  id: string;
  email: string;
  name?: string;
  display_name?: string;
  photo_url?: string;
  email_verified: boolean;
  phone_number?: string;
  created_at: string;
  updated_at: string;
  last_sign_in_at?: string;
  metadata?: Record<string, unknown>;
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
  trading_alerts: boolean;
  price_alerts: boolean;
  system_updates: boolean;
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

export interface UserSubscription {
  id: string;
  tier: string;
  plan_name: string;
  status: 'active' | 'inactive' | 'expired' | 'cancelled';
  valid_until: string;
  features: string[];
  limits: SubscriptionLimits;
  billing?: BillingInfo;
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
  billing_cycle: 'monthly' | 'yearly';
  next_billing_date?: string;
  payment_method?: string;
}

// Authentication types  
export interface LoginRequest {
  email: string;
  password: string;
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

export interface AuthResponse {
  user: UserProfile;
  tokens: AuthTokens;
  permissions: string[];
  subscription?: UserSubscription;
}

export interface AuthTokens {
  access_token: string;
  refresh_token: string;
  id_token?: string;
  token_type: 'Bearer';
  expires_in: number;
  expires_at: number;
}

export interface DeviceInfo {
  user_agent?: string;
  ip_address?: string;
  device_type?: 'desktop' | 'mobile' | 'tablet';
  os?: string;
  browser?: string;
}

// Payment types
export interface PaymentRequest {
  amount: number;
  currency: string;
  payment_method: string;
  product_id: string;
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
  payment_id: string;
  order_id: string;
  status: PaymentStatus;
  amount: number;
  currency: string;
  payment_url?: string;
  qr_code?: string;
  expires_at?: string;
}

export type PaymentStatus = 
  | 'pending' 
  | 'processing' 
  | 'completed' 
  | 'failed' 
  | 'cancelled' 
  | 'expired' 
  | 'refunded';

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

// Notification types
export interface NotificationMessage {
  id: string;
  user_id: string;
  type: NotificationType;
  category: NotificationCategory;
  priority: NotificationPriority;
  title: string;
  body: string;
  data?: Record<string, unknown>;
  channels: NotificationChannel[];
  status: NotificationStatus;
  created_at: string;
  scheduled_for?: string;
  sent_at?: string;
  read_at?: string;
  expires_at?: string;
}

export type NotificationType = 
  | 'info' 
  | 'success' 
  | 'warning' 
  | 'error' 
  | 'marketing' 
  | 'system';

export type NotificationCategory = 
  | 'trading' 
  | 'account' 
  | 'billing' 
  | 'security' 
  | 'system' 
  | 'marketing' 
  | 'price_alert';

export type NotificationPriority = 'low' | 'normal' | 'high' | 'urgent';

export type NotificationChannel = 'email' | 'push' | 'in_app' | 'sms';

export type NotificationStatus = 
  | 'pending' 
  | 'sent' 
  | 'delivered' 
  | 'read' 
  | 'failed' 
  | 'expired';

// Websocket types
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

// File upload types
export interface FileUploadRequest {
  file: File;
  type: 'avatar' | 'document' | 'image' | 'data_export';
  metadata?: Record<string, unknown>;
}

export interface FileUploadResponse {
  file_id: string;
  filename: string;
  file_type: string;
  file_size: number;
  url: string;
  thumbnail_url?: string;
  expires_at?: string;
}

// Search types
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

// Generic utility types
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

// Type guards
export function isApiError(error: unknown): error is ApiError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    'status' in error &&
    typeof (error as ApiError).message === 'string' &&
    typeof (error as ApiError).status === 'number'
  );
}

export function isApiResponse<T>(response: unknown): response is ApiResponse<T> {
  return (
    typeof response === 'object' &&
    response !== null &&
    'data' in response &&
    'status' in response &&
    'success' in response &&
    typeof (response as ApiResponse).status === 'number' &&
    typeof (response as ApiResponse).success === 'boolean'
  );
}

// Request validation helpers
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