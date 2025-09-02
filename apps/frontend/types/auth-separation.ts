/**
 * User-Focused Authentication Types for Main Frontend
 * Optimized for user experience and performance
 */

import { JWTPayload } from 'jose';

// ============================================================================
// User Authentication Types (Performance Optimized)
// ============================================================================

export interface UserJWTPayload extends JWTPayload {
  sub: string;
  email: string;
  name: string;
  iat: number;
  exp: number;
  iss: string;
  aud: string;
  
  // Token identification
  token_type: 'user_access';
  
  // Lightweight permissions structure
  permissions: {
    permissions: string[]; // Structured permissions: "epsx:resource:action"
    package_tier: string;
    expires_at?: number; // For time-limited permissions
  };
  
  // User context (minimal for performance)
  user_context: {
    package_tier: string;
    firebase_uid?: string;
    platform_preferences: string[];
  };
  
  // Platform information
  platforms: string[];
  primary_platform: string;
}

export interface UserSessionData {
  user: UserProfile;
  isLoggedIn: true;
  expiresAt: number;
  platformContext: {
    currentPlatform: string;
    availablePlatforms: string[];
  };
}

export interface UserProfile {
  id: string;
  email: string;
  name: string;
  role: 'user' | 'premium_user';
  
  // User-specific permissions (structured)
  permissions: string[];
  packageTier: string;
  
  // User metadata (lightweight)
  firebaseUid?: string;
  lastActivityAt?: string;
  
  // Platform context
  platforms: string[];
  primaryPlatform: string;
  platformContext?: string;
  
  // Analytics preferences
  analyticsPreferences?: {
    defaultView: 'cards' | 'table' | 'chart';
    autoRefresh: boolean;
    notifications: boolean;
  };
  
  // Trading preferences
  tradingPreferences?: {
    riskTolerance: 'low' | 'medium' | 'high';
    investmentHorizon: 'short' | 'medium' | 'long';
    preferredSectors: string[];
  };
}

// ============================================================================
// Package & Subscription Types
// ============================================================================

export enum PackageTier {
  FREE = 'FREE',
  BRONZE = 'BRONZE',
  SILVER = 'SILVER', 
  GOLD = 'GOLD',
  PLATINUM = 'PLATINUM',
  ENTERPRISE = 'ENTERPRISE'
}

export interface PackageFeatures {
  maxWatchlist: number;
  realTimeData: boolean;
  advancedAnalytics: boolean;
  customAlerts: number;
  exportData: boolean;
  apiAccess: boolean;
  prioritySupport: boolean;
  customReports: boolean;
}

export interface UserSubscription {
  id: string;
  userId: string;
  packageTier: PackageTier;
  features: PackageFeatures;
  startDate: string;
  endDate?: string;
  status: 'active' | 'cancelled' | 'expired' | 'trial';
  autoRenew: boolean;
  trialEndsAt?: string;
}

// ============================================================================
// Permission System Types (User-Focused)
// ============================================================================

export interface UserPermissionCheck {
  permission: string;
  platform?: string;
  requiresSubscription?: boolean;
}

export interface PermissionValidation {
  hasPermission: boolean;
  reason?: 'valid' | 'no_permission' | 'tier_insufficient' | 'expired';
  requiredTier?: PackageTier;
  upgradeUrl?: string;
}

// ============================================================================
// Analytics & Trading Types
// ============================================================================

export interface UserAnalyticsAccess {
  canViewRankings: boolean;
  canExportData: boolean;
  maxStocksTracked: number;
  realTimeAccess: boolean;
  advancedFilters: boolean;
  customIndicators: boolean;
}

export interface UserTradingAccess {
  paperTrading: boolean;
  liveTrading: boolean;
  advancedOrders: boolean;
  algorithmicTrading: boolean;
  portfolioAnalysis: boolean;
  riskManagement: boolean;
}

// ============================================================================
// Session Management Types
// ============================================================================

export interface UserSessionConfig {
  maxConcurrentSessions: number;
  sessionTimeout: number; // minutes
  requireReauth: boolean;
  rememberDevice: boolean;
}

export interface DeviceInfo {
  deviceId: string;
  deviceType: 'desktop' | 'mobile' | 'tablet';
  browser: string;
  lastUsed: string;
  trusted: boolean;
}

// ============================================================================
// Notification Types
// ============================================================================

export interface UserNotificationPreferences {
  priceAlerts: boolean;
  portfolioUpdates: boolean;
  marketNews: boolean;
  systemUpdates: boolean;
  promotionalOffers: boolean;
  
  // Delivery channels
  inApp: boolean;
  email: boolean;
  push: boolean;
  sms: boolean;
}

export interface UserAlert {
  id: string;
  type: 'price' | 'volume' | 'technical' | 'news';
  symbol: string;
  condition: string;
  value: number;
  triggered: boolean;
  createdAt: string;
  triggeredAt?: string;
}

// ============================================================================
// Authentication State Types
// ============================================================================

export interface AuthState {
  isAuthenticated: boolean;
  user: UserProfile | null;
  session: UserSessionData | null;
  loading: boolean;
  error: string | null;
}

export interface LoginCredentials {
  email: string;
  password: string;
  rememberDevice?: boolean;
}

export interface RegistrationData {
  email: string;
  password: string;
  name: string;
  acceptTerms: boolean;
  marketingConsent?: boolean;
}

// ============================================================================
// API Response Types
// ============================================================================

export interface AuthResponse {
  success: boolean;
  user?: UserProfile;
  accessToken?: string;
  refreshToken?: string;
  expiresIn?: number;
  error?: string;
}

export interface TokenRefreshResponse {
  accessToken: string;
  expiresIn: number;
  error?: string;
}

// ============================================================================
// Utility Types
// ============================================================================

export type AuthAction = 
  | 'login'
  | 'logout'
  | 'refresh'
  | 'verify'
  | 'register'
  | 'reset_password';

export interface AuthEvent {
  type: AuthAction;
  success: boolean;
  timestamp: string;
  metadata?: Record<string, any>;
}

// ============================================================================
// Type Guards
// ============================================================================

export function isUserJWT(payload: any): payload is UserJWTPayload {
  return payload?.token_type === 'user_access';
}

export function hasValidSubscription(user: UserProfile): boolean {
  return user.packageTier !== PackageTier.FREE;
}

export function canAccessFeature(user: UserProfile, feature: string): boolean {
  return user.permissions.some(p => 
    p.includes(feature) || 
    p.includes('*') ||
    p === 'epsx:*:*'
  );
}

export function isTrialUser(subscription: UserSubscription): boolean {
  return subscription.status === 'trial' && !!subscription.trialEndsAt;
}

export function isPremiumTier(tier: PackageTier): boolean {
  return [PackageTier.GOLD, PackageTier.PLATINUM, PackageTier.ENTERPRISE].includes(tier);
}