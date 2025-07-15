import { cookies } from 'next/headers';
import { getAuthAdmin } from './firebase-admin';

const SESSION_KEY = '__session';
const MAX_AGE = 60 * 60 * 24 * 5; // 5 days in seconds

export async function createSession(token: string) {
  const cookieStore = await cookies();
  
  // Set the session cookie with explicit configuration
  cookieStore.set(SESSION_KEY, token, {
    maxAge: MAX_AGE,
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    path: '/',
  });

  console.log('Session cookie set with token length:', token.length);
  return { success: true };
}

export async function verifySession(): Promise<SessionClaims | null> {
  try {
    const cookieStore = await cookies();
    const token = cookieStore.get(SESSION_KEY)?.value || null;
    if (!token) {
      console.log('No session token found');
      return null;
    }

    console.log('Verifying session token, length:', token.length);
    // Verify the Firebase ID token without checkRevoked to avoid API calls that can cause loops
    const auth = getAuthAdmin();
    const decodedToken = await auth.verifyIdToken(token, false); // checkRevoked = false for better performance
    console.log('Session verified successfully for user:', decodedToken.uid);
    
    return {
      uid: decodedToken.uid,
      email: decodedToken.email,
      email_verified: decodedToken.email_verified,
      name: decodedToken.name,
      picture: decodedToken.picture,
      exp: decodedToken.exp,
    };
  } catch (error) {
    console.error('Session verification failed:', error);
    // Only clear invalid session cookie if the error is not a network/temporary issue
    if (error && typeof error === 'object' && 'code' in error) {
      const errorCode = (error as any).code;
      // Don't clear session for temporary network issues
      if (errorCode !== 'auth/network-request-failed' && errorCode !== 'auth/internal-error') {
        try {
          const cookieStore = await cookies();
          cookieStore.delete(SESSION_KEY);
          console.log('Cleared invalid session cookie');
        } catch (deleteError) {
          console.error('Failed to clear invalid session cookie:', deleteError);
        }
      }
    }
    return null;
  }
}

export async function destroySession() {
  const cookieStore = await cookies();
  console.log('Destroying session cookie');
  cookieStore.delete(SESSION_KEY);
}

export interface SessionClaims {
  uid: string;
  email?: string;
  email_verified?: boolean;
  name?: string;
  picture?: string;
  exp?: number;
}
