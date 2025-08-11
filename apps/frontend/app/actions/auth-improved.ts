'use server';

import { redirect } from 'next/navigation';
import { getCurrentUser } from '@epsx/server-actions';

/**
 * Ensure the user is NOT authenticated (guest only)
 * Redirects to home page if user is already logged in
 */
export async function requireGuest(): Promise<void> {
  const user = await getCurrentUser();
  
  if (user) {
    redirect('/');
  }
}