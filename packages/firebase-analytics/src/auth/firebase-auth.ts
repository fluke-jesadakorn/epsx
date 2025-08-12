// Firebase Authentication Manager
// Handles Firebase Authentication for multi-provider auth system

import { 
  getAuth, 
  signInWithEmailAndPassword,
  signInWithPopup,
  createUserWithEmailAndPassword,
  signOut,
  onAuthStateChanged,
  User as FirebaseUser,
  GoogleAuthProvider,
  GithubAuthProvider,
  UserCredential,
  Auth
} from 'firebase/auth';
import { initializeApp, getApps } from 'firebase/app';

// Firebase configuration (reusing from core)
const firebaseConfig = {
  apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
  authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
  projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
  storageBucket: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
  messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
  appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID,
  measurementId: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID
};

/**
 * Firebase authentication manager for multi-provider system
 */
export class FirebaseAuthManager {
  private auth: Auth;
  private googleProvider: GoogleAuthProvider;
  private githubProvider: GithubAuthProvider;

  constructor() {
    // Initialize Firebase app if not already initialized
    if (!getApps().length) {
      initializeApp(firebaseConfig);
    }

    this.auth = getAuth();
    this.googleProvider = new GoogleAuthProvider();
    this.githubProvider = new GithubAuthProvider();

    // Configure providers
    this.configureProviders();
  }

  private configureProviders(): void {
    // Configure Google provider
    this.googleProvider.addScope('email');
    this.googleProvider.addScope('profile');
    
    // Configure GitHub provider
    this.githubProvider.addScope('user:email');
    
    // Set custom parameters if needed
    this.googleProvider.setCustomParameters({
      prompt: 'select_account'
    });
  }

  /**
   * Sign in with email and password
   */
  async signInWithCredentials(email: string, password: string): Promise<FirebaseAuthResult> {
    try {
      const credential = await signInWithEmailAndPassword(this.auth, email, password);
      return this.createAuthResult(credential);
    } catch (error) {
      throw new FirebaseAuthError('Email/password sign in failed', error);
    }
  }

  /**
   * Sign in with Google OAuth
   */
  async signInWithGoogle(): Promise<FirebaseAuthResult> {
    try {
      const credential = await signInWithPopup(this.auth, this.googleProvider);
      return this.createAuthResult(credential);
    } catch (error) {
      throw new FirebaseAuthError('Google sign in failed', error);
    }
  }

  /**
   * Sign in with GitHub OAuth
   */
  async signInWithGitHub(): Promise<FirebaseAuthResult> {
    try {
      const credential = await signInWithPopup(this.auth, this.githubProvider);
      return this.createAuthResult(credential);
    } catch (error) {
      throw new FirebaseAuthError('GitHub sign in failed', error);
    }
  }

  /**
   * Create new user account with email and password
   */
  async createAccount(email: string, password: string): Promise<FirebaseAuthResult> {
    try {
      const credential = await createUserWithEmailAndPassword(this.auth, email, password);
      return this.createAuthResult(credential);
    } catch (error) {
      throw new FirebaseAuthError('Account creation failed', error);
    }
  }

  /**
   * Sign out current user
   */
  async signOut(): Promise<void> {
    try {
      await signOut(this.auth);
    } catch (error) {
      throw new FirebaseAuthError('Sign out failed', error);
    }
  }

  /**
   * Get current Firebase user
   */
  getCurrentUser(): FirebaseUser | null {
    return this.auth.currentUser;
  }

  /**
   * Get Firebase ID token for backend validation
   * This token will be sent to our backend token broker
   */
  async getIdToken(forceRefresh = false): Promise<string | null> {
    const user = this.getCurrentUser();
    if (!user) {
      return null;
    }

    try {
      return await user.getIdToken(forceRefresh);
    } catch (error) {
      throw new FirebaseAuthError('Failed to get ID token', error);
    }
  }

  /**
   * Get user claims from Firebase token
   */
  async getUserClaims(): Promise<Record<string, any> | null> {
    const user = this.getCurrentUser();
    if (!user) {
      return null;
    }

    try {
      const tokenResult = await user.getIdTokenResult();
      return tokenResult.claims;
    } catch (error) {
      throw new FirebaseAuthError('Failed to get user claims', error);
    }
  }

  /**
   * Listen to authentication state changes
   */
  onAuthStateChanged(callback: (user: FirebaseUser | null) => void): () => void {
    return onAuthStateChanged(this.auth, callback);
  }

  /**
   * Check if user is authenticated
   */
  isAuthenticated(): boolean {
    return this.getCurrentUser() !== null;
  }

  /**
   * Get user profile information
   */
  getUserProfile(): FirebaseUserProfile | null {
    const user = this.getCurrentUser();
    if (!user) {
      return null;
    }

    return {
      uid: user.uid,
      email: user.email,
      displayName: user.displayName,
      photoURL: user.photoURL,
      emailVerified: user.emailVerified,
      providerId: user.providerId,
      phoneNumber: user.phoneNumber,
      metadata: {
        creationTime: user.metadata.creationTime,
        lastSignInTime: user.metadata.lastSignInTime,
      }
    };
  }

  /**
   * Refresh the current user's token
   */
  async refreshToken(): Promise<string | null> {
    return this.getIdToken(true);
  }

  private createAuthResult(credential: UserCredential): FirebaseAuthResult {
    const user = credential.user;
    const profile = this.getUserProfile();

    return {
      user,
      profile,
      credential,
      isNewUser: (credential as any).additionalUserInfo?.isNewUser || false,
      providerId: credential.providerId,
    };
  }
}

/**
 * Firebase authentication result
 */
export interface FirebaseAuthResult {
  user: FirebaseUser;
  profile: FirebaseUserProfile | null;
  credential: UserCredential;
  isNewUser: boolean;
  providerId: string | null;
}

/**
 * Firebase user profile interface
 */
export interface FirebaseUserProfile {
  uid: string;
  email: string | null;
  displayName: string | null;
  photoURL: string | null;
  emailVerified: boolean;
  providerId: string;
  phoneNumber: string | null;
  metadata: {
    creationTime: string | undefined;
    lastSignInTime: string | undefined;
  };
}

/**
 * Firebase authentication error
 */
export class FirebaseAuthError extends Error {
  public readonly code: string;
  public readonly originalError: any;

  constructor(message: string, originalError?: any) {
    super(message);
    this.name = 'FirebaseAuthError';
    this.originalError = originalError;
    this.code = originalError?.code || 'unknown';
  }

  /**
   * Get user-friendly error message
   */
  getUserFriendlyMessage(): string {
    switch (this.code) {
      case 'auth/user-not-found':
        return 'No account found with this email address';
      case 'auth/wrong-password':
        return 'Incorrect password';
      case 'auth/email-already-in-use':
        return 'An account with this email address already exists';
      case 'auth/weak-password':
        return 'Password is too weak';
      case 'auth/invalid-email':
        return 'Invalid email address';
      case 'auth/popup-closed-by-user':
        return 'Sign-in was cancelled';
      case 'auth/popup-blocked':
        return 'Pop-up was blocked by browser';
      case 'auth/network-request-failed':
        return 'Network error occurred';
      default:
        return this.message;
    }
  }
}

/**
 * Firebase authentication configuration
 */
export interface FirebaseAuthConfig {
  enablePersistence?: boolean;
  tenantId?: string;
  languageCode?: string;
  customDomain?: string;
}

/**
 * Singleton instance for global access
 */
let firebaseAuthManager: FirebaseAuthManager | null = null;

/**
 * Get Firebase authentication manager instance
 */
export function getFirebaseAuth(): FirebaseAuthManager {
  if (!firebaseAuthManager) {
    firebaseAuthManager = new FirebaseAuthManager();
  }
  return firebaseAuthManager;
}

/**
 * Initialize Firebase authentication with configuration
 */
export function initializeFirebaseAuth(config?: FirebaseAuthConfig): FirebaseAuthManager {
  const manager = getFirebaseAuth();
  
  if (config) {
    // Apply configuration
    const auth = getAuth();
    
    if (config.tenantId) {
      auth.tenantId = config.tenantId;
    }
    
    if (config.languageCode) {
      auth.languageCode = config.languageCode;
    }
    
    if (config.customDomain) {
      // Configure custom domain if needed
    }
  }
  
  return manager;
}

export default FirebaseAuthManager;