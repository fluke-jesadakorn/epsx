// Re-export auth types from the main types package to eliminate duplication
export type {
  UserProfile,
  LoginRequest,
  RegisterRequest,
  EnhancedRegisterRequest,
  PasswordChangeRequest,
  PasswordResetRequest,
  ProfileUpdateRequest,
  RegistrationResponse,
  AuthCookies,
  UserPreferences,
  NotificationPreferences,
  AuthenticatedUser,
  FrontendUser
} from '@epsx/types';

// Import types for local usage  
import type { UserProfile, RegistrationResponse } from '@epsx/types';

// Auth-shared specific extensions and context types
export interface AuthContextState {
  user: UserProfile | null;
  loading: boolean;
  initialized: boolean;
  error: string | null;
}

// Extended auth service interface for compatibility
export interface AuthService {
  signInWithEmailAndPassword(credentials: { email: string; password: string }): Promise<UserProfile>;
  signUp(data: { email: string; password: string; name?: string; package_tier?: string }): Promise<RegistrationResponse>;
  signOut(): Promise<void>;
  sendPasswordResetEmail(email: string): Promise<void>;
  getCurrentUser(): Promise<UserProfile | null>;
  refreshSession?(): Promise<{ expires_at: string }>;
  updateUserProfile?(data: { name?: string; displayName?: string; avatar?: string }): Promise<void>;
  changePassword?(currentPassword: string, newPassword: string): Promise<void>;
  getCurrentUserToken?(forceRefresh?: boolean): Promise<string | null>;
}

// Auth result types for backward compatibility
export interface AuthResult {
  isAuthenticated: boolean;
  user?: UserProfile;
  error?: string;
}

export interface AuthError {
  code: string;
  message: string;
  details?: any;
}

// Legacy user types for backward compatibility - deprecated, use types from @epsx/types instead
/** @deprecated Use BaseUser from @epsx/types instead */
export interface LegacyBaseUser {
  id: string;
  user_id?: string;
  uid?: string;
  email: string;
  emailVerified?: boolean;
  displayName?: string;
  photoURL?: string;
  createdAt?: string;
  updatedAt?: string;
}

/** @deprecated Use AuthenticatedUser from @epsx/types instead */
export interface LegacyAuthenticatedUser extends LegacyBaseUser {
  role: string;
  permissions: string[];
  isAdmin: boolean;
  expires_at?: string;
  session_type?: string;
}

/** @deprecated Use FrontendUser from @epsx/types instead */
export interface LegacyFrontendUser extends LegacyAuthenticatedUser {
  subscription?: any;
  token_balance?: number;
  features?: string[];
  usdtDetails?: any;
  subscription_tier?: string;
  package_tier?: string;
}

/** @deprecated Use AdminUser from @epsx/types instead */
export interface LegacyAdminUser extends LegacyAuthenticatedUser {
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

/** @deprecated Use UserProfile from @epsx/types instead */  
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

// Legacy credential types for backward compatibility - deprecated
/** @deprecated Use LoginRequest from @epsx/types instead */
export interface SignInCredentials {
  email: string;
  password: string;
}

/** @deprecated Use RegisterRequest from @epsx/types instead */
export interface SignUpData {
  email: string;
  password: string;
  displayName?: string;
  package_tier?: string;
}

/** @deprecated Use LoginRequest from @epsx/types instead */
export interface UserCredentials {
  email: string;
  password: string;
}