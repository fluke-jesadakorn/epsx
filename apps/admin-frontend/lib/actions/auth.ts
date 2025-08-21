'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { env } from '@/config/env';

export async function handleSignOut() {
  // Use OIDC logout endpoint to properly revoke tokens
  const backendUrl = env.BACKEND_URL;
  
  // Get current JWT from cookie for revocation (check admin cookie first)
  const cookieStore = await cookies();
  const jwt = cookieStore.get('epsx_admin_jwt')?.value || cookieStore.get('epsx_jwt')?.value;
  
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
  
  // Clear JWT cookies locally (both admin and standard)
  cookieStore.delete('epsx_admin_jwt');
  cookieStore.delete('epsx_jwt');
  
  // Redirect to backend Chef Kitchen login - NEXT_REDIRECT error is expected behavior
  const { redirectToBackendAdminLogin } = await import('@/lib/server/auth');
  redirectToBackendAdminLogin();
}