// Backend-compatible user type
export interface BackendUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  package_tier: string;
  expires_at: string;
  session_type: string;
  // Firebase compatibility properties
  uid?: string;
  emailVerified?: boolean;
  displayName?: string;
  photoURL?: string;
}

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

export interface AuthService {
  signInWithEmailAndPassword(credentials: SignInCredentials): Promise<BackendUser>;
  signUp(data: SignUpData): Promise<{ user_id: string; email: string; verification_sent: boolean; message: string }>;
  signOut(): Promise<void>;
  sendPasswordResetEmail(email: string): Promise<void>;
  getCurrentUser(): Promise<BackendUser | null>;
  refreshSession(): Promise<{ expires_at: string }>;
}