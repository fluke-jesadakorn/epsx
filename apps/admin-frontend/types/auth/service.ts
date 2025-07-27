// Backend user type instead of Firebase User
export interface BackendUser {
  user_id: string;
  email: string;
  role: string;
  expires_at?: string;
  displayName?: string;
  permissions?: string[];
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

export interface AuthService {
  signInWithEmailAndPassword(credentials: SignInCredentials): Promise<BackendUser>;
  signUp(data: SignUpData): Promise<BackendUser>;
  signOut(): Promise<void>;
  sendPasswordResetEmail(email: string): Promise<void>;
  updateUserProfile(data: { displayName?: string; photoURL?: string }): Promise<void>;
  changePassword(currentPassword: string, newPassword: string): Promise<void>;
  getCurrentUserToken(forceRefresh?: boolean): Promise<string | null>;
}
