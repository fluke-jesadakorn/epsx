// Base user interface that works for both frontend and admin contexts
export interface BaseUser {
  id: string;
  user_id?: string; // Backend compatibility
  uid?: string; // Firebase compatibility
  email: string;
  emailVerified?: boolean;
  displayName?: string;
  photoURL?: string;
  createdAt?: string;
  updatedAt?: string;
}

// Extended user with role and permission information
export interface AuthenticatedUser extends BaseUser {
  role: string;
  permissions: string[];
  isAdmin: boolean;
  expires_at?: string;
  session_type?: string;
}

// Frontend-specific user with subscription data
export interface FrontendUser extends AuthenticatedUser {
  subscription?: any; // Generic subscription type to avoid dependency
  token_balance?: number;
  features?: string[];
  usdtDetails?: any; // Type from frontend
  subscription_tier?: string;
  package_tier?: string;
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