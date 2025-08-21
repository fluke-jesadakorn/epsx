import { redirect } from 'next/navigation';

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
  const backendUrl = process.env.NEXT_PUBLIC_API_URL || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  
  // Build OAuth register URL with proper parameters
  const params = new URLSearchParams({
    client_id: 'epsx-frontend',
    redirect_uri: `${process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000'}/api/auth/callback/epsx-backend`,
    state: Buffer.from(JSON.stringify({ redirectTo })).toString('base64url'),
  });
  
  const registerUrl = `${backendUrl}/oauth/register?${params.toString()}`;
  
  // Server-side redirect to backend registration
  redirect(registerUrl);
}
