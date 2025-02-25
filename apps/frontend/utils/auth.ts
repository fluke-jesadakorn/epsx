import { signInWithEmailAndPassword, createUserWithEmailAndPassword, signInWithPopup } from 'firebase/auth';
import { auth } from '@/lib/firebase-client';
import { AuthProvider } from 'firebase/auth';

interface SignUpCredentials {
  email: string;
  password: string;
}

export const signInWithOAuth = async (provider: AuthProvider) => {
  try {
    const result = await signInWithPopup(auth, provider);
    return result.user;
  } catch (error) {
    console.error('Error signing in with OAuth:', error);
    throw error;
  }
};

export const signUpWithEmailPassword = async ({ email, password }: SignUpCredentials) => {
  try {
    const userCredential = await createUserWithEmailAndPassword(auth, email, password);
    return userCredential.user;
  } catch (error) {
    console.error('Error signing up with email/password:', error);
    throw error;
  }
};

export const signInWithEmailPassword = async ({ email, password }: SignUpCredentials) => {
  try {
    const userCredential = await signInWithEmailAndPassword(auth, email, password);
    return userCredential.user;
  } catch (error) {
    console.error('Error signing in with email/password:', error);
    throw error;
  }
};

export const signOut = async () => {
  try {
    await auth.signOut();
  } catch (error) {
    console.error('Error signing out:', error);
    throw error;
  }
};
