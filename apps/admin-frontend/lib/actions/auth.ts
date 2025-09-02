'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { env } from '@/config/env';

export async function handleSignOut() {
  // Use OIDC logout endpoint to properly revoke tokens
  const backendUrl = env.BACKEND_URL;
  
  // OIDC Migration: Get current OIDC access token for revocation
  const cookieStore = await cookies();
  const jwt = cookieStore.get('access_token')?.value;
  
  if (jwt) {
    try {
      // Call OIDC logout endpoint to revoke token
      await fetch(`${backendUrl}/oauth/logout`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${jwt}`
        }
      });
    } catch (logoutError) {
      console.error('❌ Backend logout failed:', logoutError);
      // Continue with local logout even if backend fails
    }
  }
  
  // OIDC Migration: Clear OIDC tokens instead of legacy JWT
  cookieStore.delete('access_token');
  cookieStore.delete('id_token'); 
  cookieStore.delete('refresh_token');
  
  // Redirect to login page with proper PKCE flow
  redirect('/login');
}