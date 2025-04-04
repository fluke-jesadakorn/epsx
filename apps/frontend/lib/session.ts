import { cookies } from 'next/headers';
import { auth } from './firebase-admin';

const SESSION_KEY = '__session';
const MAX_AGE = 60 * 60 * 24 * 5; // 5 days in seconds

export async function createSession(idToken: string) {
  try {
    const sessionCookie = await auth.createSessionCookie(idToken, {
      expiresIn: MAX_AGE * 1000, // Convert to milliseconds
    });

    const cookieStore = await cookies();
    cookieStore.set(SESSION_KEY, sessionCookie, {
      maxAge: MAX_AGE,
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      path: '/',
    });

    return { success: true };
  } catch (error) {
    console.error('Failed to create session:', error);
    return { success: false, error: 'Failed to create session' };
  }
}

export async function verifySession() {
  const cookieStore = await cookies();
  const session = cookieStore.get(SESSION_KEY)?.value;

  if (!session) {
    return null;
  }

  try {
    const decodedClaims = await auth.verifySessionCookie(session, true);
    return decodedClaims;
  } catch (error) {
    console.error('Failed to verify session:', error);
    return null;
  }
}

export async function destroySession() {
  const cookieStore = await cookies();
  const session = cookieStore.get(SESSION_KEY)?.value;

  if (session) {
    try {
      const decodedClaims = await auth.verifySessionCookie(session);
      await auth.revokeRefreshTokens(decodedClaims.sub);
    } catch (error) {
      console.error('Failed to revoke refresh tokens:', error);
    }
  }

  cookieStore.delete(SESSION_KEY);
}

export interface SessionClaims {
  uid: string;
  email?: string;
  email_verified?: boolean;
  name?: string;
  picture?: string;
  iss: string;
  aud: string;
  auth_time: number;
  exp: number;
  iat: number;
  sub: string;
}
