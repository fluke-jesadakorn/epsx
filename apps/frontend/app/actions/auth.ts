'use server';

import { createSession, destroySession, verifySession } from '@/lib/session';

import { getPaymentDetails } from './payment';

import type { User } from '@/types/auth/user';

export async function handleSignIn(idToken: string) {
  try {
    console.log('Creating session with token');
    const result = await createSession(idToken);
    if (!result.success) {
      throw new Error('Failed to create session');
    }
    console.log('Session created successfully');
    return { success: true };
  } catch (_error) {
    console.error('Sign-in error:', _error);
    throw new Error('Authentication failed');
  }
}

export async function refreshSession(): Promise<{ success: boolean }> {
  try {
    // This will check if current session is valid
    const currentUser = await getCurrentUser();
    if (currentUser) {
      console.log('Session is valid, no refresh needed');
      return { success: true };
    }
    
    console.log('Session needs refresh or is invalid');
    return { success: false };
  } catch (_error) {
    console.error('Session refresh check failed:', _error);
    return { success: false };
  }
}

export async function handleSignOut() {
  try {
    console.log('Destroying session');
    await destroySession();
    console.log('Session destroyed successfully');
  } catch (_error) {
    console.error('Sign-out error:', _error);
    throw new Error('Failed to sign out');
  }
}

export async function getCurrentUser(): Promise<User | null> {
  try {
    const token = await verifySession();
    if (!token) return null;

    // Use token properties directly as it is a SessionClaims object
    const user: User = {
      id: token.uid,
      email: token.email || '',
      createdAt: token.exp ? new Date(token.exp * 1000).toISOString() : new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      emailVerified: token.email_verified || false,
      role: 'USER',
      displayName: token.name || undefined,
      photoURL: token.picture || undefined
    };

    const usdtDetails = await getPaymentDetails(user.id);
    return {
      ...user,
      usdtDetails
    };
  } catch (_error) {
    // console.error('Get current user error:', _error);
    return null;
  }
}
