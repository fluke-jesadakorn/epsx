import type { User as FirebaseUser } from 'firebase/auth';

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
  signInWithEmailAndPassword(credentials: SignInCredentials): Promise<FirebaseUser>;
  signUp(data: SignUpData): Promise<FirebaseUser>;
  signInWithGoogle(): Promise<FirebaseUser>;
  signOut(): Promise<void>;
  sendPasswordResetEmail(email: string): Promise<void>;
  sendEmailVerification(user?: FirebaseUser): Promise<void>;
  updateUserProfile(data: { displayName?: string; photoURL?: string }): Promise<void>;
  linkGoogleAccount(): Promise<void>;
  unlinkProvider(providerId: string): Promise<void>;
  changePassword(currentPassword: string, newPassword: string): Promise<void>;
  getCurrentUserToken(forceRefresh?: boolean): Promise<string | null>;
}
