'use server'

import { redirect } from 'next/navigation';
import { createSession, destroySession, verifySession } from '@/lib/session';

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

export async function getCurrentUser() {
  try {
    const claims = await verifySession();
    return claims;
  } catch (error) {
    console.error('Get current user error:', error);
    return null;
  }
}
