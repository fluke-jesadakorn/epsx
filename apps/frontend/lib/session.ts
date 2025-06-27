import { cookies } from 'next/headers';

const SESSION_KEY = '__session';
const MAX_AGE = 60 * 60 * 24 * 5; // 5 days in seconds

export async function createSession(token: string) {
  const cookieStore = await cookies();
  cookieStore.set(SESSION_KEY, token, {
    maxAge: MAX_AGE,
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    path: '/',
  });

  return { success: true };
}

export async function verifySession() {
  const cookieStore = await cookies();
  return cookieStore.get(SESSION_KEY)?.value || null;
}

export async function destroySession() {
  const cookieStore = await cookies();
  cookieStore.delete(SESSION_KEY);
}

export interface SessionClaims {
  uid: string;
  email?: string;
  email_verified?: boolean;
  name?: string;
  picture?: string;
}
