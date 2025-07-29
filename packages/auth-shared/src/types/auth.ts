// Authentication credentials and service interfaces
export interface SignInCredentials {
  email: string;
  password: string;
}

export interface SignUpData {
  email: string;
  password: string;
  displayName?: string;
  package_tier?: string;
}

export interface UserCredentials {
  email: string;
  password: string;
}

// Auth context state
export interface AuthContextState {
  user: any | null;
  loading: boolean;
  initialized: boolean;
  error: string | null;
}

// Auth service interface
export interface AuthService {
  signInWithEmailAndPassword(credentials: SignInCredentials): Promise<any>;
  signUp(data: SignUpData): Promise<any>;
  signOut(): Promise<void>;
  sendPasswordResetEmail(email: string): Promise<void>;
  getCurrentUser(): Promise<any | null>;
  refreshSession?(): Promise<{ expires_at: string }>;
  updateUserProfile?(data: { displayName?: string; photoURL?: string }): Promise<void>;
  changePassword?(currentPassword: string, newPassword: string): Promise<void>;
  getCurrentUserToken?(forceRefresh?: boolean): Promise<string | null>;
}

// Auth result types
export interface AuthResult {
  isAuthenticated: boolean;
  user?: any;
  error?: string;
}

export interface AuthError {
  code: string;
  message: string;
  details?: any;
}