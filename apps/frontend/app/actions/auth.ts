'use server';

import { createSession, destroySession, verifySession } from '@/lib/session';
import type { User } from '@/types/auth/user';
import { getPaymentDetails } from './payment';

export async function handleSignIn(idToken: string) {
  try {
    const result = await createSession(idToken);
    if (!result.success) {
      throw new Error('Failed to create session');
    }
  } catch (error) {
    console.error('Sign-in error:', error);
    throw new Error('Authentication failed');
  }
}

export async function handleSignOut() {
  try {
    await destroySession();
  } catch (error) {
    console.error('Sign-out error:', error);
    throw new Error('Failed to sign out');
  }
}

export async function getCurrentUser(): Promise<User | null> {
  try {
    const token = await verifySession();
    if (!token) return null;

    // Decode JWT token to get user information
    const payload = JSON.parse(atob(token.split('.')[1]));
    
    const user: User = {
      id: payload.sub,
      email: payload.email || '',
      createdAt: payload.iat ? new Date(payload.iat * 1000).toISOString() : new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      emailVerified: payload.email_verified || false,
      role: 'USER',
      displayName: payload.name || undefined,
      photoURL: payload.picture || undefined
    };

    const usdtDetails = await getPaymentDetails(user.id);
    return {
      ...user,
      usdtDetails
    };
  } catch (error) {
    console.error('Get current user error:', error);
    return null;
  }
}
