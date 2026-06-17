/**
 * FRONTEND API TYPES - MIGRATED TO SHARED
 * All API types moved to shared/types/api with compatibility layer
 * This file now re-exports shared types for backward compatibility
 */

// Import for local re-export with legacy names (maintaining compatibility)
import type {
  AnalyticsPreferences as SharedAnalyticsPreferences,
  ApiError as SharedApiError,
  ApiResponse as SharedApiResponse,
  LoginResponse as SharedAuthResponse,
  AuthTokens as SharedAuthTokens,
  LoginRequest as SharedLoginRequest,
  NotificationPreferences as SharedNotificationPreferences,
  PaginatedResponse as SharedPaginatedResponse,
  PaymentRequest as SharedPaymentRequest,
  PaymentResponse as SharedPaymentResponse,
  PaymentStatus as SharedPaymentStatus,
  RegisterRequest as SharedRegisterRequest,
  SearchRequest as SharedSearchRequest,
  SearchResponse as SharedSearchResponse,
  UserPreferences as SharedUserPreferences,
  UserProfile as SharedUserProfile,
  UserSubscription as SharedUserSubscription,
  WebSocketMessage as SharedWebSocketMessage
} from '@/shared/types/api';

import type {
  AnalyticsQueryParams as SharedAnalyticsQueryParams,
  AnalyticsRankingItem as SharedAnalyticsRankingItem,
  CountryOption as SharedCountryOption,
  FilterOptions as SharedFilterOptions,
  QuarterlyEPSData as SharedQuarterlyEPSData,
} from '@/shared/types/analytics';

import type {
  NotificationWSMessage as SharedNotificationMessage,
} from '@/shared/types/notifications';

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