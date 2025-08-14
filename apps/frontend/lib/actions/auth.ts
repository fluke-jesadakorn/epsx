'use server';

import { signOut } from '@/lib/auth';

export async function handleSignOut() {
  // Use custom signOut with proper cleanup
  await signOut();
}