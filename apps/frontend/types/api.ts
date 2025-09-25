/**
 * FRONTEND API TYPES - MIGRATED TO SHARED
 * All API types moved to shared/types/api with compatibility layer
 * This file now re-exports shared types for backward compatibility
 */

// Re-export everything from shared API types
export * from '../../../shared/types/api';

// Import for local re-export with legacy names (maintaining compatibility)
import type {
  ApiResponse as SharedApiResponse,
  ApiError as SharedApiError,
  PaginatedResponse as SharedPaginatedResponse,
  AnalyticsQueryParams as SharedAnalyticsQueryParams,
  AnalyticsRankingItem as SharedAnalyticsRankingItem,
  QuarterlyEPSData as SharedQuarterlyEPSData,
  FilterOptions as SharedFilterOptions,
  CountryOption as SharedCountryOption,
  UserProfile as SharedUserProfile,
  UserPreferences as SharedUserPreferences,
  NotificationPreferences as SharedNotificationPreferences,
  AnalyticsPreferences as SharedAnalyticsPreferences,
  UserSubscription as SharedUserSubscription,
  LoginRequest as SharedLoginRequest,
  RegisterRequest as SharedRegisterRequest,
  AuthResponse as SharedAuthResponse,
  AuthTokens as SharedAuthTokens,
  PaymentRequest as SharedPaymentRequest,
  PaymentResponse as SharedPaymentResponse,
  PaymentStatus as SharedPaymentStatus,
  NotificationMessage as SharedNotificationMessage,
  WebSocketMessage as SharedWebSocketMessage,
  SearchRequest as SharedSearchRequest,
  SearchResponse as SharedSearchResponse
} from '../../../shared/types/api';

// Re-export with exact same names for backward compatibility
export type ApiResponse<T = unknown> = SharedApiResponse<T>;
export type ApiError = SharedApiError;
export type PaginatedResponse<T> = SharedPaginatedResponse<T>;

// Analytics types
export type AnalyticsQueryParams = SharedAnalyticsQueryParams;
export type AnalyticsRankingItem = SharedAnalyticsRankingItem;
export type QuarterlyEPSData = SharedQuarterlyEPSData;
export type FilterOptions = SharedFilterOptions;
export type CountryOption = SharedCountryOption;

// User management types
export type UserProfile = SharedUserProfile;
export type UserPreferences = SharedUserPreferences;
export type NotificationPreferences = SharedNotificationPreferences;
export type AnalyticsPreferences = SharedAnalyticsPreferences;
export type UserSubscription = SharedUserSubscription;

// Authentication types
export type LoginRequest = SharedLoginRequest;
export type RegisterRequest = SharedRegisterRequest;
export type AuthResponse = SharedAuthResponse;
export type AuthTokens = SharedAuthTokens;

// Payment types
export type PaymentRequest = SharedPaymentRequest;
export type PaymentResponse = SharedPaymentResponse;
export type PaymentStatus = SharedPaymentStatus;

// Notification types
export type NotificationMessage = SharedNotificationMessage;

// WebSocket types
export type WebSocketMessage<T = unknown> = SharedWebSocketMessage<T>;

// Search types
export type SearchRequest = SharedSearchRequest;
export type SearchResponse = SharedSearchResponse;