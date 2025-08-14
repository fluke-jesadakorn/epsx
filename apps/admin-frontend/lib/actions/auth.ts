'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';

export async function handleSignOut() {
  try {
    // Use OIDC logout endpoint to properly revoke tokens
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    
    // Get current JWT from cookie for revocation
    const cookieStore = await cookies();
    const jwt = cookieStore.get('epsx_jwt')?.value;
    
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
    
    // Clear JWT cookie locally
    cookieStore.delete('epsx_jwt');
    
    // Redirect to login page
    redirect('/login');
  } catch (error) {
    console.error('❌ Admin sign out error:', error);
    redirect('/login');
  }
}