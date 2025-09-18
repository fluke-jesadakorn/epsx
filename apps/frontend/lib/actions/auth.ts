'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { getBackendUrl, getFrontendUrl, oidcUrls, callbackUrls } from '../../../../shared/utils/url-resolver';

export async function handleSignOut() {
  // Clear JWT cookie
  const cookieStore = await cookies();
  cookieStore.delete('epsx_frontend_jwt');
  
  // Redirect to backend OAuth login page - NEXT_REDIRECT error is expected behavior
  const backendLoginUrl = new URL('/oauth/authorize', getBackendUrl('server'));
  backendLoginUrl.searchParams.set('client_id', 'epsx-frontend');
  backendLoginUrl.searchParams.set('redirect_uri', callbackUrls.frontend('server'));
  backendLoginUrl.searchParams.set('scope', 'openid profile email');
  backendLoginUrl.searchParams.set('response_type', 'code');
  redirect(backendLoginUrl.toString());
}