'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';

export async function handleSignOut() {
  try {
    // Clear JWT cookie
    const cookieStore = await cookies();
    cookieStore.delete('epsx_jwt');
    
    // Redirect to login page
    redirect('/login');
  } catch (error) {
    console.error('❌ Sign out error:', error);
    redirect('/login');
  }
}