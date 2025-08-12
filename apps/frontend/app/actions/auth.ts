'use server';

import { redirect } from 'next/navigation';
import { cookies } from 'next/headers';

/**
 * Logout action that handles OIDC auth cleanup
 */
export async function logoutAction(): Promise<void> {
  try {
    const cookieStore = cookies();
    
    // Clear all authentication cookies
    cookieStore.delete('auth-token');
    cookieStore.delete('refresh-token');
    cookieStore.delete('id-token');
    
    // Log successful logout
    console.log('🔐 User logout action completed', {
      timestamp: new Date().toISOString(),
      action: 'logout'
    });

    // Redirect to logout endpoint for complete cleanup
    redirect('/auth/logout');
  } catch (error) {
    console.error('Logout action failed:', error);
    throw error;
  }
}