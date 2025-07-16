// Common session and authentication types used across the monorepo
export interface SessionClaims {
  uid: string;
  email?: string;
  email_verified?: boolean;
  name?: string;
  picture?: string;
  exp?: number;
  iat?: number;
  role?: string;
  permissions?: string[];
  custom_claims?: Record<string, any>;
}

export interface SessionResult {
  success: boolean;
  claims?: SessionClaims;
  needsRefresh?: boolean;
  error?: string;
}

export interface AuthState {
  user: any | null; // Firebase User
  loading: boolean;
  error: string | null;
  isInitialized: boolean;
}

export interface SignInCredentials {
  email: string;
  password: string;
}

export interface SignUpData {
  email: string;
  password: string;
  displayName?: string;
}

export interface AuthActions {
  signInWithEmailAndPassword: (credentials: SignInCredentials) => Promise<void>;
  signInWithGoogle: () => Promise<void>;
  signUp: (data: SignUpData) => Promise<void>;
  signOut: () => Promise<void>;
  sendPasswordResetEmail: (email: string) => Promise<void>;
  sendEmailVerification: () => Promise<void>;
  clearError: () => void;
  refreshSession: () => Promise<void>;
}

export type AuthContextType = AuthState & AuthActions;

// Admin-specific types
export interface AdminAuthState extends AuthState {
  isAdmin: boolean;
}

export interface AdminAuthActions {
  signIn: (email: string, password: string) => Promise<void>;
  signOut: () => Promise<void>;
  checkAdminStatus: () => Promise<boolean>;
  clearError: () => void;
}

export type AdminAuthContextType = AdminAuthState & AdminAuthActions;

// Session configuration
export interface SessionConfig {
  sessionKey: string;
  maxAge: number;
  refreshThreshold: number;
  secure?: boolean;
  sameSite?: 'strict' | 'lax' | 'none';
  httpOnly?: boolean;
}

// Auth service configuration
export interface AuthConfig {
  apiUrl: string;
  endpoints: {
    login: string;
    register: string;
    logout: string;
    refresh: string;
    me: string;
  };
  tokenKey: string;
  refreshTokenKey: string;
  tokenType: string;
}
