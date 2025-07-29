import type { UserSubscription } from '../payment';

export interface UserProfile {
  id: string;
  email: string;
  name?: string;
  displayName?: string;
  avatar?: string;
  role: string;
  isActive: boolean;
  createdAt: Date;
  updatedAt: Date;
  preferences?: UserPreferences;
  subscription?: UserSubscription;
}

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