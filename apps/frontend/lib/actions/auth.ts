'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';

export async function handleSignOut() {
  // Clear JWT cookie
  const cookieStore = await cookies();
  cookieStore.delete('epsx_frontend_jwt');
  
  // Redirect to backend Pancake login page - NEXT_REDIRECT error is expected behavior
  const backendLoginUrl = new URL('/oauth/authorize', process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io');
  backendLoginUrl.searchParams.set('client_id', process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend');
  backendLoginUrl.searchParams.set('redirect_uri', `${process.env.NEXT_PUBLIC_APP_URL || 'https://epsx.io'}/api/auth/callback/epsx-backend`);
  backendLoginUrl.searchParams.set('scope', 'openid profile email');
  backendLoginUrl.searchParams.set('response_type', 'code');
  redirect(backendLoginUrl.toString());
}