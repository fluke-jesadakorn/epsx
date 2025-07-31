import type { UserSubscription, PaymentTier } from '../payment';

// Base user interface that works for all contexts
export interface BaseUser {
  id: string;
  user_id?: string; // Backend compatibility
  uid?: string; // Firebase compatibility
  email: string;
  emailVerified?: boolean;
  displayName?: string;
  name?: string; // Alias for displayName
  photoURL?: string;
  avatar?: string; // Alias for photoURL
  createdAt?: string | Date;
  updatedAt?: string | Date;
}

// Core user interface with authentication data
export interface User extends BaseUser {
  role: string;
  isActive?: boolean;
  preferences?: UserPreferences;
}

// Extended user with role and permission information
export interface AuthenticatedUser extends User {
  permissions: string[];
  isAdmin: boolean;
  expires_at?: string;
  session_type?: string;
}

// Frontend-specific user with subscription data
export interface FrontendUser extends AuthenticatedUser {
  subscription?: UserSubscription;
  subscription_tier?: string;
  package_tier?: string;
  token_balance?: number;
  features?: string[];
  usdtDetails?: any; // External type compatibility
}

// Admin-specific user with enhanced permissions
export interface AdminUser extends AuthenticatedUser {
  roles: string[];
  permission_profiles?: string[];
  customClaims?: {
    role?: string;
    permissions?: string[];
    [key: string]: any;
  };
  metadata?: {
    creationTime?: string;
    lastSignInTime?: string;
    lastRefreshTime?: string;
  };
}

// Backend user response format
export interface BackendUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier?: string;
  package_tier?: string;
  expires_at?: string;
  session_type?: string;
  permission_profiles?: string[];
  emailVerified?: boolean;
  displayName?: string;
  photoURL?: string;
}

// Legacy alias for backward compatibility
export interface UserProfile extends FrontendUser {}

export interface UserPreferences {
  theme?: 'light' | 'dark' | 'system';
  language?: string;
  timezone?: string;
  notifications?: NotificationPreferences;
}

export interface NotificationPreferences {
  email: boolean;
  push: boolean;
  sms: boolean;
  marketing: boolean;
  updates: boolean;
}

export interface AuthCookies {
  session?: string;
  csrfToken?: string;
  refreshToken?: string;
}

export interface UserCredentials {
  email: string;
  password: string;
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

export interface PasswordChangeRequest {
  currentPassword: string;
  newPassword: string;
}

export interface PasswordResetRequest {
  email: string;
}

export interface ProfileUpdateRequest {
  name?: string;
  displayName?: string;
  avatar?: string;
  preferences?: Partial<UserPreferences>;
}

export interface RegistrationResponse {
  success: boolean;
  user?: UserProfile;
  token?: string;
  message?: string;
}