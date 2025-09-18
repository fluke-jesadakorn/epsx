import { redirect } from 'next/navigation';
import { URL, URLContext, Service } from '../../../../shared/utils/url-resolver';

interface RegisterPageProps {
  searchParams: {
    redirect?: string;
    callbackUrl?: string;
    error?: string;
  };
}

export default async function RegisterPage({ searchParams }: RegisterPageProps) {
  const awaitedSearchParams = await searchParams;
  const redirectTo = awaitedSearchParams.redirect || awaitedSearchParams.callbackUrl || '/dashboard';
  
  // Get backend URL from environment
  const backendUrl = URL.get(Service.BACKEND, URLContext.SERVER);
  
  // Build OAuth register URL with proper parameters
  const params = new URLSearchParams({
    client_id: 'epsx-frontend',
    redirect_uri: URL.callback(Service.FRONTEND, URLContext.SERVER),
    state: Buffer.from(JSON.stringify({ redirectTo })).toString('base64url'),
  });
  
  const registerUrl = `${backendUrl}/oauth/register?${params.toString()}`;
  
  // Server-side redirect to backend registration
  redirect(registerUrl);
}
