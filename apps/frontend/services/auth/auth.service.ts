'use client';

import {
  createUserWithEmailAndPassword,
  GoogleAuthProvider,
  signInWithEmailAndPassword,
  signInWithPopup,
  signOut as firebaseSignOut,
  sendPasswordResetEmail,
  sendEmailVerification,
  updateProfile,
  linkWithPopup,
  unlink,
  updatePassword,
  reauthenticateWithCredential,
  EmailAuthProvider,
} from 'firebase/auth';
import type { 
  User as FirebaseUser,
  AuthError as FirebaseAuthError,
} from 'firebase/auth';

import { auth } from '@/lib/firebase';
import { AuthError, ErrorCode } from '@/types/auth/errors';
import type { AuthService as IAuthService, SignInCredentials, SignUpData } from '@/types/auth/service';

export class AuthService implements IAuthService {
  private static instance: AuthService;

  public static getInstance(): AuthService {
    if (!AuthService.instance) {
      AuthService.instance = new AuthService();
    }
    return AuthService.instance;
  }

  private constructor() {}

  /**
   * Sign in with email and password
   */
  async signInWithEmailAndPassword(credentials: SignInCredentials): Promise<FirebaseUser> {
    try {
      const { email, password } = credentials;
      const userCredential = await signInWithEmailAndPassword(auth, email, password);
      return userCredential.user;
    } catch (error) {
      throw this.handleAuthError(error as FirebaseAuthError);
    }
  }

  /**
   * Sign up with email and password
   */
  async signUp(data: SignUpData): Promise<FirebaseUser> {
    try {
      const { email, password, displayName } = data;
      const userCredential = await createUserWithEmailAndPassword(auth, email, password);
      
      // Update profile with display name if provided
      if (displayName) {
        await updateProfile(userCredential.user, { displayName });
      }

      // Send email verification
      await this.sendEmailVerification(userCredential.user);
      
      return userCredential.user;
    } catch (error) {
      throw this.handleAuthError(error as FirebaseAuthError);
    }
  }

  /**
   * Sign in with Google
   */
  async signInWithGoogle(): Promise<FirebaseUser> {
    try {
      const provider = new GoogleAuthProvider();
      provider.addScope('email');
      provider.addScope('profile');
      
      const userCredential = await signInWithPopup(auth, provider);
      return userCredential.user;
    } catch (error) {
      // Handle popup closed error specifically
      if ((error as FirebaseAuthError).code === 'auth/popup-closed-by-user') {
        throw new AuthError(ErrorCode.USER_CANCELLED, 'Sign-in was cancelled');
      }
      throw this.handleAuthError(error as FirebaseAuthError);
    }
  }

  /**
   * Sign out
   */
  async signOut(): Promise<void> {
    try {
      await firebaseSignOut(auth);
    } catch (error) {
      throw this.handleAuthError(error as FirebaseAuthError);
    }
  }

  /**
   * Send password reset email
   */
  async sendPasswordResetEmail(email: string): Promise<void> {
    try {
      await sendPasswordResetEmail(auth, email);
    } catch (error) {
      throw this.handleAuthError(error as FirebaseAuthError);
    }
  }

  /**
   * Send email verification
   */
  async sendEmailVerification(user?: FirebaseUser): Promise<void> {
    try {
      const currentUser = user || auth.currentUser;
      if (!currentUser) {
        throw new AuthError(ErrorCode.USER_NOT_FOUND, 'No user found');
      }
      await sendEmailVerification(currentUser);
    } catch (error) {
      throw this.handleAuthError(error as FirebaseAuthError);
    }
  }

  /**
   * Update user profile
   */
  async updateUserProfile(data: { displayName?: string; photoURL?: string }): Promise<void> {
    try {
      const user = auth.currentUser;
      if (!user) {
        throw new AuthError(ErrorCode.USER_NOT_FOUND, 'No user found');
      }
      await updateProfile(user, data);
    } catch (error) {
      throw this.handleAuthError(error as FirebaseAuthError);
    }
  }

  /**
   * Link Google account to current user
   */
  async linkGoogleAccount(): Promise<void> {
    try {
      const user = auth.currentUser;
      if (!user) {
        throw new AuthError(ErrorCode.USER_NOT_FOUND, 'No user found');
      }

      const provider = new GoogleAuthProvider();
      await linkWithPopup(user, provider);
    } catch (error) {
      throw this.handleAuthError(error as FirebaseAuthError);
    }
  }

  /**
   * Unlink provider from current user
   */
  async unlinkProvider(providerId: string): Promise<void> {
    try {
      const user = auth.currentUser;
      if (!user) {
        throw new AuthError(ErrorCode.USER_NOT_FOUND, 'No user found');
      }

      await unlink(user, providerId);
    } catch (error) {
      throw this.handleAuthError(error as FirebaseAuthError);
    }
  }

  /**
   * Change password
   */
  async changePassword(currentPassword: string, newPassword: string): Promise<void> {
    try {
      const user = auth.currentUser;
      if (!user || !user.email) {
        throw new AuthError(ErrorCode.USER_NOT_FOUND, 'No user found');
      }

      // Reauthenticate user before changing password
      const credential = EmailAuthProvider.credential(user.email, currentPassword);
      await reauthenticateWithCredential(user, credential);
      
      // Update password
      await updatePassword(user, newPassword);
    } catch (error) {
      throw this.handleAuthError(error as FirebaseAuthError);
    }
  }

  /**
   * Get current user's ID token
   */
  async getCurrentUserToken(forceRefresh = false): Promise<string | null> {
    try {
      const user = auth.currentUser;
      if (!user) return null;
      
      return await user.getIdToken(forceRefresh);
    } catch (error) {
      throw this.handleAuthError(error as FirebaseAuthError);
    }
  }

  /**
   * Handle Firebase auth errors and convert to custom AuthError
   */
  private handleAuthError(error: FirebaseAuthError): AuthError {
    switch (error.code) {
      case 'auth/user-not-found':
      case 'auth/wrong-password':
        return new AuthError(ErrorCode.INVALID_CREDENTIALS, 'Invalid email or password');
      
      case 'auth/email-already-in-use':
        return new AuthError(ErrorCode.VALIDATION_ERROR, 'This email is already registered');
      
      case 'auth/weak-password':
        return new AuthError(ErrorCode.VALIDATION_ERROR, 'Password should be at least 6 characters');
      
      case 'auth/invalid-email':
        return new AuthError(ErrorCode.VALIDATION_ERROR, 'Please enter a valid email address');
      
      case 'auth/user-disabled':
        return new AuthError(ErrorCode.FORBIDDEN, 'This account has been disabled');
      
      case 'auth/too-many-requests':
        return new AuthError(ErrorCode.VALIDATION_ERROR, 'Too many failed attempts. Please try again later');
      
      case 'auth/network-request-failed':
        return new AuthError(ErrorCode.NETWORK_ERROR, 'Network error. Please check your connection');
      
      case 'auth/popup-closed-by-user':
        return new AuthError(ErrorCode.USER_CANCELLED, 'Sign-in was cancelled');
      
      case 'auth/requires-recent-login':
        return new AuthError(ErrorCode.SESSION_EXPIRED, 'Please sign in again to complete this action');
      
      case 'auth/provider-already-linked':
        return new AuthError(ErrorCode.VALIDATION_ERROR, 'This account is already linked');
      
      case 'auth/no-such-provider':
        return new AuthError(ErrorCode.VALIDATION_ERROR, 'This provider is not linked to your account');
      
      default:
        console.error('Unhandled auth error:', error);
        return new AuthError(ErrorCode.UNKNOWN_ERROR, error.message || 'An unexpected error occurred');
    }
  }
}

// Export singleton instance
export const authService = AuthService.getInstance();
