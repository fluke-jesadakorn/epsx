'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';

export async function handleSignOut() {
  // Clear JWT cookie
  const cookieStore = await cookies();
  cookieStore.delete('epsx_frontend_jwt');
  
  // Redirect to backend Pancake login page - NEXT_REDIRECT error is expected behavior
  const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || (process.env.NODE_ENV === 'development' ? 'http://localhost:8080' : undefined);
  const appUrl = process.env.NEXT_PUBLIC_APP_URL || (process.env.NODE_ENV === 'development' ? 'http://localhost:3000' : undefined);
  const clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || (process.env.NODE_ENV === 'development' ? 'epsx-frontend' : undefined);
  
  if (!backendUrl || !appUrl || !clientId) {
    throw new Error('Required environment variables (NEXT_PUBLIC_BACKEND_URL, NEXT_PUBLIC_APP_URL, NEXT_PUBLIC_OAUTH_CLIENT_ID) are missing');
  }
  
  const backendLoginUrl = new URL('/oauth/authorize', backendUrl);
  backendLoginUrl.searchParams.set('client_id', clientId);
  backendLoginUrl.searchParams.set('redirect_uri', `${appUrl}/api/auth/callback/epsx-backend`);
  backendLoginUrl.searchParams.set('scope', 'openid profile email');
  backendLoginUrl.searchParams.set('response_type', 'code');
  redirect(backendLoginUrl.toString());
}