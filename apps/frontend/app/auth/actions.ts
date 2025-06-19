import { verifySession, destroySession } from '@/lib/session';

export async function getCurrentSession(): Promise<{
  userId: string;
  email?: string;
  expiresAt: number;
} | null> {
  const claims = await verifySession();
  if (!claims) return null;

  return {
    userId: claims.uid,
    email: claims.email,
    expiresAt: claims.exp * 1000, // Convert to milliseconds
  };
}

export async function logout() {
  'use server';
  await destroySession();
}
