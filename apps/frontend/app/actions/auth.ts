'use server';

import { auth } from '@/lib/auth';
import { redirect } from 'next/navigation';

/**
 * Ensure user is not authenticated (guest only pages)
 * Uses custom iron-session based authentication
 */
export async function requireGuest(): Promise<void> {
  const session = await auth();
  
  // If authenticated, redirect to dashboard
  if (session?.user) {
    redirect('/dashboard');
  }
}